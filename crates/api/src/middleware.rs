use axum::{
    body::Body,
    http::{header, Request},
    middleware::Next,
    response::Response,
};

/// Middleware that extracts the `Accept-Language` header and stores it
/// in the request extensions for use by handlers.
pub async fn translation(mut req: Request<Body>, next: Next) -> Response {
    let lang = req
        .headers()
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    if let Some(lang) = lang {
        req.extensions_mut().insert(lang);
    }
    next.run(req).await
}

/// Very permissive CORS middleware used for development.
pub async fn cors(req: Request<Body>, next: Next) -> Response {
    let mut res = next.run(req).await;
    res.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::HeaderValue::from_static("*"),
    );
    res
}
