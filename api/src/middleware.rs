//! Custom middleware for the API

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::{info, instrument};
use uuid::Uuid;

/// Middleware to add a unique request ID to each request
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4();

    // Insert request ID into extensions so handlers can access it
    request.extensions_mut().insert(request_id);

    // Add request ID to response headers
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("X-Request-ID", request_id.to_string().parse().unwrap());

    response
}

/// Middleware to log all requests and responses
#[instrument(skip(request, next))]
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = request.extensions().get::<Uuid>().copied();

    info!(
        request_id = ?request_id,
        method = %method,
        uri = %uri,
        "Incoming request"
    );

    let response = next.run(request).await;

    info!(
        request_id = ?request_id,
        status = response.status().as_u16(),
        "Request completed"
    );

    response
}

/// Middleware to enforce request size limits
pub async fn request_size_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    const MAX_SIZE: usize = 1024 * 1024; // 1MB

    let content_length = request
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok());

    if let Some(length) = content_length {
        if length > MAX_SIZE {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                "Request body too large (max 1MB)",
            ));
        }
    }

    Ok(next.run(request).await)
}
