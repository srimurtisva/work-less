use crux_core::{Command, macros::effect, render::render};
use facet::Facet;
use loro::{ExportMode, LoroDoc};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct App;

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum Event {
    Append(String),
    Clear,
    Insert { index: usize, text: String },
    Import(Vec<u8>),
}

#[derive(Debug)]
pub struct Model {
    pub doc: LoroDoc,
}

impl Default for Model {
    fn default() -> Self {
        let doc = LoroDoc::new();
        let text = doc.get_text("content");
        text.insert(0, "Start Loro P2P").unwrap();
        doc.commit();
        Self { doc }
    }
}

#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct ViewModel {
    pub content: String,
    pub peer_id: String,
    pub version: u64,
    pub snapshot: Vec<u8>,
}

use crux_core::render::RenderOperation;

#[effect]
pub enum Effect {
    Render(RenderOperation),
}

impl crux_core::App for App {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
        let text = model.doc.get_text("content");
        match event {
            Event::Append(s) => {
                let len = text.len_unicode();
                text.insert(len, &s).unwrap();
                model.doc.commit();
            }
            Event::Clear => {
                let len = text.len_unicode();
                if len > 0 {
                    text.delete(0, len).unwrap();
                    model.doc.commit();
                }
            }
            Event::Insert { index, text: s } => {
                let len = text.len_unicode();
                let idx = index.min(len);
                text.insert(idx, &s).unwrap();
                model.doc.commit();
            }
            Event::Import(bytes) => {
                let _ = model.doc.import(&bytes).inspect_err(|error| {
                    tracing::error!("Failed to import data: {error}");
                });
            }
        }
        render()
    }

    fn view(&self, model: &Model) -> ViewModel {
        let text = model.doc.get_text("content");
        ViewModel {
            content: text.to_string(),
            peer_id: model.doc.peer_id().to_string(),
            version: model.doc.oplog_vv().len() as u64,
            snapshot: model.export_snapshot(),
        }
    }
}

impl Model {
    pub fn export_snapshot(&self) -> Vec<u8> {
        self.doc.export(ExportMode::Snapshot).unwrap_or_default()
    }
}

use tokio::sync::mpsc;

pub struct CoreWrapper {
    pub core: crux_core::Core<App>,
    pub p2p_sender: Option<mpsc::Sender<Vec<u8>>>,
}
