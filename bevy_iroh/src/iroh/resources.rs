use bevy_ecs::prelude::*;
use derive_more::{Constructor, Deref, DerefMut};

/// Router shoud be kept alive for connections to function. Resource is the best form for that.
#[derive(Resource, Deref, DerefMut, Constructor)]
pub struct Router(iroh::protocol::Router);


#[derive(Resource, Deref, DerefMut, Constructor)]
pub struct RouterBuilder(pub Option<iroh::protocol::RouterBuilder>);

/// Empty signal 'Sender' from async thread to wake the main thread up.
#[derive(Resource, Deref, DerefMut)]
pub struct MainThreadWaker(pub std::sync::mpsc::Sender<super::events::General>);

