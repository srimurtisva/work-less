use bevy_ecs::{
    observer::On,
    system::{Commands, ResMut},
};

use crate::General;

pub fn spawn_router(
    _event: On<super::events::BuilderReady>,
    mut option: ResMut<super::resources::RouterBuilder>,
    mut commands: Commands,
) {
    if let Some(builder) = option.take() {
        let router = builder.spawn();
        commands.insert_resource(super::resources::Router::new(router));
        commands.trigger(super::events::RouterSpawned);
    }
}

pub fn build_router(trigger: On<super::events::General>, mut commands: Commands) {
    if let General::EndpointBinded(endpoint) = trigger.event() {
        let router_builder = iroh::protocol::Router::builder(endpoint.to_owned());
        tracing::info!("RouterBuilder created.");

        // sender.send(General::BuilderCreated);
        commands.insert_resource(super::resources::RouterBuilder::new(Some(router_builder)));
        tracing::info!("RouterBuilder inserted as a resource.");

        commands.trigger(super::events::BuilderCreated);
        tracing::info!("RouterBulder started to accept protocols.");

        commands.trigger(super::events::BuilderReady);
        tracing::info!("All protocols are accepted. Building the router.");
    }
}
