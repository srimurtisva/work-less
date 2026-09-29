// p2p/src/iroh_middleware.rs
use std::sync::mpsc::{Sender, channel};
use std::thread;

use crux_core::middleware::{EffectMiddleware, EffectResolver};

use crate::iroh_capability::{IrohBroadcast, IrohBroadcasted};

pub struct IrohBroadcastMiddleware {
    jobs_tx: Sender<(IrohBroadcast, EffectResolver<IrohBroadcasted>)>,
}

impl IrohBroadcastMiddleware {
    pub fn new() -> Self {
        let (jobs_tx, jobs_rx) = channel::<(IrohBroadcast, EffectResolver<IrohBroadcasted>)>();

        // Фоновый поток — здесь позже будет реальная отправка через gossip
        thread::spawn(move || {
            while let Ok((broadcast, mut resolver)) = jobs_rx.recv() {
                tracing::info!(
                    "IrohBroadcastMiddleware: получен broadcast, {} байт",
                    broadcast.data.len()
                );

                // ЗАГЛУШКА: пока просто резолвим успех.
                // Позже сюда придёт реальная логика отправки.
                resolver.resolve(IrohBroadcasted);
            }
        });

        Self { jobs_tx }
    }
}

impl EffectMiddleware for IrohBroadcastMiddleware {
    type Op = IrohBroadcast;

    fn try_process_effect(
        &self,
        operation: IrohBroadcast,
        resolver: EffectResolver<IrohBroadcasted>,
    ) {
        self.jobs_tx
            .send((operation, resolver))
            .expect("IrohBroadcastMiddleware: не удалось отправить задачу в воркер");
    }
}