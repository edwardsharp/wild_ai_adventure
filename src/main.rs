use axum::{
    extract::Extension, http::StatusCode, middleware as axum_middleware, response::IntoResponse,
    routing::post, Router,
};

use std::net::SocketAddr;
#[cfg(feature = "wasm")]
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
use crate::auth::{
    finish_authentication, finish_register, logout, start_authentication, start_register,
};
use crate::middleware::{require_authentication, security_logging};
use crate::startup::AppState;

#[macro_use]
extern crate tracing;

mod auth;
mod cli;
mod database;
mod error;
mod middleware;
mod startup;

#[cfg(all(feature = "javascript", feature = "wasm", not(doc)))]
compile_error!("Feature \"javascript\" and feature \"wasm\" cannot be enabled at the same time");

// 7. That's it! The user has now authenticated!

// =======
// Below is glue/stubs that are needed to make the above work, but don't really affect
// the work flow too much.

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    // initialize tracing
    tracing_subscriber::fmt::init();

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Create the app state with database connection
    let app_state = AppState::new(&database_url)
        .await
        .expect("Failed to initialize app state");

    // Create memory session store (for simplicity - use PostgresStore in production)
    let session_store = MemoryStore::default();

    // Create protected routes that require authentication
    let protected_routes = Router::new()
        .nest_service(
            "/private",
            tower_http::services::ServeDir::new("assets/private"),
        )
        .layer(axum_middleware::from_fn(require_authentication));

    // Create public routes
    let public_routes = Router::new().nest_service(
        "/public",
        tower_http::services::ServeDir::new("assets/public"),
    );

    // build our application with routes
    let app = Router::new()
        .route("/register_start/:username", post(start_register))
        .route("/register_finish", post(finish_register))
        .route("/login_start/:username", post(start_authentication))
        .route("/login_finish", post(finish_authentication))
        .route("/logout", post(logout))
        .merge(protected_routes)
        .merge(public_routes)
        .layer(Extension(app_state))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_name("webauthnrs")
                .with_same_site(SameSite::Strict)
                .with_secure(false) // TODO: change this to true when running on an HTTPS/production server instead of locally
                .with_expiry(Expiry::OnInactivity(Duration::seconds(360))),
        )
        .layer(axum_middleware::from_fn(security_logging))
        .fallback(handler_404);

    #[cfg(feature = "wasm")]
    if !PathBuf::from("./assets/wasm").exists() {
        panic!("Can't find WASM files to serve!")
    }

    #[cfg(feature = "wasm")]
    let app = Router::new()
        .merge(app)
        .nest_service("/", tower_http::services::ServeDir::new("assets/wasm"));

    #[cfg(feature = "javascript")]
    let app = Router::new()
        .merge(app)
        .nest_service("/", tower_http::services::ServeDir::new("assets/js"));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Unable to spawn tcp listener");

    axum::serve(listener, app).await.unwrap();
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
