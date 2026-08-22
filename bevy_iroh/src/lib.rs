pub mod iroh;
// pub mod gossip;
// pub mod blobs;
// pub mod docs;

// Ре-экспорты для удобства
pub use iroh::plugin::IrohCorePlugin;
// pub use gossip::plugin::IrohGossipPlugin;
// pub use blobs::plugin::IrohBlobsPlugin;
// pub use docs::plugin::IrohDocsPlugin;
//
pub use iroh::events::*;

/// Универсальный плагин, добавляющий все подсистемы Iroh
// pub struct IrohPlugin;

// impl Plugin for IrohPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_plugins(IrohCorePlugin)
//            .add_plugins(IrohGossipPlugin)
//            .add_plugins(IrohBlobsPlugin)
//            .add_plugins(IrohDocsPlugin);
//     }
// }

mod iroh_ecs;
pub use iroh_ecs::IrohPlugin;


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

/* src/
├── lib.rs                 // Корень: ре-экспорты, документация
├── core/                  // Базовый endpoint
│   ├── mod.rs
│   ├── plugin.rs          // IrohCorePlugin
│   ├── resources.rs       // IrohEndpoint, NodeId, SecretKeyStore и т.д.
│   ├── systems.rs         // startup_iroh, обработчики │   ├── events.rs          // CoreEvent (NodeStarted, NodeStopped, ...)
│   └── secret_key.rs      // Хранение и загрузка ключа
├── gossip/                // Подсистема gossip
│   ├── mod.rs
│   ├── plugin.rs          // IrohGossipPlugin
│   ├── components.rs      // Gossip component (обёртка над iroh_gossip::Gossip)
│   ├── resources.rs       // GossipRuntime, каналы, настройки
│   ├── systems.rs         // join_topic, send_message, receive_messages
│   └── events.rs          // GossipEvent (MessageReceived, TopicJoined, ...)
├── blobs/                 // Подсистема blobs
│   ├── mod.rs
│   ├── plugin.rs          // IrohBlobsPlugin
│   ├── components.rs      // Blobs component (обёртка над iroh_blobs::Blobs)
│   ├── resources.rs       // BlobsRuntime, настройки
│   ├── systems.rs         // download_blob, upload_blob, хранение состояния
│   └── events.rs          // BlobsEvent (DownloadCompleted, UploadProgress, ...)
├── docs/                  // Подсистема docs
│   ├── mod.rs
│   ├── plugin.rs          // IrohDocsPlugin
│   ├── components.rs      // Docs component
│   ├── resources.rs       // DocsRuntime
│   ├── systems.rs         // create_document, sync_docs, ...
│   └── events.rs          // DocsEvent (SyncFinished, DocumentUpdated, ...)
└── utils/                 // Общие утилиты
    ├── mod.rs
    ├── channels.rs        // Обёртки для tokio каналов
    └── errors.rs          // Общий тип ошибок */
