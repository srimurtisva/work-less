// mod screen;

use iced::Element;
use iced::Task;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::unbounded_channel;

// Root for backend-related functionality
pub struct State {
    // app: shared::App,
    // screen: screen::State,
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let (_wake_tx, wake_rx) = unbounded_channel::<String>();

        // let app = shared::App::default();
        // let screen = screen::State::new();

        let backend = Self { 
            // app
             // , screen
        };

        let task = signal_listener_task(wake_rx);
        (backend, task)
    }

    pub fn view(&self) -> Element<'_, Message> {
        iced::widget::space().into()
        // self.screen.view(&self.app).map(Message::Screen)
    }

    pub fn update(&mut self, backend_message: Message) -> Task<Message> {
        match backend_message {
            Message::BevySignal(_string) => {
                // self.app.update();
                Task::none()
            }
            // Message::Screen(message) => {
                // self.screen.update(&mut self.app, message).map(Message::Screen)
            //     Task::none()
            // },
        }
    }
}

fn signal_listener_task(mut events_rx: UnboundedReceiver<String>) -> Task<Message> {
    Task::stream(iced::stream::channel(
        100,
        |mut output: Sender<String>| async move {
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
    BevySignal(String),
    // Screen(screen::Message),
}
