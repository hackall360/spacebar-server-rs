use axum::body::Body;
use axum::middleware::Next;
use axum::response::Response;
use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::from_fn,
    routing::post,
    Json, Router,
};
use serde::Deserialize;

use crate::AppState;

#[derive(Deserialize)]
struct StopRequest {
    reason: Option<String>,
}

async fn handler(State(_state): State<AppState>, Json(payload): Json<StopRequest>) -> StatusCode {
    println!("/stop was called: {:?}", payload.reason);
    StatusCode::OK
}

async fn auth(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let authorized = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "Bearer operator")
        .unwrap_or(false);
    if authorized {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(handler).layer(from_fn(auth)))
}
