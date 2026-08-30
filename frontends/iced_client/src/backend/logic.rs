pub mod plugin;
use crate::backend::logic::plugin::AppStatus;
use bevy_app::App;
use bevy_ecs::prelude::*;
use bevy_iroh::{BuilderReady, EndpointBinded, General};
use derive_more::{Deref, DerefMut};
use tokio::sync::mpsc::UnboundedSender;

pub fn prepare_app(wake_tx: UnboundedSender<()>) -> App {
    let mut app = App::new();
    app.insert_resource(MainThreadWaker(wake_tx));

    app.add_plugins(plugin::TextSyncPlugin);
    app.add_plugins(bevy_iroh::IrohPlugin);

    app.add_observer(on_gossip_message_received);
    app.add_observer(on_endpoint_binded);
    app.add_observer(on_builder_ready);

    app.update();

    app
}

// Ресурс для пробуждения Iced
#[derive(Resource, Deref, DerefMut)]
pub struct MainThreadWaker(pub UnboundedSender<()>);

pub fn on_gossip_message_received(
    event: On<bevy_iroh::GossipMessageReceived>,
    // mut ResMut<MainThreadWaker>,
    mut status: ResMut<AppStatus>,
) {
    let text = event.content.clone();
    status.messages.push(text.clone());

    // Ограничиваем историю, например, последними 50 сообщениями
    if status.messages.len() > 50 {
        status.messages.remove(0);
    }
    println!("Message received from bevy {text}");
}

pub fn on_endpoint_binded(
    trigger: On<General>,
    waker: ResMut<MainThreadWaker>,
    mut status: ResMut<AppStatus>,
) {
    let General::EndpointBinded(endpoint) = trigger.event();
    status.node_id = endpoint.id().to_string();
    let _ = waker.send(());
}

pub fn on_builder_ready(_trigger: On<BuilderReady>, mut status: ResMut<AppStatus>) {
    status.shared_text = String::from("Router builser is ready");
}
