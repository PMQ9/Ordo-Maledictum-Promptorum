//! Intent Segregation API Server
//!
//! This server orchestrates all modules in the Intent Segregation Architecture:
//! - Malicious input detection
//! - Parser ensemble with voting
//! - Intent comparison against provider config
//! - Human approval workflow
//! - Trusted intent generation
//! - Processing engine execution
//! - Append-only audit ledger
//!
//! Built with Axum for high-performance async HTTP handling.

mod config;
mod error;
mod handlers;
mod middleware;
mod state;
mod types;

use crate::config::Config;
use crate::error::AppError;
use crate::handlers::{approval, health, ledger, process};
use crate::middleware::{logging_middleware, request_id_middleware};
use crate::state::AppState;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing/logging
    init_tracing();

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded successfully");

    // Initialize application state
    let state = AppState::new(config.clone()).await?;
    let state = Arc::new(state);
    info!("Application state initialized");

    // Build the application router
    let app = build_router(state.clone(), &config);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Starting server on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

/// Initialize tracing/logging with environment filter
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,intent_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Build the application router with all routes and middleware
fn build_router(state: Arc<AppState>, config: &Config) -> Router {
    // API routes
    let api_routes = Router::new()
        // Process user input through full pipeline
        .route("/process", post(process::process_input))
        // Approval endpoints
        .route("/approvals/:id", get(approval::get_approval_status))
        .route("/approvals/:id", post(approval::submit_approval_decision))
        // Ledger query endpoints
        .route("/ledger/query", get(ledger::query_ledger))
        .route("/ledger/stats", get(ledger::get_stats))
        .route("/ledger/:id", get(ledger::get_entry));

    // Health check route
    let health_routes = Router::new().route("/health", get(health::health_check));

    // Combine all routes
    let mut app = Router::new()
        .nest("/api", api_routes)
        .merge(health_routes)
        .with_state(state);

    // Serve static frontend files if configured
    if let Some(frontend_path) = &config.server.frontend_path {
        info!("Serving static frontend from: {}", frontend_path);
        app = app.nest_service(
            "/",
            ServeDir::new(frontend_path).append_index_html_on_directories(true),
        );
    }

    // Add middleware layers
    app.layer(
        ServiceBuilder::new()
            // CORS layer
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            // HTTP tracing layer
            .layer(TraceLayer::new_for_http())
            // Custom middleware
            .layer(axum_middleware::from_fn(request_id_middleware))
            .layer(axum_middleware::from_fn(logging_middleware)),
    )
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Received Ctrl+C signal, initiating graceful shutdown");
        },
        _ = terminate => {
            warn!("Received terminate signal, initiating graceful shutdown");
        },
    }
}
