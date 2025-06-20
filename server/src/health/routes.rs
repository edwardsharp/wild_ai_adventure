//! Health routes module
//!
//! This module contains health check and monitoring related routes.

use axum::{routing::get, Router};

use super::health_check;

/// Build health check routes
pub fn build_health_routes() -> Router {
    Router::new().route("/health", get(health_check))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_health_routes() {
        let _router = build_health_routes();
        // Basic test to ensure router builds without panicking
    }
}
