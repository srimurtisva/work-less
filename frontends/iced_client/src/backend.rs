use bevy_app::App;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::system::Single;
use bevy_iroh::IrohPlugin;
use derive_more::Deref;
use derive_more::DerefMut;
use derive_more::Display;
use iced::Element;
use iced::Task;
use iced::futures::SinkExt;
use iced::futures::channel::mpsc::Sender;
use iced::futures::channel::mpsc::UnboundedReceiver;
use iced::futures::channel::mpsc::{UnboundedSender, unbounded};

// Root for backend-related functionality
pub struct State {
    app: App,
    message_entity: Entity,
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let (wake_tx, wake_rx) = unbounded::<()>();

        let mut app = prepare_app(wake_tx);

        let message_entity = app.world_mut().spawn(AppStatus::default()).id();

        let backend = Self {
            app,
            message_entity,
        };

        let task = signal_listener_task(wake_rx);
        (backend, task)
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::widget::{button, column, row, text};
        let message = self.get_message();
        row![
            column![
                text(message),
                button("Start iroh node").on_press(Message::StartIrohNode)
            ]
            .spacing(10),
        ]
        .into()
    }

    fn get_message(&self) -> &str {
        (self
            .app
            .world()
            .get::<AppStatus>(self.message_entity)
            .map(|m| m.message.as_str())
            .unwrap_or_default()) as _
    }

    pub fn update(&mut self, backend_message: Message) -> Task<Message> {
        // tracing::info!("Message: {}", backend_message);
        match backend_message {
            Message::BevySignal(()) => {
                self.app.update();

                Task::none()
            }
            Message::StartIrohNode => {
                println!("Command 'StartIrohNode' sent");

                self.app.update();
                Task::none()
            }
        }
    }
}

fn prepare_app(wake_tx: UnboundedSender<()>) -> App {
    let mut app = App::new();
    app.world_mut().spawn(IcedWaker(wake_tx));
    app.world_mut().spawn(AppStatus::default());

    app.add_plugins(IrohPlugin);
    app.add_observer(send_events_to_iced);
    app.add_observer(on_iroh_started);
    app
}

fn signal_listener_task(mut events_rx: UnboundedReceiver<()>) -> Task<Message> {
    Task::stream(iced::stream::channel(
        100,
        |mut output: Sender<()>| async move {
            while let Ok(event) = events_rx.recv().await {
                println!("Event recieved on iced");
                let _ = output.send(event).await;
            }
        },
    ))
    .map(Message::BevySignal)
}

#[derive(Debug, Clone)]
pub enum Message {
    BevySignal(()),
    StartIrohNode,
}

#[derive(Component, Deref, DerefMut)]
pub struct IcedWaker(pub UnboundedSender<()>);

pub fn send_events_to_iced(event: On<StartIrohCommand>, event_sender: Single<&IcedWaker>) {
    tracing::info!("Sending to iced on event: {}", event.event());

    let _ = event_sender.unbounded_send(());
}

use bevy_ecs::prelude::Event;
#[derive(Debug, Display, Clone, Event)]
pub struct StartIrohCommand;

#[derive(Component, Default)]
pub struct AppStatus {
    pub message: String,
}

pub fn on_iroh_started(
    event: On<bevy_iroh::NodeStarted>,
    mut status: Single<&mut AppStatus>,
    waker: Single<&IcedWaker>,
) {
    tracing::info!("Iroh node started event received in Bevy!");
    println!("on_iroh_started");

    // Обновляем статус в мире Bevy (например, берем node_id из события)
    // status.message = format!("Node started: {}",event().node_id);
    status.message = "Iroh node is fully running!/n".to_string();
    status.message.push_str(&event.event().to_string());

    // Будим Iced
    let _ = waker.unbounded_send(());
}
