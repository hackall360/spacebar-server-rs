use axum::Router;

use crate::AppState;

pub mod ping;
pub mod science;
pub mod stop;
pub mod track;

/// Combine all API routes into a single router.
pub fn create_router() -> Router<AppState> {
    Router::new()
        .nest("/ping", ping::router())
        .nest("/stop", stop::router())
        .nest("/science", science::router())
        .nest("/track", track::router())
}
