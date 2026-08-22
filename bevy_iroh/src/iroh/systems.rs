use crate::iroh::components::{Gossip, Router};
use crate::iroh::events::{Error, NodeStarted,EndpointBinded};
use bevy_ecs::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use iroh::Endpoint;
use iroh::endpoint::presets;

pub fn startup_iroh(
    runtime: ResMut<TokioTasksRuntime>,
) {
    let key = super::util::NodeIdentityManager::auto().get_or_create_key();

    runtime.spawn_background_task(move |mut ctx| async move {
        let endpoint = match Endpoint::builder(presets::N0)
            .secret_key(key)
            .bind()
            .await
        {
            Ok(ep) => {
                let id = ep.id();
                ctx.run_on_main_thread(move |ctx| {
                    let world = ctx.world;
                    world.trigger(EndpointBinded(id.to_string()));
                })
                .await;
                ep},
            Err(e) => {
                ctx.run_on_main_thread(move |ctx| {
                    let world = ctx.world;
                    world.trigger(Error(e.to_string()));
                })
                .await;
                return;
            }
        };

        println!("Iroh Endpoint succesfuly started");
        println!("Node Id: {}", endpoint.id());

        let gossip = iroh_gossip::Gossip::builder().spawn(endpoint.clone());
        let router = iroh::protocol::Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        let node_id = endpoint.id().to_string();
        ctx.run_on_main_thread(move |ctx| {
            let world = ctx.world;
            world.spawn(Router::new(router));
            world.spawn(Gossip::new(gossip));
            world.trigger(NodeStarted(node_id));
        })
        .await;
    });
}
