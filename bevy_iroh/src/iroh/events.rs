use bevy_ecs::prelude::Event;
use derive_more::{Constructor, Deref, DerefMut, Display};

// --- Events are placed here in the iroh startup order.
/// After succesful start of 'endpoint'
#[derive(Debug, Display, Deref, DerefMut, Clone, Event)]
pub struct EndpointBinded(pub String);

/// Router builder is ready for accepting protocols. Take it from the event.
#[derive(Debug, Event)]
pub struct BuilderCreated;

/// All protocols are accepted. Builder is ready to build the router.
#[derive(Debug, Event)]
pub struct BuilderReady;

/// Router is ready and can be accessed as a resource.
#[derive(Debug, Display, Clone, Event)]
pub struct RouterSpawned;

#[derive(Debug, Display, Deref, DerefMut, Clone, Event)]
pub struct Error(pub String);

#[derive(Debug, Clone, Event)]
pub enum General {
    EndpointBinded(iroh::Endpoint),
}

#[deprecated]
#[derive(Debug, Display, Deref, DerefMut, Clone, Event)]
pub struct NodeStarted(pub String);

#[deprecated]
#[derive(Debug, Display, Clone, Event)]
pub struct NodeStopped;

#[deprecated]
// Входящие данные из сети в Bevy
#[derive(Event)]
pub struct NetworkDataReceived {
    pub sender: String,
    pub content: String,
}

#[deprecated]
// Исходящие данные из UI в сеть
#[derive(Event)]
pub struct SendNetworkMessage(pub String);
