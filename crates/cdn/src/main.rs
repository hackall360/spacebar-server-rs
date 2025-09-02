//! CDN service for serving static assets.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use axum::{
    extract::{MatchedPath, Path as AxumPath, State},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use config::Config;
use tokio::{fs, net::TcpListener, signal};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use util_db::{init_database, DbPool};

/// Shared application state.
#[derive(Clone)]
struct AppState {
    storage_root: Arc<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Load configuration and database.
    let _config = Config::init().await;
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".into());
    let db = init_database(&database_url).await?;

    // Run clean-up for any stale attachment signatures.
    cleanup_attachment_signatures(&db).await.ok();

    // Determine storage location for files.
    let storage_root = std::env::var("STORAGE_LOCATION").unwrap_or_else(|_| "files".to_string());
    let storage_root = Arc::new(PathBuf::from(storage_root));

    let state = AppState { storage_root };

    // Build application with routes and middleware.
    let mut app = Router::new()
        .nest("/avatars", avatars_router())
        .nest("/role-icons", role_icons_router())
        .nest("/emojis", avatars_router())
        .nest(
            "/guilds/:guild_id/users/:user_id/avatars",
            guild_profile_router(),
        )
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::new().allow_methods(Any).allow_origin(Any))
                .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024)),
        );

    // Enable request logging if requested.
    if std::env::var("LOG_REQUESTS").is_ok() {
        app = app.layer(TraceLayer::new_for_http());
    }

    let addr: SocketAddr = "0.0.0.0:3001".parse().unwrap();
    let listener = TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
}

/// Router for user avatars and other generic asset folders that follow the same
/// `:id/:hash` pattern.
fn avatars_router() -> Router<AppState> {
    Router::new()
        .route("/:id", get(get_simple_file))
        .route("/:id/:hash", get(get_nested_file))
}

/// Router for guild profile assets under
/// `/guilds/:guild_id/users/:user_id/avatars`.
fn guild_profile_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_guild_profile_root))
        .route("/:hash", get(get_guild_profile_file))
}

/// Router for role icons which follow the `:role_id/:hash` pattern.
fn role_icons_router() -> Router<AppState> {
    Router::new().route("/:role_id/:hash", get(get_nested_file))
}

/// Serve a file directly under `<storage>/<route>/<id>`.
async fn get_simple_file(
    AxumPath(id): AxumPath<String>,
    State(state): State<AppState>,
    matched: MatchedPath,
) -> Result<Response, StatusCode> {
    let route = route_base(matched.as_str())?;
    let path = state.storage_root.join(route).join(id);
    serve_path(&path).await
}

/// Serve a file under `<storage>/<route>/<id>/<hash>`.
async fn get_nested_file(
    AxumPath((id, hash)): AxumPath<(String, String)>,
    State(state): State<AppState>,
    matched: MatchedPath,
) -> Result<Response, StatusCode> {
    let route = route_base(matched.as_str())?;
    let path = state.storage_root.join(route).join(id).join(hash);
    serve_path(&path).await
}

/// Serve the avatar stored for a guild member without specifying a hash.
async fn get_guild_profile_root(
    AxumPath((guild_id, user_id)): AxumPath<(String, String)>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let path = state
        .storage_root
        .join("guilds")
        .join(guild_id)
        .join("users")
        .join(user_id)
        .join("avatars");
    serve_path(&path).await
}

/// Serve a specific guild profile avatar hash.
async fn get_guild_profile_file(
    AxumPath((guild_id, user_id, hash)): AxumPath<(String, String, String)>,
    State(state): State<AppState>,
) -> Result<Response, StatusCode> {
    let path = state
        .storage_root
        .join("guilds")
        .join(guild_id)
        .join("users")
        .join(user_id)
        .join("avatars")
        .join(hash);
    serve_path(&path).await
}

/// Utility to derive the first component of the matched route path.
fn route_base(path: &str) -> Result<&str, StatusCode> {
    path.trim_start_matches('/')
        .split('/')
        .next()
        .ok_or(StatusCode::NOT_FOUND)
}

/// Read a file from disk and return it as a response with appropriate headers.
async fn serve_path(path: &Path) -> Result<Response, StatusCode> {
    match fs::read(path).await {
        Ok(contents) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let res = Response::builder()
                .status(StatusCode::OK)
                .header(header::CACHE_CONTROL, "public, max-age=31536000")
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(axum::body::Body::from(contents))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(res)
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// Remove any stale signature parameters from attachment URLs in the database.
async fn cleanup_attachment_signatures(db: &DbPool) -> Result<(), sqlx::Error> {
    use sqlx::Row;

    let rows = sqlx::query(
        "SELECT id, url, proxy_url FROM attachments WHERE url LIKE '%?ex=%' OR proxy_url LIKE '%?ex=%'",
    )
    .fetch_all(db)
    .await?;

    for row in rows {
        let id: String = row.try_get("id")?;
        let url: Option<String> = row.try_get("url")?;
        let proxy_url: Option<String> = row.try_get("proxy_url")?;

        let new_url = url.map(|u| u.split("?ex=").next().unwrap().to_string());
        let new_proxy = proxy_url.map(|u| u.split("?ex=").next().unwrap().to_string());

        sqlx::query("UPDATE attachments SET url = ?, proxy_url = ? WHERE id = ?")
            .bind(new_url)
            .bind(new_proxy)
            .bind(id)
            .execute(db)
            .await?;
    }
    Ok(())
}
