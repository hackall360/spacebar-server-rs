//! API service entry point using Axum.

use std::{net::SocketAddr, sync::Arc, thread::available_parallelism};

use anyhow::Result;
use axum::{middleware::from_fn, serve};
use config::Config;
use dotenvy::dotenv;
use sentry_tower::{NewSentryLayer, SentryHttpLayer};
use tokio::{net::TcpListener, signal};
use tower::limit::ConcurrencyLimitLayer;

use events::init_event;
use util_db::{init_database, DbPool};

mod middleware;
mod models;
mod routes;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Arc<Config>,
}

/// Primary server structure.
pub struct SpacebarServer;

impl SpacebarServer {
    /// Initialise configuration, database, events, sentry and HTTP routes.
    pub async fn start() -> Result<()> {
        // Load configuration file
        let config = Config::init().await;

        // Initialise database connection pool
        let database_url =
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".into());
        let db = init_database(&database_url).await?;

        // Initialise event system
        init_event().await?;

        // Configure Sentry if enabled
        let _sentry = if config.sentry.enabled {
            let mut opts = sentry::ClientOptions::new();
            opts.traces_sample_rate = config.sentry.trace_sample_rate;
            if let Some(env) = &config.sentry.environment {
                opts.environment = Some(env.clone().into());
            }
            Some(sentry::init((config.sentry.endpoint.as_str(), opts)))
        } else {
            None
        };

        let state = AppState { db, config };

        // Build routes and attach middleware
        let app = routes::create_router()
            .with_state(state)
            .layer(from_fn(middleware::cors))
            .layer(from_fn(middleware::translation))
            .layer(from_fn(middleware::authentication))
            .layer(ConcurrencyLimitLayer::new(100))
            .layer(NewSentryLayer::new_from_top())
            .layer(SentryHttpLayer::new().enable_transaction());

        // Start HTTP server
        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3001);
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr).await?;
        serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await?;

        Ok(())
    }
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:?}");
    }
}

fn run() -> Result<()> {
    dotenv().ok();

    let threads = std::env::var("THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            available_parallelism()
                .map(|n| n.get())
                .unwrap_or_else(|_| {
                    eprintln!("[API] Failed to get thread count! Using 1...");
                    1
                })
        });

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()?;

    runtime.block_on(async { SpacebarServer::start().await })
}
