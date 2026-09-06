use crate::iroh::systems::*;
use bevy_app::{App, Plugin, Startup};
use bevy_tokio_tasks::TokioTasksPlugin;

pub struct IrohCorePlugin;
impl Plugin for IrohCorePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TokioTasksPlugin>() {
            app.add_plugins(TokioTasksPlugin::default());
        }
        app
            .init_resource::<super::resources::Waker>()
            .add_systems(
            Startup,
            startup_endpoint
        )
        .add_observer(super::observers::spawn_router)
        .add_observer(super::observers::build_router)
        ;
    }
} 
