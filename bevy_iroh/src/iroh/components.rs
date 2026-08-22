use bevy_ecs::prelude::*;
use derive_more::{Constructor, Deref, DerefMut};

#[derive(Component, Deref, DerefMut, Constructor)]
pub struct Router(iroh::protocol::Router);

#[derive(Component, Deref, DerefMut, Constructor)]
pub struct Gossip(iroh_gossip::Gossip);

