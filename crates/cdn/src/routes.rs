use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    routing::{delete, get, post},
    Json, Router,
};
use infer::Infer;
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{signature, AppState};

pub fn attachments_router() -> Router<AppState> {
    Router::new()
        .route("/:channel_id", post(upload_attachment))
        .route("/:channel_id/:id/:filename", get(get_attachment))
        .route("/:channel_id/:id/:filename", delete(delete_attachment))
}

#[derive(Serialize)]
struct UploadResponse {
    id: String,
    content_type: String,
    filename: String,
    size: u64,
    url: String,
    path: String,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Serialize)]
struct Success {
    success: bool,
}

async fn upload_attachment(
    Path(channel_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, StatusCode> {
    let signature = headers
        .get("signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if signature != state.config.security.request_signature {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut file_bytes = None;
    let mut filename = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        if field.name() == Some("file") {
            filename = field.file_name().map(|s| sanitize_filename::sanitize(s));
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            file_bytes = Some(data.to_vec());
            break;
        }
    }

    let data = file_bytes.ok_or(StatusCode::BAD_REQUEST)?;
    let filename = filename.unwrap_or_else(|| "file".into());

    let id = Uuid::new_v4().to_string();
    let path = format!("attachments/{}/{}/{}", channel_id, id, filename);
    state
        .storage
        .set(&path, &data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let info = Infer::new();
    let content_type = info
        .get(&data)
        .map(|t| t.mime_type())
        .unwrap_or("application/octet-stream");

    let dims = imagesize::blob_size(&data).ok();
    let (width, height) = dims
        .map(|d| (Some(d.width as u32), Some(d.height as u32)))
        .unwrap_or((None, None));

    let endpoint = state
        .config
        .cdn
        .endpoint
        .endpoint_public
        .clone()
        .unwrap_or_else(|| "http://localhost:3001".into());
    let url = format!("{}/{}", endpoint.trim_end_matches('/'), path);

    Ok(Json(UploadResponse {
        id,
        content_type: content_type.to_string(),
        filename,
        size: data.len() as u64,
        url,
        path,
        width,
        height,
    }))
}

async fn get_attachment(
    Path((channel_id, id, filename)): Path<(String, String, String)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
) -> Result<Response, StatusCode> {
    let path = format!("attachments/{}/{}/{}", channel_id, id, filename);

    if state.config.security.cdn_sign_urls {
        let ex = params.get("ex").ok_or(StatusCode::NOT_FOUND)?;
        let is = params.get("is").ok_or(StatusCode::NOT_FOUND)?;
        let hm = params.get("hm").ok_or(StatusCode::NOT_FOUND)?;

        let ua = headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok());
        if !signature::has_valid_signature(
            &format!("/attachments/{}/{}/{}", channel_id, id, filename),
            ex,
            is,
            hm,
            Some(&addr.ip().to_string()),
            ua,
            &state.config,
        ) {
            return Err(StatusCode::NOT_FOUND);
        }
    }

    let data = state
        .storage
        .get(&path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let info = Infer::new();
    let mut mime = info
        .get(&data)
        .map(|t| t.mime_type())
        .unwrap_or("application/octet-stream");
    let sanitized = [
        "text/html",
        "text/mhtml",
        "multipart/related",
        "application/xhtml+xml",
    ];
    if sanitized.contains(&mime) {
        mime = "application/octet-stream";
    }

    let res = Response::builder()
        .status(StatusCode::OK)
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .header(header::CONTENT_TYPE, mime)
        .body(axum::body::Body::from(data))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(res)
}

async fn delete_attachment(
    Path((channel_id, id, filename)): Path<(String, String, String)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Success>, StatusCode> {
    let signature = headers
        .get("signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if signature != state.config.security.request_signature {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let path = format!("attachments/{}/{}/{}", channel_id, id, filename);
    state
        .storage
        .delete(&path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(Success { success: true }))
}
