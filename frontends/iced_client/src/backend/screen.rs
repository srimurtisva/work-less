use crate::backend::logic::plugin::AppStatus;
use bevy_app::App;
use derive_more::Display;
use iced::Element;
use iced::Task;

pub struct State {}

impl State {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view<'a>(&'a self, app: &'a App) -> Element<'a, Message> {
        use iced::widget::{button, column, container, row, scrollable, space, text, text_input};
        use iced::{Alignment, Color, Length};

        let status = app.world().resource::<AppStatus>();

        // 1. Шапка с информацией о ноде
        let header = column![
            row![
                text(format!("My Node ID: {}", status.node_id))
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
                button("Copy").on_press(Message::CopyNodeId).padding(5),
            ],
            text(format!("Topic: {}", status.topic_id)).size(18),
        ]
        .spacing(5);

        // 2. Список сообщений (Лог)
        let message_log = scrollable(
            column(status.messages.iter().map(|m| text(m).into()))
                .spacing(10)
                .padding(10),
        )
        .height(Length::Fill);

        // 3. Поле ввода
        let input_field = row![
            text_input("Type a message and press Enter...", &status.shared_text)
                .on_input(Message::TextChanged)
                .on_submit(Message::SendMessage) // Новое событие для отправки
                .padding(10),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        // Собираем всё вместе
        container(
            column![
                header,
                space().height(10),
                container(message_log)
                    .style(container::dark) // Или любая другая стилизация
                    .height(Length::Fill)
                    .padding(5),
                input_field,
            ]
            .spacing(10)
            .padding(20),
        )
        .into()
    }

    pub fn update(&mut self, app: &mut App, backend_message: Message) -> Task<Message> {
        tracing::info!("Message: {}", backend_message);
        match backend_message {
            Message::TextChanged(value) => {
                update_local_state(app, value.clone());
                send_event_to_network(app, value);
                app.update();
                Task::none()
            }
            Message::SendMessage => {
                let mut status = app.world_mut().resource_mut::<AppStatus>();
                let text_to_send = std::mem::take(&mut status.shared_text); // Очищаем поле ввода

                if !text_to_send.is_empty() {
                    // Добавляем себе в лог (опционально, iroh-gossip может прислать его обратно)
                    status.messages.push(format!("Me: {}", text_to_send));

                    // Триггерим событие для вашего IrohGossipPlugin
                    // Здесь мы передаем текст в байтах
                    app.world_mut().trigger(bevy_iroh::SendGossipMessage {
                        // topic: your_topic_id, // Его нужно хранить в ресурсе или генерировать
                        content: text_to_send,
                    });

                    app.update(); // Обновляем ECS
                }
                Task::none()
            }
            Message::CopyNodeId => {
                let status = app.world().resource::<AppStatus>();
                iced::clipboard::write(status.node_id.clone())
            }
        }
    }
}

fn send_event_to_network(app: &mut App, value: String) {
    app.world_mut()
        .trigger(bevy_iroh::SendGossipMessage { content: value });
}

fn update_local_state(app: &mut App, value: String) {
    app.world_mut().resource_mut::<AppStatus>().shared_text = value;
}

#[derive(Debug, Clone, Display)]
pub enum Message {
    TextChanged(String),
    SendMessage,
    CopyNodeId,
}
