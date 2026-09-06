use crux_core::{Command, render::render};
use facet::Facet;
use loro::LoroDoc;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct App;

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum Event {
    Append(String),
    Clear,
    Insert { index: usize, text: String },
}

pub struct Model {
    pub doc: LoroDoc,
}

impl Default for Model {
    fn default() -> Self {
        let doc = LoroDoc::new();
        let text = doc.get_text("content");
        text.insert(0, "Привет от Loro CRDT!").unwrap();
        doc.commit();
        Self { doc }
    }
}

#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct ViewModel {
    pub content: String,
    pub peer_id: String,
    pub version: u64,
}

use crux_core::macros::effect;
use crux_core::render::RenderOperation;

#[effect(facet_typegen)]
#[derive(Debug)]
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
        }

        render()
    }

    fn view(&self, model: &Model) -> ViewModel {
        let text = model.doc.get_text("content");
        ViewModel {
            content: text.to_string(),
            peer_id: model.doc.peer_id().to_string(),
            version: model.doc.oplog_vv().len() as u64,
        }
    }
}
