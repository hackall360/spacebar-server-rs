use axum::extract::ws::WebSocket;
use serde::Deserialize;
use serde_json::Value;

use crate::{error::GatewayError, GatewayState};

mod handlers;

#[derive(Deserialize)]
pub struct Payload {
    pub op: u8,
    #[serde(default)]
    pub d: Value,
}

pub async fn dispatch(
    socket: &mut WebSocket,
    state: &GatewayState,
    payload: Payload,
) -> Result<(), GatewayError> {
    match payload.op {
        1 => handlers::heartbeat(socket, state, payload.d).await,
        2 => handlers::identify(socket, state, payload.d).await,
        6 => handlers::resume(socket, state, payload.d).await,
        _ => Err(GatewayError::UnknownOpcode(payload.op)),
    }
}
