use bevy_ecs::prelude::*;
use derive_more::{Deref, DerefMut, Constructor};
use iroh_gossip::Gossip;

#[derive(Component, Deref, DerefMut, Constructor)]
pub struct GossipComponent(pub iroh_gossip::Gossip);
