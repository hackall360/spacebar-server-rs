use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Routes that do not require authentication.
const NO_AUTHORIZATION_ROUTES: &[(&str, &str)] =
    &[("GET", "/ping"), ("POST", "/science"), ("POST", "/track")];

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

/// Simple bearer token authentication layer used for testing.
pub async fn authentication(req: Request<Body>, next: Next) -> Response {
    let method = req.method().as_str();
    let path = req.uri().path();
    if NO_AUTHORIZATION_ROUTES
        .iter()
        .any(|(m, p)| *m == method && path.starts_with(p))
    {
        return next.run(req).await;
    }

    let authorized = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "Bearer operator")
        .unwrap_or(false);

    if authorized {
        next.run(req).await
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}
