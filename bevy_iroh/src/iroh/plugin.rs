use bevy_app::{App, Plugin, Startup};
use bevy_tokio_tasks::TokioTasksPlugin;
use crate::iroh:: systems::*;

pub struct IrohCorePlugin;
impl Plugin for IrohCorePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TokioTasksPlugin>() {
            app.add_plugins(TokioTasksPlugin::default());
        }
            app
        // .init_resource::<IrohEndpoint>()
           // .init_resource::<NodeId>()
           // .add_event::<CoreEvent>()
           .add_systems(Startup, startup_iroh);
    }
}
