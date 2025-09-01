use std::{collections::HashSet, net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::Response,
    routing::get,
    serve, Router,
};
use config::Config;
use events::init_event;
use tokio::{
    net::TcpListener,
    signal,
    sync::{oneshot, Mutex},
};
use util_db::{close_database, init_database, DbPool};

#[derive(Clone)]
struct GatewayState {
    db: DbPool,
    config: Arc<Config>,
    connections: Arc<Mutex<HashSet<SocketAddr>>>,
}

pub struct GatewayServer {
    port: u16,
    state: Option<GatewayState>,
    shutdown: Option<oneshot::Sender<()>>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl GatewayServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            state: None,
            shutdown: None,
            handle: None,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let config = Config::init().await;
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".into());
        let db = init_database(&database_url).await?;
        init_event().await?;

        let state = GatewayState {
            db,
            config,
            connections: Arc::new(Mutex::new(HashSet::new())),
        };
        self.state = Some(state.clone());

        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(state);

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let listener = TcpListener::bind(addr).await?;
        let (tx, rx) = oneshot::channel();
        let server = serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = rx.await;
        });

        self.handle = Some(tokio::spawn(async move {
            let _ = server.await;
        }));
        self.shutdown = Some(tx);
        println!("[Gateway] listening on {}", addr);
        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
        if let Some(state) = self.state.take() {
            close_database(state.db).await;
        }
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<GatewayState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr, state: GatewayState) {
    state.connections.lock().await.insert(addr);
    let total = state.connections.lock().await.len();
    println!("[Gateway] New connection from {}, total {}", addr, total);

    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Close(_) = msg {
            break;
        }
    }

    state.connections.lock().await.remove(&addr);
    let total = state.connections.lock().await.len();
    println!("[Gateway] Connection closed from {}, total {}", addr, total);
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = GatewayServer::new(3001);
    server.start().await?;
    let _ = signal::ctrl_c().await;
    server.stop().await;
    Ok(())
}
