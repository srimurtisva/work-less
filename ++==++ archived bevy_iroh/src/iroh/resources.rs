use std::sync::Arc;

use bevy_ecs::prelude::*;
use derive_more::{Constructor, Deref, DerefMut};

/// Router shoud be kept alive for connections to function. Resource is the best form for that.
#[derive(Resource, Deref, DerefMut, Constructor)]
pub struct Router(iroh::protocol::Router);

/// Temporary builder for the iroh router
#[derive(Resource, Deref, DerefMut, Constructor)]
pub struct RouterBuilder(pub Option<iroh::protocol::RouterBuilder>);

/// This resource is meant for waiking up the consumer layer when a network event happens.
#[derive(Resource, Clone)]
pub struct Waker {
    pub notify: Arc<tokio::sync::Notify>,
}

impl Default for Waker {
    fn default() -> Self {
        Self {
            notify: Arc::new(tokio::sync::Notify::new()),
        }
    }
}

impl Waker {
    pub fn wake(&self) {
        self.notify.notify_one();
    }
    pub async fn wait(&self) {
        self.notify.notified().await;
    }
}
