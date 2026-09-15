use std::sync::{Arc, Mutex};

use crux_core::Command;
use facet::Facet;
use serde::{Deserialize, Serialize};

mod backend;
pub const TOPIC_NAME: &[u8; 32] = b"loro-crdt-sync-sample-room-12345";

pub struct App {
    outgoing_tx: tokio::sync::mpsc::Sender<OutgoingCommand>,
    outgoing_rx: Arc<Mutex<Option<tokio::sync::mpsc::Receiver<OutgoingCommand>>>>,
}

impl Default for App {
    fn default() -> Self {
        let (outgoing_tx, outgoing_rx) = tokio::sync::mpsc::channel(100);
        let outgoing_rx = Arc::new(Mutex::new(Some(outgoing_rx)));
        Self {
            outgoing_tx,
            outgoing_rx,
        }
    }
}

impl App {
    fn broadcast(&self, data: Vec<u8>) {
        let tx = self.outgoing_tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(OutgoingCommand::Broadcast(data)).await;
        });
    }
    fn connect(&self, id: String) {
        let tx = self.outgoing_tx.clone();
        tracing::trace!("Sending connection info to the background thread.");
        tokio::spawn(async move {
            let _ = tx.send(OutgoingCommand::Join(id)).await;
        });
    }
}
/// `Start` kicks the app out of `Uninitialized`. The remaining variants carry
#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[repr(C)]
pub enum Event {
    /// Sent by the shell once, at launch. Triggers initialisation.
    Start,

    Connect(String),

    // Internal events resolving the parallel initialisation fetches.
    // #[serde(skip)]
    // #[facet(skip)]
    Ready(String),

    // #[serde(skip)]
    // #[facet(skip)]
    PeerJoined(String),

    // #[serde(skip)]
    // #[facet(skip)]
    PeerLeft(String),

    DataReceived(Vec<u8>),
    Broadcast(Vec<u8>),
    Failed(String),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Status {
    #[default]
    Uninitialized,
    Connecting,
    Connected {
        ticket: String,
        peers_count: usize,
    },
    Failed(String),
}

#[derive(Default, Debug)]
pub struct Model {
    pub status: Status,
    pub connected_peers: Vec<String>,
}

#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct ViewModel {
    pub status_text: String,
    pub is_connected: bool,
    pub peers_count: u32,
    pub ticket: String,
}

use crux_core::macros::effect;
use crux_core::render::{RenderOperation, render};
#[effect]
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
        tracing::trace!("Event: {:?}", event);

        match event {
            Event::Start => {
                model.status = Status::Connecting;
                tracing::trace!("{:?}", model.status);

                if let Some(rx) = self.outgoing_rx.lock().unwrap().take() {
                    Command::new(move |ctx| backend::P2pWorker::run(ctx, rx))
                } else {
                    tracing::warn!("P2pWorker is aalready running");
                    Command::done()
                }
            }

            Event::Ready(ticket) => {
                model.status = Status::Connected {
                    ticket,
                    peers_count: 0,
                };
                render()
            }
            Event::PeerJoined(public_key) => {
                model.connected_peers.push(public_key);
                if let Status::Connected {
                    ticket,
                    peers_count,
                } = model.status.clone()
                {
                    let peers_count = peers_count + 1;
                    model.status = Status::Connected {
                        ticket,
                        peers_count,
                    };
                }
                render()
            }
            Event::PeerLeft(public_key) => {
                model.connected_peers.retain(|p| p != &public_key);
                if let Status::Connected {
                    ticket,
                    peers_count,
                } = model.status.clone()
                {
                    let peers_count = peers_count - 1;
                    model.status = Status::Connected {
                        ticket,
                        peers_count,
                    };
                }
                render()
            }

            Event::DataReceived(_) => Command::done(),

            Event::Failed(msg) => {
                model.status = Status::Failed(msg);
                render()
            }
            Event::Broadcast(items) => {
                self.broadcast(items);
                Command::done()
            }
            Event::Connect(ticket_string) => {
                self.connect(ticket_string);
                Command::done()
            }
        }
    }

    fn view(&self, model: &Model) -> ViewModel {
        let status_text = format!("{:?}", model.status);
        let ticket = if let Status::Connected {
            ticket: node_id, ..
        } = model.status.clone()
        {
            node_id
        } else {
            "".to_string()
        };
        let is_connected = matches!(model.status, Status::Connected { .. });
        let peers_count: u32 = match model.connected_peers.len().try_into() {
            Ok(val) => val,
            Err(error) => {
                tracing::error!("Failed to convert number: {error}");
                0
            }
        };
        ViewModel {
            status_text,
            is_connected,
            peers_count,
            ticket,
        }
    }
}

#[derive(Debug)]
pub enum OutgoingCommand {
    Broadcast(Vec<u8>),
    Join(String),
}

