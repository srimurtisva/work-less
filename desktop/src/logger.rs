// use crate::update::Update;
use backend::IcedLogLayer;
use futures::{SinkExt, StreamExt};
use iced::Task;
use iced::futures::channel::mpsc::{Sender, unbounded};
use iced::{Element, Length};
use std::collections::{HashMap, VecDeque};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::reload;
use tracing_subscriber::util::SubscriberInitExt; // <-- Добавили reload

mod backend;

// Создаем удобный алиас для типа хэндла перезапуска фильтра
type FilterHandle = reload::Handle<EnvFilter, tracing_subscriber::Registry>;

#[derive(Debug)]
pub struct State {
    entries: VecDeque<LogEntry>,
    max_entries: usize,
    is_expanded: bool,
    known_targets: HashMap<String, LogTarget>,
    // Храним хэндл для динамического изменения фильтра
    filter_handle: Option<FilterHandle>,
    split_position: f32,
}

impl State {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: 50,
            is_expanded: false,
            known_targets: HashMap::new(),
            filter_handle: None,
            split_position: 0.7,
        }
    }

    pub fn boot(&mut self) -> Task<Message> {
        let (tx, mut rx) = unbounded();
        let gui_layer = IcedLogLayer::new(tx);

        // 1. Создаем начальный фильтр (разрешаем всё по умолчанию или пустой)
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"));

        // 2. Оборачиваем фильтр в reload-слой
        let (filter_layer, filter_handle) = reload::Layer::new(filter);
        self.filter_handle = Some(filter_handle);

        // 3. Регистрируем слои. Теперь filter_layer управляет тем, что дойдет до gui_layer
        let _ = tracing_subscriber::registry()
            .with(filter_layer)
            .with(gui_layer)
            .try_init();

        Task::stream(iced::stream::channel(
            10,
            |mut output: Sender<Message>| async move {
                while let Some(msg) = rx.next().await {
                    let _ = output.send(msg).await;
                }
            },
        ))
    }

    pub fn update(&mut self, log_message: Message) -> Task<Message> {
        match log_message {
            Message::TogglePanel => {
                self.is_expanded = !self.is_expanded;
                Task::none()
            }

            Message::LogReceived(entry) => {
                self.known_targets
                    .entry(entry.target.clone())
                    .or_insert(LogTarget::from_string(entry.target.clone()));

                self.entries.push_back(entry);
                if self.entries.len() > self.max_entries {
                    self.entries.pop_front();
                }

                Task::none()
            }

            Message::Target(key, target_message) => {
                if let Some(target) = self.known_targets.get_mut(&key) {
                    // Если состояние видимости изменилось, обновляем глобальный EnvFilter
                    if target.update(target_message) {
                        self.apply_filters_to_tracing();
                    }
                }
                Task::none()
            }
            Message::SplitResized(position) => {
                self.split_position = position;
                Task::none()
            }
        }
    }

    // Вспомогательный метод для сборки строки фильтра и отправки ее в tracing
    fn apply_filters_to_tracing(&self) {
        if let Some(handle) = &self.filter_handle {
            // Собираем директивы вида: "target1=trace,target2=off,target3=trace"
            // Если visible == true, ставим "trace" (или любой другой максимальный уровень), иначе "off"
            let mut filter_string = String::new();

            for (target_name, target_state) in &self.known_targets {
                if !filter_string.is_empty() {
                    filter_string.push(',');
                }
                let level = if target_state.visible { "trace" } else { "off" };
                filter_string.push_str(&format!("{}={}", target_name, level));
            }

            // Если список пустой, разрешаем все
            if filter_string.is_empty() {
                filter_string = "trace".to_string();
            }

            // Создаем новый EnvFilter и закидываем его в работающий рантайм
            if let Ok(new_filter) = EnvFilter::try_new(&filter_string)
                && let Err(e) = handle.reload(new_filter)
            {
                println!("Не удалось обновить EnvFilter: {}", e);
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        use iced::widget::{button, column, container, text};
        use iced_split::vertical_split;

        let header = button(text(if self.is_expanded {
            "▼ Log"
        } else {
            "▶ Log"
        }))
        .on_press(Message::TogglePanel);

        let mut content = column![header].spacing(10).padding(10);

        if self.is_expanded {
            let split = vertical_split(
                self.log_scroll(),
                self.sidebar(),
                self.split_position,
                Message::SplitResized,
            );

            content = content.push(split);
        }

        container(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    fn sidebar(&self) -> iced::widget::Scrollable<'_, Message> {
        use iced::widget::{column, scrollable};
        let targets = column(self.known_targets.iter().map(|(key, target)| {
            target
                .view()
                .map({ move |msg| Message::Target(key.clone(), msg) })
        }))
        .spacing(2);

        scrollable(targets)
    }

    fn log_scroll(&self) -> iced::widget::Scrollable<'_, Message> {
        use iced::widget::{column, row, scrollable, text};

        // Теперь мы итерируемся напрямую по self.entries, они уже отфильтрованы!
        let log_content = column(self.entries.iter().map(|log| {
            let name = self
                .known_targets
                .get(&log.target)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| log.target.clone());
            row![
                text(log.timestamp.clone()).size(10),
                text(name).size(10),
                text(log.message.clone()).size(12),
            ]
            .spacing(10)
            .into()
        }))
        .spacing(5);

        scrollable(log_content).anchor_bottom().width(iced::Fill)
    }

    pub fn loaded(&mut self, saved: Saved) {
        self.is_expanded = saved.is_expanded;
        self.known_targets = saved.known_targets;
        // После загрузки старых таргетов применяем фильтр, если boot уже отработал
        self.apply_filters_to_tracing();
    }

    pub fn save(&self) -> Saved {
        Saved {
            is_expanded: self.is_expanded,
            known_targets: self.known_targets.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePanel,
    LogReceived(LogEntry),
    Target(String, TargetMessage),
    SplitResized(f32),
}

#[derive(Default)]
pub enum Effect {
    #[default]
    None,
    Dirty,
}

use tracing::Level;
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: Level,
    pub message: String,
    pub target: String,
    pub timestamp: String,
}

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Saved {
    is_expanded: bool,
    known_targets: HashMap<String, LogTarget>,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct LogTarget {
    name: String,
    visible: bool,
}

impl LogTarget {
    pub fn from_string(key: String) -> Self {
        Self {
            name: key,
            visible: true,
        }
    }
    pub fn view(&self) -> Element<'_, TargetMessage> {
        use iced::widget::{checkbox, row, text_input};
        let cb = checkbox(self.visible).on_toggle(TargetMessage::ToggleVisible);
        let name = text_input(&self.name, &self.name)
            .on_input(TargetMessage::NewName)
            .size(12.0);
        row![cb, name].spacing(4).into()
    }
    pub fn update(&mut self, message: TargetMessage) -> bool {
        match message {
            TargetMessage::ToggleVisible(is_visible) => {
                self.visible = is_visible;
                true // Возвращаем true, чтобы State понял, что нужно обновить EnvFilter
            }
            TargetMessage::NewName(name) => {
                self.name = name;
                false
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum TargetMessage {
    ToggleVisible(bool),
    NewName(String),
}
