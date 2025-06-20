use axum::{
    extract::Extension, http::StatusCode, middleware as axum_middleware, response::IntoResponse,
};

use std::net::SocketAddr;
use std::path::PathBuf;
use tower_sessions::{
    cookie::{time::Duration, SameSite},
    Expiry, SessionManagerLayer,
};

// The handlers that process the data can be found in the auth.rs file
// This file contains the wasm client loading code and the axum routing
use webauthn_server::analytics::{analytics_middleware, security_logging};
use webauthn_server::config::AppConfig;
use webauthn_server::routes::build_routes;
use webauthn_server::startup::AppState;
use webauthn_server::storage::SessionStore;

#[macro_use]
extern crate tracing;

// Both JavaScript and WASM frontends are always available

// 7. That's it! The user has now authenticated!

// =======
// Below is glue/stubs that are needed to make the above work, but don't really affect
// the work flow too much.

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    if let Err(e) = dotenvy::dotenv() {
        // It's okay if .env file doesn't exist, just log it
        eprintln!("⚠️  Could not load .env file: {}", e);
    }

    // Load configuration and secrets
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.jsonc".to_string());
    let secrets_path =
        std::env::var("SECRETS_PATH").unwrap_or_else(|_| "config.secrets.jsonc".to_string());

    let config = if std::path::Path::new(&config_path).exists() {
        let secrets_path_opt = if std::path::Path::new(&secrets_path).exists() {
            Some(&secrets_path)
        } else {
            None
        };

        match AppConfig::from_files(&config_path, secrets_path_opt) {
            Ok((config, secrets)) => {
                println!("✅ Loaded configuration from {}", config_path);
                if secrets.is_some() {
                    println!("🔐 Loaded secrets from {}", secrets_path);
                }
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

    // Get session store and analytics service from app state
    let session_store = match &app_state.session_store {
        SessionStore::Memory(store) => SessionManagerLayer::new(store.clone()),
        SessionStore::Postgres(_store) => {
            // For now, fall back to memory store to avoid type conflicts
            SessionManagerLayer::new(tower_sessions::MemoryStore::default())
        }
    };

    // Get analytics service for middleware
    let analytics_service = app_state.analytics.clone();

    // Build all routes using the routes module

    // Build session manager with config
    let same_site = match config.sessions.same_site.as_str() {
        "strict" => SameSite::Strict,
        "lax" => SameSite::Lax,
        "none" => SameSite::None,
        _ => SameSite::Strict,
    };

    let session_layer = session_store
        .with_name("webauthnrs")
        .with_same_site(same_site)
        .with_secure(config.sessions.secure)
        .with_http_only(config.sessions.http_only)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(
            config.sessions.max_age_seconds,
        )));

    // Build main router with all routes
    let mut app = build_routes(&config)
        .layer(Extension(app_state.database.clone()))
        .layer(Extension(app_state))
        .layer(axum_middleware::from_fn(security_logging));

    // Add analytics middleware if enabled
    if config.features.analytics_enabled {
        app = app
            .layer(axum_middleware::from_fn(analytics_middleware))
            .layer(Extension(analytics_service));
    }

    // Serve main assets directory (contains both JS and WASM frontends)
    let assets_dir = config.static_files.assets_directory.clone();

    if !PathBuf::from(&assets_dir).exists() {
        panic!("Can't find assets directory at: {}", assets_dir);
    }

    app = app.layer(session_layer).fallback_service(
        tower_http::services::ServeDir::new(&assets_dir).not_found_service(tower::service_fn(
            |_| async {
                Ok::<_, std::convert::Infallible>(
                    (StatusCode::NOT_FOUND, "nothing to see here").into_response(),
                )
            },
        )),
    );

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
    info!("💾 Analytics storage: {:?}", config.storage.analytics);
    info!("🗄️  Session storage: {:?}", config.storage.sessions);

    if config.development.auto_generate_invites && config.app.environment == "development" {
        info!("🎫 Auto-generating invite codes for development...");
        // This would be handled by the startup logic
    }

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Unable to spawn tcp listener");

    axum::serve(listener, app).await.unwrap();
}

// async fn handler_404() -> impl IntoResponse {
//     (StatusCode::NOT_FOUND, "nothing to see here")
// }
