use crate::iroh::events::EndpointBinded;
use bevy_ecs::prelude::*;
use iroh::endpoint::presets;

pub fn startup_endpoint(commands: Commands) {
    let (sender, reciever) = std::sync::mpsc::channel::<EndpointBinded>();

    tokio::spawn(startup_endpoint_task(sender));

    trigger_event(commands, reciever);
}

async fn startup_endpoint_task (
    sender: std::sync::mpsc::Sender<EndpointBinded>,
){
        let Some(endpoint) = create_endpoint().await else {
            return;
        };

        tracing::info!("Iroh Endpoint succesfuly started.");

        send_event_to_the_main_thread(sender, EndpointBinded(endpoint));
        tracing::info!("EndpointBinded event triggered.");
}

fn trigger_event(mut commands: Commands, reciever: std::sync::mpsc::Receiver<EndpointBinded>) {
    if let Ok(event) = reciever.recv() {
        commands.trigger(event);
        tracing::info!("Triggered event.");
    }
}

async fn create_endpoint() -> Option<iroh::Endpoint> {
    let key = super::util::NodeIdentityManager::auto().get_or_create_key();

    let dht = iroh_mainline_address_lookup::DhtAddressLookup::builder();
    let mdns = iroh_mdns_address_lookup::MdnsAddressLookup::builder();

    iroh::Endpoint::builder(presets::N0)
        .secret_key(key)
        .address_lookup(dht)
        .address_lookup(mdns)
        .bind()
        .await
        .inspect_err(|error| {
            tracing::error!(%error,"Failed to create Iroh Endpoint.");
        })
        .ok()
}

fn send_event_to_the_main_thread(
    sender: std::sync::mpsc::Sender<EndpointBinded>,
    event: EndpointBinded,
) {
    let _ = sender.send(event).inspect_err(|error| {
        tracing::error!("Failed to send event: {error}");
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn send_event_sends_event_to_receiver() {
        let (sender, receiver) = mpsc::channel();

        sender.send(42).unwrap();

        assert_eq!(receiver.recv().unwrap(), 42);
    }

}

