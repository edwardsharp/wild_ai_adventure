//! Health check module
//!
//! Provides simple health check endpoints for monitoring and load balancers

use crate::error::WebauthnError;
use axum::response::{IntoResponse, Json};

/// Simple health check endpoint
/// Returns basic status information and timestamp
pub async fn health_check() -> Result<impl IntoResponse, WebauthnError> {
    let health_response = serde_json::json!({
        "status": "healthy",
        "timestamp": time::OffsetDateTime::now_utc(),
        "message": "WebAuthn server is running"
    });

    Ok(Json(health_response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let result = health_check().await;
        assert!(result.is_ok());
    }
}
