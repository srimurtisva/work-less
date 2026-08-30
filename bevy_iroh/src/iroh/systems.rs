use crate::iroh::events::{EndpointBinded, General};
use bevy_ecs::prelude::*;
use iroh::endpoint::presets;

// pub fn startup_endpoint(runtime: ResMut<TokioTasksRuntime>) {
//     runtime.spawn_background_task(move |mut ctx| async move {
//         match create_endpoint().await {
//             Ok(endpoint) => {
//                 tracing::info!("Iroh Endpoint succesfuly started.");
//                 ctx.run_on_main_thread(move |ctx| {
//                     let world = ctx.world;

//                     world.trigger(EndpointBinded(endpoint.id().to_string()));
//                     tracing::info!("EndpointBinded event triggered.");
//                 })
//                 .await;
//             }
//             Err(error) => {
//                 tracing::error!(%error,"Failed to create Iroh Endpoint.");
//             }
//         };
//     });
// }

async fn create_endpoint() -> Result<iroh::Endpoint, iroh::endpoint::BindError> {
    let key = super::util::NodeIdentityManager::auto().get_or_create_key();

    let dht = iroh_mainline_address_lookup::DhtAddressLookup::builder();
    let mdns = iroh_mdns_address_lookup::MdnsAddressLookup::builder();
    iroh::Endpoint::builder(presets::N0)
        .secret_key(key)
        .address_lookup(dht)
        .address_lookup(mdns)
        .bind()
        .await
}

pub fn startup_endpoint2(mut commands: Commands) {
    let (sender, reciever) = std::sync::mpsc::channel::<General>();

    tokio::spawn(async move {
        match create_endpoint().await {
            Ok(endpoint) => {
                tracing::info!("Iroh Endpoint succesfuly started.");

                send_event(sender, General::EndpointBinded(endpoint));
                tracing::info!("EndpointBinded event triggered.");
            }
            Err(error) => {
                tracing::error!(%error,"Failed to create Iroh Endpoint.");
            }
        };
    });

    if let Ok(event) = reciever.recv() {
        commands.trigger(event);
        tracing::info!("Triggered event.");
    }
}

fn send_event(sender: std::sync::mpsc::Sender<crate::General>, event: General) {
    let result = sender.send(event);
    if let Err(error) = result {
        tracing::error!("Failed to send event: {error}");
    };
}

// pub fn event_loop(mut commands: Commands) {
//     let (sender, reciever) = std::sync::mpsc::channel::<General>();
//     commands.insert_resource(super::resources::MainThreadWaker(sender));
//     tracing::info!("Resource MainthreadWaker inserted.");
//     while let Ok(event) = reciever.try_recv() {
//         commands.trigger(event);
//         tracing::info!("Triggered event.");
//     }
// }
