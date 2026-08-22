use bevy_ecs::prelude::Event;
use derive_more::Display;


#[derive(Debug, Display, Clone, Event)]
pub struct NodeStarted(pub String);

#[derive(Debug, Display, Clone, Event)]
pub struct EndpointBinded (pub String);

#[derive(Debug, Display, Clone, Event)]
pub struct NodeStopped;

#[derive(Debug, Display, Clone, Event)]
pub struct Error(pub String);
