//! Static Filez module
//!
//! This module handles all static file serving functionality including:
//! - Public static files (no authentication required)
//! - Private static files (authentication required)
//! - Main assets directory with fallback handling

use axum::{http::StatusCode, middleware as axum_middleware, response::IntoResponse, Router};
use std::path::PathBuf;
use tower::service_fn;
use tower_http::services::ServeDir;

use crate::auth::require_authentication;
use crate::config::AppConfig;

/// Build public static file routes (no authentication required)
pub fn build_public_static_routes(config: &AppConfig) -> Router {
    Router::new().nest_service(
        "/public",
        ServeDir::new(&config.static_files.public_directory),
    )
}

/// Build protected static file routes (authentication required)
pub fn build_protected_static_routes(config: &AppConfig) -> Router {
    Router::new()
        .nest_service(
            "/private",
            ServeDir::new(&config.static_files.private_directory),
        )
        .layer(axum_middleware::from_fn(require_authentication))
}

/// Build the main assets fallback service
/// This serves the main assets directory and provides a 404 fallback
pub fn build_assets_fallback_service(config: &AppConfig) -> Router {
    let assets_dir = &config.static_files.assets_directory;

    // Validate assets directory exists
    if !PathBuf::from(assets_dir).exists() {
        panic!("Can't find assets directory at: {}", assets_dir);
    }

    Router::new().fallback_service(ServeDir::new(assets_dir).not_found_service(service_fn(
        |_| async {
            Ok::<_, std::convert::Infallible>(
                (StatusCode::NOT_FOUND, "nothing to see here").into_response(),
            )
        },
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_public_static_routes() {
        let config = AppConfig::default();
        let _router = build_public_static_routes(&config);
        // Basic test to ensure router builds without panicking
    }

    #[test]
    fn test_build_protected_static_routes() {
        let config = AppConfig::default();
        let _router = build_protected_static_routes(&config);
        // Basic test to ensure router builds without panicking
    }

    #[test]
    fn test_build_assets_fallback_service() {
        let mut config = AppConfig::default();
        // Use a directory that should exist for testing
        config.static_files.assets_directory = ".".to_string();

        let _service = build_assets_fallback_service(&config);
        // Basic test to ensure service builds without panicking
    }
}
