use super::observers::*;
use bevy_app::{App, Plugin};

pub struct IrohGossipPlugin;

impl Plugin for IrohGossipPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::IrohCorePlugin>() {
            app.add_plugins(crate::IrohCorePlugin);
        }
        app.add_observer(super::observers::startup_gossip)
            .add_observer(super::observers::run_gossip)
            .add_observer(on_send_gossip_message);
    }
}
