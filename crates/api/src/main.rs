//! API service entry point using Axum.

use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{middleware::from_fn, serve};
use config::Config;
use sentry_tower::{NewSentryLayer, SentryHttpLayer};
use tokio::{net::TcpListener, signal};
use tower::limit::ConcurrencyLimitLayer;

use events::init_event;
use util_db::{init_database, DbPool};

mod middleware;
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
            .layer(ConcurrencyLimitLayer::new(100))
            .layer(NewSentryLayer::new_from_top())
            .layer(SentryHttpLayer::new().enable_transaction());

        // Start HTTP server
        let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
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

#[tokio::main]
async fn main() -> Result<()> {
    SpacebarServer::start().await
}
