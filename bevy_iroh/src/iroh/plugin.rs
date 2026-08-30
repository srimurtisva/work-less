use crate::iroh::systems::*;
use bevy_app::{App, Plugin, PreStartup, Startup};
use bevy_tokio_tasks::TokioTasksPlugin;

pub struct IrohCorePlugin;
impl Plugin for IrohCorePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TokioTasksPlugin>() {
            app.add_plugins(TokioTasksPlugin::default());
        }
        app
            // .add_systems(PreStartup,event_loop)
            .add_systems(
            Startup,
            startup_endpoint2
        )
        .add_observer(super::observers::spawn_router)
        .add_observer(super::observers::build_router)
        ;
    }
} 
