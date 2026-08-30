pub mod logic;
mod screen;

use logic::*;
use bevy_app::App;
use iced::Element;
use iced::Task;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::unbounded_channel;

// Root for backend-related functionality
pub struct State {
    app: App,
    screen: screen::State,
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let (wake_tx, wake_rx) = unbounded_channel::<()>();

        let app = prepare_app(wake_tx);
        let screen = screen::State::new();

        let backend = Self { app , screen};

        let task = signal_listener_task(wake_rx);
        (backend, task)
    }

    pub fn view(&self) -> Element<'_, Message> {
        self.screen.view(&self.app).map(Message::Screen)
    }

    pub fn update(&mut self, backend_message: Message) -> Task<Message> {
        match backend_message {
            Message::BevySignal(()) => {
                self.app.update();
                Task::none()
            }
            Message::Screen(message) => {
                self.screen.update(&mut self.app, message).map(Message::Screen)
            },
        }
    }
}

fn signal_listener_task(mut events_rx: UnboundedReceiver<()>) -> Task<Message> {
    Task::stream(iced::stream::channel(
        100,
        |mut output: Sender<()>| async move {
            while let Some(event) = events_rx.recv().await {
                println!("   +++++ ===== ----- >>>>> Event recieved on iced");
                let _ = output.send(event).await;
            }
        },
    ))
    .map(Message::BevySignal)
}

#[derive(Debug, Clone)]
pub enum Message {
    BevySignal(()),
    Screen(screen::Message),
}
