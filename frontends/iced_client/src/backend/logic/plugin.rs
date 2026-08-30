use bevy_app::{App, Plugin};
use bevy_ecs::resource::Resource;
use bevy_iroh::GossipMessageReceived;


// Основной плагин логики нашего приложения
pub struct TextSyncPlugin;

impl Plugin for TextSyncPlugin {
    fn build(&self, app: &mut App) {
        // Инициализируем глобальное состояние
        app.init_resource::<AppStatus>();
        
    }
}

// Ресурс вместо Компонента. Хранит глобальное состояние для UI.
#[derive(Resource )]
pub struct AppStatus {
    pub node_id: String,        // Наш ID
    pub topic_id: String,       // Текущий топик (Hex или строка)
    pub shared_text: String,    // Текст в инпуте
    pub messages: Vec<String>,  // История сообщений
}

impl Default for AppStatus {
    fn default() -> Self {
        Self {
            node_id: "Starting...".into(),
            topic_id: "default-topic".into(),
            shared_text: String::new(),
            messages: Vec::new(),
        }
    }
}


