use std::collections::HashMap;
use std::net::SocketAddr;

use axum::extract::ws::{Message, WebSocket};
use serde_json::json;
use uuid::Uuid;

use crate::{GatewayState, ConnectionInfo};
use crate::error::GatewayError;
use crate::opcodes::{self, Payload};

pub async fn handle_socket(
    mut socket: WebSocket,
    addr: SocketAddr,
    query: HashMap<String, String>,
    state: GatewayState,
) {
    let session_id = Uuid::new_v4().to_string();

    let mut shard = None;
    if let Some(s) = query.get("shard") {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() == 2 {
            if let (Ok(id), Ok(count)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
                shard = Some(crate::ShardInfo { id, count });
            }
        }
    }

    {
        let mut conns = state.connections.lock().await;
        conns.insert(addr, ConnectionInfo { session_id: session_id.clone(), shard });
    }
    let total = state.connections.lock().await.len();
    println!("[Gateway] New connection from {addr}, session {session_id}, total {total}");

    let hello = json!({"op": 10, "d": {"heartbeat_interval": 30_000}});
    let _ = socket.send(Message::Text(hello.to_string())).await;

    loop {
        let msg = match socket.recv().await {
            Some(Ok(m)) => m,
            _ => break,
        };

        match msg {
            Message::Text(text) => {
                match serde_json::from_str::<Payload>(&text) {
                    Ok(payload) => {
                        if let Err(err) = opcodes::dispatch(&mut socket, &state, payload).await {
                            let _ = socket
                                .send(Message::Close(Some(err.close_frame())))
                                .await;
                            break;
                        }
                    }
                    Err(_) => {
                        let err = GatewayError::DecodeError;
                        let _ = socket
                            .send(Message::Close(Some(err.close_frame())))
                            .await;
                        break;
                    }
                }
            }
            Message::Binary(_) => {
                let err = GatewayError::DecodeError;
                let _ = socket
                    .send(Message::Close(Some(err.close_frame())))
                    .await;
                break;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    {
        let mut conns = state.connections.lock().await;
        conns.remove(&addr);
        let total = conns.len();
        println!("[Gateway] Connection closed from {addr}, total {total}");
    }
}
