use axum::{
    extract::Extension,
    http::StatusCode,
    middleware as axum_middleware,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use std::net::SocketAddr;
use std::path::PathBuf;
use tower_sessions::{
    cookie::{time::Duration, SameSite},
    Expiry, MemoryStore, SessionManagerLayer,
};

/*
 * Webauthn RS server side tutorial.
 */

// The handlers that process the data can be found in the auth.rs file
// This file contains the wasm client loading code and the axum routing
use crate::api::{
    get_analytics, get_metrics, get_prometheus_metrics, get_user_activity, health_check,
};
use crate::auth::{
    finish_authentication, finish_register, logout, start_authentication, start_register,
};
use crate::config::AppConfig;
use crate::middleware::{analytics_middleware, require_authentication, security_logging};
use crate::startup::AppState;

#[macro_use]
extern crate tracing;

mod analytics;
mod api;
mod auth;
mod cli;
mod config;
mod database;
mod error;
mod middleware;
mod startup;

// Both JavaScript and WASM frontends are always available

// 7. That's it! The user has now authenticated!

// =======
// Below is glue/stubs that are needed to make the above work, but don't really affect
// the work flow too much.

#[tokio::main]
async fn main() {
    // Load configuration
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.jsonc".to_string());
    let config = if std::path::Path::new(&config_path).exists() {
        match AppConfig::from_file(&config_path) {
            Ok(config) => {
                println!("✅ Loaded configuration from {}", config_path);
                config
            }
            Err(e) => {
                eprintln!(
                    "❌ Failed to load configuration from {}: {}",
                    config_path, e
                );
                eprintln!(
                    "💡 Run 'cargo run --bin webauthn-admin config init' to create a config file"
                );
                eprintln!("🔄 Using default configuration...");
                AppConfig::default()
            }
        }
    } else {
        eprintln!("⚠️  Configuration file '{}' not found", config_path);
        eprintln!("💡 Run 'cargo run --bin webauthn-admin config init' to create one");
        eprintln!("🔄 Using default configuration...");
        AppConfig::default()
    };

    // Set up logging based on config
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", &config.logging.level);
    }
    tracing_subscriber::fmt::init();

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("❌ Configuration validation failed: {}", e);
        eprintln!("💡 Run 'cargo run --bin webauthn-admin config validate' for details");
        std::process::exit(1);
    }

    info!("🚀 Starting {} v{}", config.app.name, config.app.version);
    info!("🌍 Environment: {}", config.app.environment);

    // Create the app state with configuration
    let app_state = AppState::new(config.clone())
        .await
        .expect("Failed to initialize app state");

    // Create session store based on config
    let session_store = match config.sessions.store_type.as_str() {
        "memory" => MemoryStore::default(),
        // TODO: Add PostgreSQL session store support
        _ => {
            warn!(
                "Unknown session store type '{}', falling back to memory",
                config.sessions.store_type
            );
            MemoryStore::default()
        }
    };

    // Get analytics service for middleware
    let analytics_service = app_state.analytics.clone();

    // Create protected routes that require authentication
    let protected_routes = Router::new()
        .nest_service(
            "/private",
            tower_http::services::ServeDir::new(&config.static_files.private_directory),
        )
        .route("/api/analytics", get(get_analytics))
        .route("/api/user/activity", get(get_user_activity))
        .layer(axum_middleware::from_fn(require_authentication));

    // Create public routes
    let mut public_routes = Router::new().nest_service(
        "/public",
        tower_http::services::ServeDir::new(&config.static_files.public_directory),
    );

    // Add metrics endpoints if enabled
    if config.analytics.metrics.enabled {
        public_routes = public_routes
            .route(&config.analytics.metrics.health_endpoint, get(health_check))
            .route(
                &config.analytics.metrics.prometheus_endpoint,
                get(get_prometheus_metrics),
            )
            .route("/api/metrics", get(get_metrics));
    }

    // Build session manager with config
    let same_site = match config.sessions.same_site.as_str() {
        "strict" => SameSite::Strict,
        "lax" => SameSite::Lax,
        "none" => SameSite::None,
        _ => SameSite::Strict,
    };

    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("webauthnrs")
        .with_same_site(same_site)
        .with_secure(config.sessions.secure)
        .with_http_only(config.sessions.http_only)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(
            config.sessions.max_age_seconds,
        )));

    // Build main router
    let mut app = Router::new();

    // Add authentication routes if registration is enabled
    if config.features.registration_enabled {
        app = app
            .route("/register_start/:username", post(start_register))
            .route("/register_finish", post(finish_register));
    }

    app = app
        .route("/login_start/:username", post(start_authentication))
        .route("/login_finish", post(finish_authentication))
        .route("/logout", post(logout))
        .merge(protected_routes)
        .merge(public_routes)
        .layer(Extension(app_state))
        .layer(axum_middleware::from_fn(security_logging));

    // Add analytics middleware if enabled
    if config.features.analytics_enabled {
        app = app
            .layer(axum_middleware::from_fn(analytics_middleware))
            .layer(Extension(analytics_service));
    }

    app = app.layer(session_layer).fallback(handler_404);

    // Serve main assets directory (contains both JS and WASM frontends)
    let assets_dir = config.static_files.assets_directory.clone();

    if !PathBuf::from(&assets_dir).exists() {
        panic!("Can't find assets directory at: {}", assets_dir);
    }

    app = Router::new()
        .merge(app)
        .nest_service("/", tower_http::services::ServeDir::new(&assets_dir));

    // Parse server address from config
    let host = config
        .server
        .host
        .parse::<std::net::IpAddr>()
        .unwrap_or_else(|_| {
            eprintln!("❌ Invalid server host: {}", config.server.host);
            std::process::exit(1);
        });
    let addr = SocketAddr::from((host, config.server.port));

    info!("🌐 Server listening on {}", addr);
    info!("🔗 WebAuthn RP ID: {}", config.webauthn.rp_id);
    info!("🔗 WebAuthn Origin: {}", config.webauthn.rp_origin);
    info!(
        "📁 Assets directory: {}",
        config.static_files.assets_directory
    );

    if config.development.auto_generate_invites && config.app.environment == "development" {
        info!("🎫 Auto-generating invite codes for development...");
        // This would be handled by the startup logic
    }

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Unable to spawn tcp listener");

    axum::serve(listener, app).await.unwrap();
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
