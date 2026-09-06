use bevy_ecs::prelude::*;
use derive_more::*;
use iroh_gossip::TopicId;

// #[derive(Event)]
// pub struct SendGossipMessage {
//     pub content: String,
// }

// #[derive(Event)]
// pub struct GossipMessageReceived(pub iroh_gossip::api::Event);

#[derive(Event, Deref, DerefMut)]
pub struct JoinTopic(pub TopicId);

#[derive(Event)]
pub struct GossipSpawned;

#[derive(Event)]
pub struct TopicOpen;
