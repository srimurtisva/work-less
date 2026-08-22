use bevy_ecs::prelude::*;
use tokio::sync::mpsc;

#[derive(Resource)]
pub struct GossipRuntime {
    pub command_tx: Option<mpsc::UnboundedSender<GossipCommand>>,
    pub event_rx: Option<mpsc::UnboundedReceiver<GossipEventInternal>>,
}
