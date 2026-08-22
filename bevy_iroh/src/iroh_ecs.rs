use  derive_more::Display;

use bevy_app::{App, Plugin, Startup};
use bevy_ecs::prelude::*;
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use derive_more::{Constructor, Deref, DerefMut};
use iroh::{Endpoint, endpoint::presets};

pub struct IrohPlugin;

impl Plugin for IrohPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TokioTasksPlugin>() {
            app.add_plugins(TokioTasksPlugin::default());
        }
        app.add_systems(Startup, startup_iroh);
    }
}

fn startup_iroh(query_router: Query<(), With<Router>>, runtime: ResMut<TokioTasksRuntime>) {
    if !query_router.is_empty() {
        return; // already started
    }
    runtime.spawn_background_task(|mut ctx| async move {
        // let key = SecretKeyStore::load_secret_key();
        let dht = iroh_mainline_address_lookup::DhtAddressLookup::builder();
        let mdns = iroh_mdns_address_lookup::MdnsAddressLookup::builder();
        let endpoint = Endpoint::builder(presets::N0)
            // .secret_key(key)
            .address_lookup(dht)
            .address_lookup(mdns)
            .bind()
            .await
            // TODO: Spawn error event instead of throwing error message
            // .unwrap_or(default)
            .expect("Failed to start Iroh Endpoint");

        println!("Iroh Endpoint succesfuly started");
        println!("Node Id: {}", endpoint.id());

        let gossip = iroh_gossip::Gossip::builder().spawn(endpoint.clone());

        let router = iroh::protocol::Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        ctx.run_on_main_thread(move |ctx| {
            ctx.world.spawn(Router::new(router));
            ctx.world.spawn(Gossip::new(gossip));
            let id = endpoint.id().to_string();
            ctx.world
             .trigger(Event::NodeStarted(id));
        })
        .await;
    });
}

#[derive(Component, Deref, DerefMut, Constructor)]
pub struct Router(iroh::protocol::Router);

#[derive(Component, Deref, DerefMut, Constructor)]
pub struct Gossip(iroh_gossip::Gossip);

// #[derive(serde::Serialize, serde::Deserialize)]
// pub struct SecretKeyStore {
//     pub secret_key: SecretKey,
// }

// impl Default for SecretKeyStore {
//     fn default() -> Self {
//         Self {
//             secret_key: SecretKey::generate(),
//         }
//     }
// }

// impl SecretKeyStore {
//     pub fn load_secret_key() -> SecretKey {
//         let app_name = env!("CARGO_PKG_NAME");
//         let config_name = "SecretKey";

//         // TODO: use 'keyring' instead of 'confy' for storing SecretKey
//         let result: Result<SecretKeyStore, confy::ConfyError> = confy::load(app_name, config_name);

//         match result {
//             Ok(loaded_key) => loaded_key.secret_key,
//             Err(e) => {
//                 eprint!("Failed to read secret key from disk: {}", e);
//                 Self::default().secret_key
//             }
//         }
//     }
// }

#[derive(Debug, Display,Clone,bevy_ecs::event::Event)]
pub enum Event {
    // Error(String),
    // PrepearingEcs,
    // EcsReady,
    NodeStarted (String),
}

