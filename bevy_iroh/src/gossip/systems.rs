use super::resources::Gossip;
use super::{events::*, resources::GossipSender};
use bevy_ecs::system::{Commands, Res, ResMut};
use bevy_tokio_tasks::TokioTasksRuntime;
use iroh::EndpointId;
use iroh_gossip::TopicId;
use iroh_gossip::api::Event;
use n0_future::StreamExt;

use crate::gossip::resources::{BootstrapPeers, Topic};

pub fn startup_gossip(
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
    mut commands: Commands,
    runtime: Res<TokioTasksRuntime>,
    gossip: Res< super::resources::Gossip>,
) {
    let gossip = gossip.to_owned();
    let topic_id = generate_topic_id();
    commands.insert_resource(Topic(topic_id));

    let bootstrap_peers: Vec<EndpointId> = vec![];
    commands.insert_resource(BootstrapPeers(bootstrap_peers.clone()));

    runtime.spawn_background_task(move |mut ctx| async move {
        let topic = match gossip.subscribe(topic_id, bootstrap_peers).await {
            Ok(topic) => topic,
            Err(error) => {
                tracing::error!("Failed to subscribe to gossip topic: {error}");
                return;
            }
        };

        let (sender, receiver) = topic.split();
        ctx.run_on_main_thread(move |ctx| {
            ctx.world.insert_resource(GossipSender(sender));
        })
        .await;

        receive_messages(topic_id, ctx, receiver).await;
    });
}

fn generate_topic_id() -> TopicId {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(b"com.work_less.gossip");

    TopicId::from_bytes(hasher.finalize().into())
}

async fn receive_messages(
    topic_id: TopicId,
    mut ctx: bevy_tokio_tasks::TaskContext,
    mut receiver: iroh_gossip::api::GossipReceiver,
) {
    while let Some(event) = receiver.next().await {
        let event = match event {
            Ok(event) => event,
            Err(error) => {
                tracing::error!("Gossip receiver error: {error}");
                break;
            }
        };

        match_event(event, topic_id, &mut ctx).await;
    }
}

// TODO: Trigger separeate events Instead of processing them.
async fn match_event(event: Event, topic_id: TopicId, ctx: &mut bevy_tokio_tasks::TaskContext) {
    match event {
        Event::Received(message) => {
            let content = message.content;
            let content = match std::str::from_utf8(&content) {
                Ok(content) => content,
                Err(error) => {
                    tracing::error!("Failed to recognize UTF8: {error}");
                    return;
                }
            }
            .to_string();
            let from = message.delivered_from;
            ctx.run_on_main_thread(move |ctx| {
                ctx.world.trigger(GossipMessageReceived { from, content });
            })
            .await;
        }

        Event::NeighborUp(public_key) => {
            tracing::debug!("Gossip neighbor joined: {public_key}");
        }
        Event::NeighborDown(public_key) => {
            tracing::debug!("Gossip neighbor left: {public_key}");
        }
        Event::Lagged => {
            tracing::debug!(
                "Missed some messages becouse GossipReceiver is not progressing fast enough "
            );
        }
    }
}
