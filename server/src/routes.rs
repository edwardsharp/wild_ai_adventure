//! Routes module
//!
//! This module contains the main routing logic and general app routes.
//! Domain-specific routes are handled by their respective modules.

use axum::Router;

use crate::analytics::build_analytics_routes;
use crate::auth::build_auth_routes;
use crate::config::AppConfig;
use crate::health::build_health_routes;
use crate::static_filez::{build_protected_static_routes, build_public_static_routes};

/// Build all routes for the application
pub fn build_routes(config: &AppConfig) -> Router {
    Router::new()
        .merge(build_auth_routes(config))
        .merge(build_analytics_routes(config))
        .merge(build_protected_static_routes(config))
        .merge(build_public_static_routes(config))
        .merge(build_health_routes())
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
}
