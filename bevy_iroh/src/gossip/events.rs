use derive_more::*;
use iroh::PublicKey;
use iroh_gossip::TopicId;
use bevy_ecs::prelude::*;

#[derive(Event)]
pub struct SendGossipMessage {
    pub content: String,
}

#[derive(Event)]
pub struct GossipMessageReceived {
    pub from: PublicKey,
    pub content: String,
}

#[derive(Event,Deref,DerefMut)]
pub struct JoinTopic(pub TopicId);

#[derive(Event)]
pub struct GossipSpawned;
