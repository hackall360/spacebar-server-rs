use axum::{extract::State, http::StatusCode, routing::post, Router};

use crate::AppState;

async fn handler(State(_state): State<AppState>) -> StatusCode {
    StatusCode::NO_CONTENT
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(handler))
}
