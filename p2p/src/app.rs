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
        peers_count: u32,
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
                    let peers_count = peers_count.saturating_add(1);
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
                    let peers_count = peers_count.saturating_sub(1);
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
        let (ticket, peers_count) = if let Status::Connected {
            ticket: node_id,
            peers_count,
        } = model.status.clone()
        {
            (node_id, peers_count)
        } else {
            ("".to_string(), 0)
        };
        let is_connected = matches!(model.status, Status::Connected { .. });
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

#[cfg(test)]
mod tests {
    use super::*;
    use crux_core::App;

    #[test]
    fn default_model_is_uninitialized() {
        let model = Model::default();

        assert_eq!(model.status, Status::Uninitialized);
        assert!(model.connected_peers.is_empty());
    }

    #[test]
    fn view_for_uninitialized_model() {
        let app = super::App::default();
        let model = Model::default();

        let view = app.view(&model);

        assert_eq!(view.status_text, "Uninitialized");
        assert!(!view.is_connected);
        assert_eq!(view.peers_count, 0);
        assert_eq!(view.ticket, "");
    }

    #[test]
    fn view_for_connected_model() {
        let app = super::App::default();

        let model = Model {
            status: Status::Connected {
                ticket: "ticket-123".to_string(),
                peers_count: 2,
            },
            connected_peers: vec!["peer-1".to_string(), "peer-2".to_string()],
        };

        let view = app.view(&model);

        assert_eq!(
            view.status_text,
            "Connected { ticket: \"ticket-123\", peers_count: 2 }"
        );
        assert!(view.is_connected);
        assert_eq!(view.peers_count, 2);
        assert_eq!(view.ticket, "ticket-123");
    }

    #[test]
    fn view_for_failed_model() {
        let app = super::App::default();

        let model = Model {
            status: Status::Failed("connection failed".to_string()),
            connected_peers: vec![],
        };

        let view = app.view(&model);

        assert_eq!(view.status_text, "Failed(\"connection failed\")");
        assert!(!view.is_connected);
        assert_eq!(view.peers_count, 0);
        assert_eq!(view.ticket, "");
    }

    #[test]
    fn start_changes_status_to_connecting() {
        let app = super::App::default();
        let mut model = Model::default();

        let _command = app.update(Event::Start, &mut model);

        assert_eq!(model.status, Status::Connecting);
    }

    #[test]
    fn ready_changes_status_to_connected() {
        let app = super::App::default();
        let mut model = Model::default();

        let _command = app.update(Event::Ready("ticket-123".to_string()), &mut model);

        assert_eq!(
            model.status,
            Status::Connected {
                ticket: "ticket-123".to_string(),
                peers_count: 0,
            }
        );
    }

    #[test]
    fn peer_joined_is_added_to_connected_peers() {
        let app = super::App::default();

        let mut model = Model {
            status: Status::Connected {
                ticket: "ticket-123".to_string(),
                peers_count: 0,
            },
            connected_peers: vec![],
        };

        let _command = app.update(Event::PeerJoined("peer-1".to_string()), &mut model);

        assert_eq!(model.connected_peers, vec!["peer-1".to_string()]);

        assert_eq!(
            model.status,
            Status::Connected {
                ticket: "ticket-123".to_string(),
                peers_count: 1,
            }
        );
    }

    #[test]
    fn peer_left_is_removed_from_connected_peers() {
        let app = super::App::default();

        let mut model = Model {
            status: Status::Connected {
                ticket: "ticket-123".to_string(),
                peers_count: 2,
            },
            connected_peers: vec!["peer-1".to_string(), "peer-2".to_string()],
        };

        let _command = app.update(Event::PeerLeft("peer-1".to_string()), &mut model);

        assert_eq!(model.connected_peers, vec!["peer-2".to_string()]);

        assert_eq!(
            model.status,
            Status::Connected {
                ticket: "ticket-123".to_string(),
                peers_count: 1,
            }
        );
    }

    #[test]
    fn peer_left_unknown_peer_does_not_change_peer_list() {
        let app = super::App::default();

        let mut model = Model {
            status: Status::Connected {
                ticket: "ticket-123".to_string(),
                peers_count: 1,
            },
            connected_peers: vec!["peer-1".to_string()],
        };

        let _command = app.update(Event::PeerLeft("unknown-peer".to_string()), &mut model);

        assert_eq!(model.connected_peers, vec!["peer-1".to_string()]);

        assert_eq!(
            model.status,
            Status::Connected {
                ticket: "ticket-123".to_string(),
                peers_count: 0,
            }
        );
    }

    #[test]
    fn failed_event_changes_status() {
        let app = super::App::default();
        let mut model = Model::default();

        let _command = app.update(
            Event::Failed("something went wrong".to_string()),
            &mut model,
        );

        assert_eq!(
            model.status,
            Status::Failed("something went wrong".to_string())
        );
    }

    #[tokio::test]
    async fn connect_sends_join_command() {
        let app = super::App::default();

        let receiver = app
            .outgoing_rx
            .lock()
            .unwrap()
            .take()
            .expect("receiver should exist");

        let mut model = Model::default();

        let _command = app.update(Event::Connect("ticket-123".to_string()), &mut model);

        let mut receiver = receiver;

        let command = receiver.recv().await.expect("Join command should be sent");

        match command {
            OutgoingCommand::Join(ticket) => {
                assert_eq!(ticket, "ticket-123");
            }
            other => panic!("expected Join command, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn broadcast_sends_broadcast_command() {
        let app = super::App::default();

        let receiver = app
            .outgoing_rx
            .lock()
            .unwrap()
            .take()
            .expect("receiver should exist");

        let mut model = Model::default();

        let data = vec![1, 2, 3, 4];

        let _command = app.update(Event::Broadcast(data.clone()), &mut model);

        let mut receiver = receiver;

        let command = receiver
            .recv()
            .await
            .expect("Broadcast command should be sent");

        match command {
            OutgoingCommand::Broadcast(received) => {
                assert_eq!(received, data);
            }
            other => panic!("expected Broadcast command, got {other:?}"),
        }
    }

    #[test]
    fn start_can_only_take_receiver_once() {
        let app = super::App::default();

        let mut first_model = Model::default();
        let mut second_model = Model::default();

        let _first_command = app.update(Event::Start, &mut first_model);
        let _second_command = app.update(Event::Start, &mut second_model);

        assert_eq!(first_model.status, Status::Connecting);
        assert_eq!(second_model.status, Status::Connecting);
    }

    #[test]
    fn peer_left_when_no_peers_does_not_underflow() {
        let app = super::App::default();

        let mut model = Model {
            status: Status::Connected {
                ticket: "ticket-123".to_string(),
                peers_count: 0,
            },
            connected_peers: vec![],
        };

        let _command = app.update(Event::PeerLeft("unknown-peer".to_string()), &mut model);

        assert_eq!(
            model.status,
            Status::Connected {
                ticket: "ticket-123".to_string(),
                peers_count: 0,
            }
        );
    }
}
