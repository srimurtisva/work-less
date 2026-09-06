use bevy_ecs::prelude::Event;
use derive_more::{Deref, DerefMut, Display};

// --- Events are placed here in the iroh startup order.
/// After succesful start of 'endpoint'
#[derive(Debug,  Deref, DerefMut, Clone, Event)]
pub struct EndpointBinded(pub iroh::Endpoint);

/// Router builder is ready for accepting protocols. Take it from the event.
#[derive(Debug, Event)]
pub struct BuilderCreated;

/// All protocols are accepted. Builder is ready to build the router.
#[derive(Debug, Event)]
pub struct BuilderReady;

/// Router is ready and can be accessed as a resource.
#[derive(Debug, Display, Clone, Event)]
pub struct RouterSpawned;

