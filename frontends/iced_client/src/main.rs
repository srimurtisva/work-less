mod backend;
use tracing_subscriber::EnvFilter;

pub fn main() -> iced::Result {
    println!("Started");
    
    let _ = tracing_subscriber::fmt()
    .compact()
    .with_env_filter(
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"))
    )
    .try_init();

    iced::application(boot, update, view)
        .window_size(iced::Size::new(800.0, 600.0))
        .subscription(subscriptions)
        .run()
}

use iced::Element;
pub fn view(state: &State) -> Element<'_, Message> {
    state
        .backend
        .view()
        .map(Message::Backend)
}

use iced::Task;
pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Backend(backend_message) => {
            state.backend.update(backend_message).map(Message::Backend)
        }
    }
}

#[derive(Debug)]
// General iced message between view and update
pub enum Message {
    Backend(backend::Message),
}

// General GUI state for iced model
pub struct State {
    backend: backend::State,
}

fn boot() -> (State, Task<Message>) {
    let (backend, task) = backend::State::new();
    let state = State { backend };

    let tasks = vec![task.map(Message::Backend)];

    (state, Task::batch(tasks))
}

use iced::Subscription;
fn subscriptions(_state: &State) -> Subscription<Message> {
    Subscription::batch(vec![])
}
