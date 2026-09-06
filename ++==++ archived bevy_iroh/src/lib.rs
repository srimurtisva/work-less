pub mod iroh;
pub mod gossip;
// pub mod blobs;
// pub mod docs;

use bevy_app::{App, Plugin};

use bevy_ecs::{observer::On, resource::Resource};
use derive_more::{DerefMut,Deref};
// Ре-экспорты для удобства
pub use iroh::plugin::IrohCorePlugin;
pub use gossip::plugin::IrohGossipPlugin;
// pub use blobs::plugin::IrohBlobsPlugin;
// pub use docs::plugin::IrohDocsPlugin;
//
pub use iroh::events::*;
pub use gossip::events::*;

/// Универсальный плагин, добавляющий все подсистемы Iroh
pub struct IrohPlugin;

impl Plugin for IrohPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(IrohCorePlugin)
           .add_plugins(IrohGossipPlugin)
           // .add_plugins(IrohBlobsPlugin)
           // .add_plugins(IrohDocsPlugin)
           ;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_tokio_tasks::TokioTasksPlugin;


 /// Тест: IrohPlugin должен добавлять TokioTasksPlugin, если он ещё не добавлен.
    #[test]
    fn iroh_plugin_adds_tokio_tasks_plugin() {
        let mut app = App::new();
        // Убедимся, что TokioTasksPlugin ещё не добавлен.
        assert!(!app.is_plugin_added::<TokioTasksPlugin>());

        app.add_plugins(IrohPlugin);

        // Теперь он должен быть добавлен.
        assert!(app.is_plugin_added::<TokioTasksPlugin>());
    }


     /// Тест: IrohPlugin не должен паниковать при повторном добавлении,
    /// даже если TokioTasksPlugin уже присутствует.
    #[test]
    fn iroh_plugin_does_not_duplicate_tokio_tasks() {
        let mut app = App::new();
        app.add_plugins(TokioTasksPlugin::default());
        app.add_plugins(IrohPlugin); // не должно паниковать или дублировать

        // Плагин всё ещё добавлен.
        assert!(app.is_plugin_added::<TokioTasksPlugin>());
        // И сам IrohPlugin тоже.
        assert!(app.is_plugin_added::<IrohPlugin>());
    }

    
}

