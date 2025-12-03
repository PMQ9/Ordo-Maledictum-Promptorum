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
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if it exists (for API keys and secrets)
    // Silently ignore if .env doesn't exist (allows production with system env vars)
    let _ = dotenvy::dotenv();

    // Initialize tracing/logging
    init_tracing();
    eprintln!("[STARTUP] Initializing tracing...");

    // Load configuration
    eprintln!("[STARTUP] Loading configuration from config/default.toml...");
    let config = Config::load().map_err(|e| {
        eprintln!("[FATAL] Failed to load configuration: {}", e);
        e
    })?;
    eprintln!("[STARTUP] Configuration loaded successfully");
    eprintln!(
        "[STARTUP] Database URL configured: postgresql://*:*@{}:{}/{}",
        "localhost", 5432, "intent_segregation"
    );
    eprintln!("[STARTUP] Server port: {}", config.server.port);

    // Initialize application state
    eprintln!("[STARTUP] Creating database connection pool...");
    let state = AppState::new(config.clone()).await.map_err(|e| {
        eprintln!("[FATAL] Failed to initialize application state: {}", e);
        eprintln!(
            "[FATAL] Check that PostgreSQL is running at {}",
            config.database.url
        );
        e
    })?;
    let state = Arc::new(state);
    eprintln!("[STARTUP] Application state initialized successfully");
    info!("Application state initialized");

    // Build the application router
    eprintln!("[STARTUP] Building router...");
    let app = build_router(state.clone(), &config);
    eprintln!("[STARTUP] Router built successfully");

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    eprintln!("[STARTUP] Binding to {}", addr);
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        eprintln!("[FATAL] Failed to bind to {}: {}", addr, e);
        e
    })?;

    eprintln!("[STARTUP] Server listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    eprintln!("[STARTUP] Server shutdown complete");
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
    let app = Router::new()
        .nest("/api", api_routes)
        .merge(health_routes)
        .with_state(state)
        // Add CORS layer
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
        .layer(axum_middleware::from_fn(logging_middleware));

    if let Some(frontend_path) = &config.server.frontend_path {
        info!("Note: Static frontend serving not configured (remove ServeDir import in future)");
        info!("Frontend path would be: {}", frontend_path);
    }

    app
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
