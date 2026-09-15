use crux_core::Command;
use facet::Facet;
use serde::{Deserialize, Serialize};

pub mod crdt;

#[derive(Default)]
pub struct App {
    crdt: crdt::App,
    p2p: p2p::app::App,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum Event {
    Crdt(crdt::Event),
    P2p(p2p::app::Event),
}

#[derive(Default, Debug)]
pub struct Model {
    pub crdt: crdt::Model,
    pub p2p: p2p::app::Model,
}

#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct ViewModel {
    pub crdt: crdt::ViewModel,
    pub p2p: p2p::app::ViewModel,
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

    fn update(&self, app_event: Event, model: &mut Model) -> Command<Effect, Event> {
        match app_event {
            Event::Crdt(event) => {
                let crdt_cmd = self
                    .crdt
                    .update(event, &mut model.crdt)
                    .map_event(Event::Crdt)
                    .map_effect(|effect| match effect {
                        crdt::Effect::Render(render) => Effect::Render(render),
                    });

                let snapshot = model.crdt.export_snapshot();

                let p2p_cmd = self
                    .p2p
                    .update(p2p::app::Event::Broadcast(snapshot), &mut model.p2p)
                    .map_event(Event::P2p)
                    .map_effect(|e| match e {
                        p2p::app::Effect::Render(r) => Effect::Render(r),
                    });

                Command::all(vec![crdt_cmd, p2p_cmd])
            }

            Event::P2p(p2p::app::Event::DataReceived(data)) => {
                let p2p_cmd = self
                    .p2p
                    .update(p2p::app::Event::DataReceived(data.clone()), &mut model.p2p)
                    .map_event(Event::P2p)
                    .map_effect(|e| match e {
                        p2p::app::Effect::Render(r) => Effect::Render(r),
                    });

                let crdt_cmd = self
                    .crdt
                    .update(crdt::Event::Import(data), &mut model.crdt)
                    .map_event(Event::Crdt)
                    .map_effect(|e| match e {
                        crdt::Effect::Render(r) => Effect::Render(r),
                    });

                Command::all(vec![p2p_cmd, crdt_cmd])
            }

            Event::P2p(p2p::app::Event::PeerJoined(peer)) => {
                let p2p_cmd = self
                    .p2p
                    .update(p2p::app::Event::PeerJoined(peer), &mut model.p2p)
                    .map_event(Event::P2p)
                    .map_effect(|e| match e {
                        p2p::app::Effect::Render(r) => Effect::Render(r),
                    });

                let snapshot = model.crdt.export_snapshot();
                let broadcast_cmd = self
                    .p2p
                    .update(p2p::app::Event::Broadcast(snapshot), &mut model.p2p)
                    .map_event(Event::P2p)
                    .map_effect(|e| match e {
                        p2p::app::Effect::Render(r) => Effect::Render(r),
                    });

                Command::all(vec![p2p_cmd, broadcast_cmd])
            }

            Event::P2p(event) => {
                let mut update = self.p2p.update(event, &mut model.p2p);

                for p2p_event in update.events() {
                    if let p2p::app::Event::DataReceived(data) = p2p_event {
                        let c = self.crdt.update(crdt::Event::Import(data), &mut model.crdt);
                        let c = c.map_event(Event::Crdt).map_effect(|effect| match effect {
                            crdt::Effect::Render(render) => Effect::Render(render),
                        });
                        return c;
                    }
                }

                update
                    .map_event(Event::P2p)
                    .map_effect(|effect| match effect {
                        p2p::app::Effect::Render(render) => Effect::Render(render),
                    })
            }
        }
    }

    fn view(&self, model: &Model) -> ViewModel {
        let crdt = self.crdt.view(&model.crdt);
        let p2p = self.p2p.view(&model.p2p);
        ViewModel { crdt, p2p }
    }
}
