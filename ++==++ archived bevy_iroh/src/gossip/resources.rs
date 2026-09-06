use bevy_ecs::prelude::*;
use derive_more::{Deref, DerefMut, Constructor};
use iroh::EndpointId;
use iroh_gossip::TopicId;


#[derive(Resource, Deref,  Constructor)]
pub struct Gossip(pub iroh_gossip::Gossip);

#[derive(Resource, Deref, Clone, Copy)]
pub struct Topic(pub TopicId);

#[derive(Resource, Deref, DerefMut,  Constructor)]
pub struct BootstrapPeers(pub Vec<EndpointId>);

#[derive(Resource, Deref)]
pub struct GossipSender(pub iroh_gossip::api::GossipSender);

#[derive(Resource, Deref)]
pub struct GossipReceiver(pub iroh_gossip::api::GossipReceiver);

#[derive(Resource)]
pub struct GossipChannel{
    pub receiver: iroh_gossip::api::GossipReceiver,
    pub sender: iroh_gossip::api::GossipSender,
}
