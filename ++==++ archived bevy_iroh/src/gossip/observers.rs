
use bevy_ecs::{observer::On, system::Res};
use bevy_tokio_tasks::TokioTasksRuntime;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    JoinTopic,
    gossip::{
        resources::{*},
    },
};

use super::events::*;
use bevy_ecs::system::{Commands, ResMut};
use iroh::EndpointId;
use iroh_gossip::TopicId;

use crate::gossip::resources::{BootstrapPeers, Topic};

pub fn startup_gossip(
    _event: On<crate::iroh::events::BuilderCreated>,
    mut res: ResMut<crate::iroh::resources::RouterBuilder>,
    mut commands: Commands,
) {
    let Some(builder) = res.take() else {
        tracing::error!("Failed to startup Gossip. There is no RouterBuilder.");
        return;
    };
    let endpoint = builder.endpoint().to_owned();

    let gossip = iroh_gossip::Gossip::builder().spawn(endpoint);
    commands.insert_resource(Gossip::new(gossip.clone()));
    tracing::info!("Gossip inserted to the world.");
    commands.trigger(GossipSpawned);

    let builder = builder.accept(iroh_gossip::ALPN, gossip);
    tracing::info!("Gossip accepted by builder.");

    res.0 = Some(builder);
}

pub fn run_gossip(
    _event: On<crate::iroh::events::RouterSpawned>,
    mut commands: Commands,
    gossip: Res<super::resources::Gossip>,
) {
    let (tokio_sender, mut tokio_receiver) = tokio::sync::mpsc::unbounded_channel::<GossipChannel>();
    let gossip = gossip.to_owned();
    let topic_id = generate_topic_id();
    commands.insert_resource(Topic(topic_id));

    let bootstrap_peers: Vec<EndpointId> = vec![];
    commands.insert_resource(BootstrapPeers(bootstrap_peers.clone()));

    tokio::spawn(background_task(
        gossip.clone(),
        topic_id,
        bootstrap_peers,
        tokio_sender,
    ));

    if let Ok(gossip_channel) = tokio_receiver.try_recv() {
        commands.insert_resource(gossip_channel);
        tracing::info!("Gossip channel inserted as a resourse.");
        commands.trigger(TopicOpen);
    }

}

async fn background_task(
    gossip: iroh_gossip::Gossip,
    topic_id: TopicId,
    bootstrap_peers: Vec<EndpointId>,
    tokio_sender: UnboundedSender<GossipChannel>,
) {
    let topic = match gossip.subscribe(topic_id, bootstrap_peers).await {
        Ok(topic) => {
            tracing::info!("Gossip subscribed.");
            topic
        }
        Err(error) => {
            tracing::error!("Failed to subscribe to gossip topic: {error}");
            return;
        }
    };

    let (gossip_sender, gossip_receiver) = topic.split();

    let _ = tokio_sender.send(GossipChannel{
        receiver: gossip_receiver,
        sender: gossip_sender,
    });
}

// async fn receive_messages(
//     topic_id: TopicId,
//     mut receiver: iroh_gossip::api::GossipReceiver,
// ) {
//     while let Some(event) = receiver.next().await {
//         let event = match event {
//             Ok(event) => event,
//             Err(error) => {
//                 tracing::error!("Gossip receiver error: {error}");
//                 break;
//             }
//         };
//     }
// }

// pub fn on_send_gossip_message(
//     trigger: On<SendGossipMessage>,
//     sender: Option<Res<GossipSender>>,
//     runtime: Res<TokioTasksRuntime>,
// ) {
//     let Some(sender) = sender else {
//         tracing::warn!("GossipSender not ready yet");
//         return;
//     };
//     let message = trigger.event().content.clone();
//     let sender = sender.0.clone();

//     runtime.spawn_background_task(move |_ctx| async move {
//         if let Err(error) = sender.broadcast(message.into()).await {
//             tracing::error!("Gossip broadcast error: {error}");
//         }
//     });
// }

pub fn on_join_topic(event: On<JoinTopic>, gossip: Res<Gossip>, runtime: Res<TokioTasksRuntime>) {
    let topic = event.0;
    let gossip = gossip.0.clone();

    runtime.spawn_background_task(move |mut ctx| async move {
        // let mut stream = match gossip.su
    });
}

fn generate_topic_id() -> TopicId {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(b"com.work_less.gossip");

    TopicId::from_bytes(hasher.finalize().into())
}

