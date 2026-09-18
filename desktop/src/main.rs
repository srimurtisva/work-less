use crux_core::{App, command::CommandOutput};
use futures::StreamExt;
use iced::{
    Alignment, Element, Length, Task,
    widget::{button, column, container, row, space, text, text_input},
};
use shared::Event;
use shared::ViewModel;
use shared::crdt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

pub fn main() -> iced::Result {
    init_logs();

    iced::application(DesktopApp::boot, DesktopApp::update, DesktopApp::view)
        .title("Loro CRDT - Desktop (Iced)")
        .window_size(iced::Size::new(800.0, 600.0))
        .run()
}

fn init_logs() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let directives = "desktop=trace,p2p::app=trace";
        // let directives = "warn";
        // let directives = "info";
        // let directives = "info,loro=warn,loro_internal=warn,iced_winit=warn,wgpu_hal=warn,wgpu_core=warn,n0_mainline=warn";
        EnvFilter::new(directives)
    });

    let _ = tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .try_init();
}

pub struct DesktopApp {
    app: shared::App,
    model: shared::Model,
    view: ViewModel,
    input_text: String,
}

impl DesktopApp {
    pub fn boot() -> (Self, Task<Message>) {
        let app = shared::App::default();
        let mut model = shared::Model::default();
        let start_command = app.update(Event::P2p(p2p::Event::Start), &mut model);
        let p2p_task = Task::stream(start_command.filter_map(|output| async move {
            match output {
                CommandOutput::Effect(shared::Effect::Render(_)) => Some(Message::Render),
                CommandOutput::Event(event) => Some(Message::ProcessEvent(event)),
            }
        }));
        let view = app.view(&model);

        let tasks = Task::batch(vec![p2p_task]);
        let desktop_app = DesktopApp {
            model,
            view,
            app,
            input_text: String::new(),
        };

        (desktop_app, tasks)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let loro_info = text(format!("Loro Peer: {}", self.view.crdt.peer_id)).size(14);

        let p2p_status = text(format!(
            "P2P: {} (Peers count: {})",
            self.view.p2p.status_text, self.view.p2p.peers_count
        ))
        .size(14);

        let content_display = container(
            text(if self.view.crdt.content.is_empty() {
                "Text is empty"
            } else {
                &self.view.crdt.content
            })
            .size(18),
        )
        .padding(15)
        .width(Length::Fill)
        .style(container::rounded_box);

        let event = Event::Crdt(crdt::Event::Append(self.input_text.clone()));

        let input = text_input("Enter text to sync...", &self.input_text)
            .on_input(Message::InputChanged)
            .on_submit(Message::ProcessEvent(event))
            .padding(10);

        let add_btn =
            button("Add to CRDT and send").on_press_maybe(if self.input_text.is_empty() {
                None
            } else {
                Some(Message::ProcessEvent(Event::Crdt(crdt::Event::Append(
                    self.input_text.clone(),
                ))))
            });

        let clear_btn =
            button("Clear").on_press(Message::ProcessEvent(Event::Crdt(crdt::Event::Clear)));

        let paste_btn = button("Paste key").on_press(Message::Paste);
        let copy_btn = button("Copy key").on_press(Message::Copy);

        let copy_paste = row![copy_btn, paste_btn].spacing(12);

        let controls = row![input, add_btn, clear_btn].spacing(10);

        let layout = column![
            text("Loro + Iroh (Crux core)").size(24),
            row![loro_info, space::horizontal(), p2p_status],
            copy_paste,
            space::vertical(),
            content_display,
            space::vertical(),
            controls,
        ]
        .spacing(10)
        .max_width(600)
        .align_x(Alignment::Center);

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(20)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        // tracing::trace!("Update Message is {:?}", message);
        match message {
            Message::InputChanged(value) => {
                self.input_text = value;
                Task::none()
            }
            Message::ProcessEvent(event) => {
                if matches!(&event, Event::Crdt(crdt::Event::Append(_))) {
                    self.input_text.clear();
                }
                let _ = self.app.update(event, &mut self.model);
                self.view = self.app.view(&self.model);
                Task::none()
            }
            Message::Render => {
                self.view = self.app.view(&self.model);
                Task::none()
            }
            Message::Copy => iced::clipboard::write(self.view.p2p.ticket.clone()),
            Message::Paste => iced::clipboard::read().map(|value| {
                if let Some(ticket_string) = value {
                    Message::ProcessEvent(Event::P2p(p2p::app::Event::Connect(ticket_string)))
                } else {
                    Message::Render
                }
            }),
        }
    }

    fn handle_effects(&mut self, requests: Vec<shared::Effect>) -> Task<Message> {
        let mut task = Task::none();
        for e in requests {
            match e {
                shared::Effect::Render(_) => {
                    task = Task::done(Message::Render);
                }
            }
        }
        task
    }
}
#[derive(Debug, Clone)]
pub enum Message {
    InputChanged(String),
    ProcessEvent(Event),
    Render,
    Copy,
    Paste,
}
