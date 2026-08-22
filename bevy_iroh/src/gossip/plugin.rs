use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::*;
use crate::core::resources::IrohEndpoint;
use crate::gossip::{components::*, resources::*, systems::*, events::*};

pub struct IrohGossipPlugin;

impl Plugin for IrohGossipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GossipRuntime>()
           .add_event::<GossipEvent>()
           .add_systems(Update, (join_topic, send_message, receive_messages))
           // инициализация gossip после запуска endpoint
           .add_systems(Startup, init_gossip.after(super::core::systems::startup_iroh));
    }
}
