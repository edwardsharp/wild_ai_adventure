use axum::{extract::Extension, middleware as axum_middleware};
use clap::Parser;

use std::net::SocketAddr;
use tower_sessions::{
    cookie::{time::Duration, SameSite},
    Expiry, SessionManagerLayer,
};

use server::analytics::{analytics_middleware, security_logging};
use server::config::AppConfig;
use server::routes::build_routes;
use server::startup::AppState;
use server::static_filez::build_assets_fallback_service;
use server::storage::SessionStore;

#[macro_use]
extern crate tracing;

#[derive(Parser)]
#[command(name = "server")]
#[command(about = "WebAuthn server application")]
struct ServerArgs {
    /// Path to configuration file
    #[arg(long, short, default_value = "assets/config/config.jsonc")]
    config: String,

    /// Path to secrets configuration file
    #[arg(long, short, default_value = "assets/config/config.secrets.jsonc")]
    secrets: String,
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = ServerArgs::parse();

    // Load environment variables from .env file
    if let Err(e) = dotenvy::dotenv() {
        // It's okay if .env file doesn't exist, just log it
        eprintln!("⚠️  Could not load .env file: {}", e);
    }

    // Use command line arguments for configuration paths
    let config_path = args.config;
    let secrets_path = args.secrets;

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
                eprintln!("💡 Run 'cargo run --bin cli config init' to create a config file");
                eprintln!("🔄 Using default configuration...");
                AppConfig::default()
            }
        }
    } else {
        eprintln!("⚠️  Configuration file '{}' not found", config_path);
        eprintln!("💡 Run 'cargo run --bin cli config init' to create one");
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
        eprintln!("💡 Run 'cargo run --bin cli config validate' for details");
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
    app = app
        .merge(build_assets_fallback_service(&config))
        .layer(session_layer);

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
