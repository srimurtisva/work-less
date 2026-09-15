use super::*;
use iced::futures::channel::mpsc::UnboundedSender;
use tracing::{Subscriber, field::Visit};
struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}").trim_matches('"').to_string();
        }
    }
}

pub struct IcedLogLayer {
    sender: UnboundedSender<Message>,
}

impl IcedLogLayer {
    pub fn new(sender: UnboundedSender<Message>) -> Self {
        Self { sender }
    }
}

use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
impl<S: Subscriber> Layer<S> for IcedLogLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = MessageVisitor {
            message: String::new(),
        };
        event.record(&mut visitor);

        let entry = LogEntry {
            level: *event.metadata().level(),
            target: event.metadata().target().into(),
            message: visitor.message,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        };

        let message = Message::LogReceived(entry);
        // Отправляем в канал iced. Если никто не слушает — просто игнорируем.
        let _ = self.sender.unbounded_send(message);
    }
}


