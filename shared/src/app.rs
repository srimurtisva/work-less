use crux_core::Command;
use facet::Facet;
use serde::{Deserialize, Serialize};

pub mod crdt;

#[derive(Default)]
pub struct App {
    crdt: crdt::App,
    p2p: p2p::App,
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum Event {
    Crdt(crdt::Event),
    P2p(p2p::Event),
}

#[derive(Default, Debug)]
pub struct Model {
    pub crdt: crdt::Model,
    pub p2p: p2p::Model,
}

#[derive(Facet, Serialize, Deserialize, Clone, Default)]
pub struct ViewModel {
    pub crdt: crdt::ViewModel,
    pub p2p: p2p::ViewModel,
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
                    .map_effect(Effect::from);

                let snapshot = model.crdt.export_snapshot();

                let p2p_cmd = self
                    .p2p
                    .update(p2p::Event::Broadcast(snapshot), &mut model.p2p)
                    .map_event(Event::P2p)
                    .map_effect(Effect::from);

                Command::all(vec![crdt_cmd, p2p_cmd])
            }

            Event::P2p(p2p::Event::DataReceived(data)) => {
                let p2p_cmd = self
                    .p2p
                    .update(p2p::Event::DataReceived(data.clone()), &mut model.p2p)
                    .map_event(Event::P2p)
                    .map_effect(Effect::from);

                let crdt_cmd = self
                    .crdt
                    .update(crdt::Event::Import(data), &mut model.crdt)
                    .map_event(Event::Crdt)
                    .map_effect(Effect::from);

                Command::all(vec![p2p_cmd, crdt_cmd])
            }

            Event::P2p(p2p::Event::PeerJoined(peer)) => {
                let p2p_cmd = self
                    .p2p
                    .update(p2p::Event::PeerJoined(peer), &mut model.p2p)
                    .map_event(Event::P2p)
                    .map_effect(Effect::from);

                let snapshot = model.crdt.export_snapshot();
                let broadcast_cmd = self
                    .p2p
                    .update(p2p::Event::Broadcast(snapshot), &mut model.p2p)
                    .map_event(Event::P2p)
                    .map_effect(Effect::from);

                Command::all(vec![p2p_cmd, broadcast_cmd])
            }

            Event::P2p(event) => self
                .p2p
                .update(event, &mut model.p2p)
                .map_event(Event::P2p)
                .map_effect(Effect::from),
        }
    }

    fn view(&self, model: &Model) -> ViewModel {
        let crdt = self.crdt.view(&model.crdt);
        let p2p = self.p2p.view(&model.p2p);
        ViewModel { crdt, p2p }
    }
}

impl From<crdt::Effect> for Effect {
    fn from(crdt::Effect::Render(r): crdt::Effect) -> Self {
        Effect::Render(r)
    }
}

impl From<p2p::Effect> for Effect {
    fn from(p2p::Effect::Render(r): p2p::Effect) -> Self {
        Effect::Render(r)
    }
}
