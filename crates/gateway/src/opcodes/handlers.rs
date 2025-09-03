use axum::extract::ws::{Message, WebSocket};
use serde_json::{Value, json};

use crate::{GatewayState};
use crate::error::GatewayError;

pub async fn heartbeat(
    socket: &mut WebSocket,
    _state: &GatewayState,
    _data: Value,
) -> Result<(), GatewayError> {
    let ack = json!({"op": 11});
    let _ = socket.send(Message::Text(ack.to_string())).await;
    Ok(())
}

pub async fn identify(
    _socket: &mut WebSocket,
    _state: &GatewayState,
    data: Value,
) -> Result<(), GatewayError> {
    println!("[Gateway] Identify: {}", data);
    Ok(())
}

pub async fn resume(
    _socket: &mut WebSocket,
    _state: &GatewayState,
    _data: Value,
) -> Result<(), GatewayError> {
    println!("[Gateway] Resume received");
    Ok(())
}
