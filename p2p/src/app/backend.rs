use super::*;
use crux_core::command::CommandContext;
use iroh::endpoint::presets;
use iroh_tickets::Ticket;

pub struct P2pWorker {
    router: iroh::protocol::Router,
    _identity_manager: crate::node_identity::NodeIdentityManager,
    #[cfg(feature = "gossip")]
    gossip: iroh_gossip::Gossip,
}

impl P2pWorker {
    async fn new(
        endpoint: iroh::Endpoint,
        identity_manager: crate::node_identity::NodeIdentityManager,
    ) -> anyhow::Result<Self> {
        let mut builder = iroh::protocol::Router::builder(endpoint.clone());

        #[cfg(feature = "gossip")]
        let gossip = {
            let gossip = iroh_gossip::net::Gossip::builder().spawn(endpoint.clone());
            builder = builder.accept(iroh_gossip::ALPN, gossip.clone());
            // ... подписка на топик ...
            gossip
        };

        // 2. Компилируется, только если включена фича "blobs"
        // #[cfg(feature = "blobs")]
        // let blobs_handle = {
        // let blobs = iroh_blobs::net::Blobs::builder().spawn(endpoint.clone());
        // builder = builder.accept(iroh_blobs::ALPN, blobs);
        // };

        // 3. Запускаем роутер
        let router = builder.spawn();

        Ok(Self {
            _identity_manager: identity_manager,
            router,
            #[cfg(feature = "gossip")]
            gossip,
        })
    }

    pub async fn run(
        ctx: CommandContext<Effect, Event>,
        mut outgoing_rx: tokio::sync::mpsc::Receiver<OutgoingCommand>,
    ) {
        tracing::trace!("p2p worker has started to run");

        let mut manager = crate::node_identity::NodeIdentityManager::auto();
        let identity = manager.acquire_identity();

        if !identity.owns_persistent_identity {
            tracing::warn!("Running as a secondary instance");
        }

        tracing::trace!("The key is there");
        let endpoint = match bind_endpoint(identity.key).await {
            Ok(endpoint) => {
                tracing::trace!("Endpoint binded.");
                endpoint
            }
            Err(error) => {
                tracing::error!("Failed to bind endpoint: {error}");
                ctx.send_event(Event::Failed(error.to_string()));
                return;
            }
        };
        let worker = match Self::new(endpoint.clone(), manager).await {
            Ok(w) => {
                tracing::trace!("Worker is running.");
                w
            }
            Err(error) => {
                tracing::error!("Failed to spawn router: {error}");
                ctx.send_event(Event::Failed(error.to_string()));
                return;
            }
        };
        endpoint.online().await;
        tracing::info!(
            id = %endpoint.id(),
            addr = ?endpoint.addr(),
            "Endpoint is online"
        );
        let ticket = iroh_tickets::endpoint::EndpointTicket::new(endpoint.addr()).encode_string();
        ctx.send_event(Event::Ready(ticket));

        tracing::trace!("Event Ready sent");
        // 3. Подключаемся к Gossip топику
        #[cfg(feature = "gossip")]
        {
            let topic_id = iroh_gossip::proto::TopicId::from(*TOPIC_NAME);
            let Ok(topic) = worker.gossip.subscribe(topic_id, vec![]).await else {
                tracing::error!("Failed to subcribe to topic");
                ctx.send_event(Event::Failed("Failed to subscribe to topic".into()));
                return;
            };
            tracing::trace!("Initial topic created");

            let (sink, mut stream) = topic.split();

            tracing::trace!("Gossip subscribed");
            let gossip_ctx = ctx.clone();

            tokio::spawn(async move {
                while let Some(command) = outgoing_rx.recv().await {
                    tracing::trace!("Command received on background thread {:?}", command);
                    match command {
                        OutgoingCommand::Broadcast(bytes) => {
                            let _ = sink.broadcast(bytes::Bytes::from(bytes)).await;
                        }
                        OutgoingCommand::Join(ticket_string) => {
                            tracing::info!("Join request: {ticket_string}");
                            let ticket = match iroh_tickets::endpoint::EndpointTicket::decode_string(
                                &ticket_string,
                            ) {
                                Ok(ticket) => ticket,
                                _ => {
                                    tracing::error!("Failed to decode ticket.");
                                    continue;
                                }
                            };
                            tracing::trace!("Ticket decoded successfully");

                            let peer_id = ticket.endpoint_addr().id;

                            tracing::info!("Requesting gossip connection to {peer_id}");

                            match sink.join_peers(vec![peer_id]).await {
                                Err(error) => {
                                    tracing::error!("Failed to join peer {peer_id}: {error}");

                                    gossip_ctx.send_event(Event::Failed(error.to_string()));
                                }
                                _ => {
                                    tracing::info!("Reqested successfully.");
                                }
                            };
                        }
                    }
                }
            });

            // 4. Бесконечный цикл: worker и worker.router живут ЗДЕСЬ всё время работы приложения!
            use futures_util::StreamExt;
            use iroh_gossip::api::Event as GossipEvent;

            while let Some(item) = stream.next().await {
                match item {
                    Ok(GossipEvent::Received(msg)) => {
                        tracing::info!("Received gossip message from {}", msg.delivered_from);

                        ctx.send_event(Event::DataReceived(msg.content.to_vec()));
                    }
                    Ok(GossipEvent::NeighborUp(peer)) => {
                        tracing::info!("Gossip NeighborUp: {peer}");

                        ctx.send_event(Event::PeerJoined(peer.to_string()));
                    }
                    Ok(GossipEvent::NeighborDown(peer)) => {
                        tracing::info!("Gossip NeighborDown: {peer}");
                        ctx.send_event(Event::PeerLeft(peer.to_string()));
                    }
                    Ok(GossipEvent::Lagged) => {
                        tracing::warn!("Gossip receiver lagged");
                    }
                    Err(error) => {
                        tracing::error!("Failed to receive event: {error}");
                    }
                }
            }
        }

        let _ = worker.shutdown().await;
    }

    pub async fn shutdown(self) -> anyhow::Result<()> {
        tracing::info!("Shutting down Iroh Router...");
        self.router.shutdown().await?;
        Ok(())
    }
}

async fn bind_endpoint(key: iroh::SecretKey) -> Result<iroh::Endpoint, iroh::endpoint::BindError> {
    tracing::trace!("Starting to bind endpoint");
    let dht = iroh_mainline_address_lookup::DhtAddressLookup::builder();
    let mdns = iroh_mdns_address_lookup::MdnsAddressLookup::builder();

    let endpoint = iroh::Endpoint::builder(presets::N0)
        .secret_key(key)
        .address_lookup(dht)
        .address_lookup(mdns)
        .bind()
        .await;
    // tracing::trace!("Endpoint binded. {:?}",endpoint);
    endpoint
}

// use std::fmt;
// use std::str::FromStr;

// // add the `Ticket` code to the bottom of the main file
// #[derive(Debug, Serialize, Deserialize)]
// struct Ticket {
//     topic: TopicId,
//     endpoints: Vec<EndpointAddr>,
// }

// impl Ticket {
//     /// Deserialize from a slice of bytes to a Ticket.
//     fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
//         serde_json::from_slice(bytes).map_err(Into::into)
//     }

//     /// Serialize from a `Ticket` to a `Vec` of bytes.
//     pub fn to_bytes(&self) -> Vec<u8> {
//         serde_json::to_vec(self).expect("serde_json::to_vec is infallible")
//     }
// }

// // The `Display` trait allows us to use the `to_string`
// // method on `Ticket`.
// impl fmt::Display for Ticket {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let mut text = data_encoding::BASE32_NOPAD.encode(&self.to_bytes()[..]);
//         text.make_ascii_lowercase();
//         write!(f, "{}", text)
//     }
// }

// // The `FromStr` trait allows us to turn a `str` into
// // a `Ticket`
// impl FromStr for Ticket {
//     type Err = anyhow::Error;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let bytes = data_encoding::BASE32_NOPAD.decode(s.to_ascii_uppercase().as_bytes())?;
//         Self::from_bytes(&bytes)
//     }
// }
