use bevy_ecs::{
    observer::On,
    system::{Commands, ResMut},
};

use super::events::*;

pub fn spawn_router(
    _event: On<BuilderReady>,
    mut option: ResMut<super::resources::RouterBuilder>,
    mut commands: Commands,
) {
    let Some(builder) = option.take() else {return;};
    let router = builder.spawn();
    commands.insert_resource(super::resources::Router::new(router));
    commands.trigger(RouterSpawned);
}

pub fn build_router(trigger: On<EndpointBinded>, mut commands: Commands) {
    let endpoint = trigger.0.clone();

    let router_builder = iroh::protocol::Router::builder(endpoint);
    tracing::info!("RouterBuilder created.");

    commands.insert_resource(super::resources::RouterBuilder::new(Some(router_builder)));
    tracing::info!("RouterBuilder inserted as a resource.");

    commands.trigger(BuilderCreated);
    tracing::info!("RouterBulder started to accept protocols.");

    commands.trigger(BuilderReady);
    tracing::info!("All protocols are accepted. Building the router.");
}
