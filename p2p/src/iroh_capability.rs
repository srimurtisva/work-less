// iroh_capability.rs
use crux_core::capability::Operation;
use facet::Facet;
use serde::{Deserialize, Serialize};

// Операция для подключения к сети iroh по тикету
#[derive(Facet, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrohConnect {
    pub ticket: String,
}

// Ответ на операцию подключения (например, ваш публичный ключ)
#[derive(Facet, Debug, PartialEq, Eq, Deserialize)]
pub struct IrohConnected {
    pub public_key: String,
}

impl Operation for IrohConnect {
    type Output = IrohConnected;
}

// Операция для рассылки данных
#[derive(Facet, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IrohBroadcast {
    pub data: Vec<u8>,
}

// Ответ на операцию рассылки (например, пустой тип, если не нужен)
#[derive(Facet, Debug, PartialEq, Eq, Deserialize)]
pub struct IrohBroadcasted;

impl Operation for IrohBroadcast {
    type Output = IrohBroadcasted;
}