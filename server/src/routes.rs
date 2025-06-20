//! Routes module
//!
//! This module contains the main routing logic and general app routes.
//! Domain-specific routes are handled by their respective modules.

use axum::{middleware as axum_middleware, routing::get, Router};
use tower_http::services::ServeDir;

use crate::analytics::build_analytics_routes;
use crate::auth::{build_auth_routes, require_authentication};
use crate::config::AppConfig;
use crate::health::health_check;

/// Build all routes for the application
pub fn build_routes(config: &AppConfig) -> Router {
    Router::new()
        .merge(build_auth_routes(config))
        .merge(build_analytics_routes(config))
        .merge(build_protected_routes(config))
        .merge(build_public_routes(config))
}

/// Build protected routes that require authentication (any role)
fn build_protected_routes(config: &AppConfig) -> Router {
    Router::new()
        .nest_service(
            "/private",
            ServeDir::new(&config.static_files.private_directory),
        )
        .route("/api/user/profile", get(health_check)) // Placeholder for user profile
        .layer(axum_middleware::from_fn(require_authentication))
}

/// Build public routes (no authentication required)
fn build_public_routes(config: &AppConfig) -> Router {
    Router::new()
        .nest_service(
            "/public",
            ServeDir::new(&config.static_files.public_directory),
        )
        .route("/health", get(health_check))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_routes() {
        let config = AppConfig::default();
        let _router = build_routes(&config);
        // Basic test to ensure router builds without panicking
    }

    #[test]
    fn test_protected_routes() {
        let config = AppConfig::default();
        let _router = build_protected_routes(&config);
        // Basic test to ensure router builds without panicking
    }

    #[test]
    fn test_public_routes() {
        let config = AppConfig::default();
        let _router = build_public_routes(&config);
        // Basic test to ensure router builds without panicking
    }
}
