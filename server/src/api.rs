// API endpoints module
//
// This module contains API endpoints that provide analytics and user data.
// Some endpoints are temporarily commented out while we migrate to the new
// modular analytics structure.

use crate::error::WebauthnError;
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;
use uuid::Uuid;

/// Query parameters for analytics endpoints
#[derive(Deserialize)]
pub struct AnalyticsQuery {
    #[serde(default = "default_hours")]
    hours: i32,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_hours() -> i32 {
    24
}

fn default_limit() -> i64 {
    10
}

/// User activity query parameters
#[derive(Deserialize)]
pub struct UserActivityQuery {
    #[serde(default = "default_activity_limit")]
    limit: i64,
}

fn default_activity_limit() -> i64 {
    20
}

/// Simple health check endpoint
pub async fn health_check() -> Result<impl IntoResponse, WebauthnError> {
    let health_response = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "message": "WebAuthn server is running"
    });

    Ok(Json(health_response))
}

/// Simple metrics endpoint
pub async fn get_metrics() -> Result<impl IntoResponse, WebauthnError> {
    let metrics = serde_json::json!({
        "system": {
            "name": "WebAuthn Demo",
            "version": "1.0.0",
            "status": "running"
        },
        "note": "Detailed metrics will be available after analytics migration is complete"
    });

    Ok(Json(metrics))
}

/// Prometheus-style metrics endpoint (simplified)
pub async fn get_prometheus_metrics() -> Result<impl IntoResponse, WebauthnError> {
    let metrics = r#"# HELP webauthn_status Server status
# TYPE webauthn_status gauge
webauthn_status{service="webauthn"} 1

# Note: Detailed metrics will be available after analytics migration
"#;

    Ok((
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        metrics,
    ))
}

// TODO: Uncomment and update these endpoints once analytics migration is complete
//
// /// Analytics response structure
// #[derive(Serialize)]
// pub struct AnalyticsResponse {
//     pub stats: RequestMetrics,
//     pub top_paths: Vec<PathMetric>,
//     pub time_period_hours: i32,
// }
//
// /// User activity response
// #[derive(Serialize)]
// pub struct UserActivityResponse {
//     pub user_id: Uuid,
//     pub requests: Vec<RequestAnalytics>,
//     pub total_shown: usize,
// }
//
// /// Get analytics data (requires authentication)
// pub async fn get_analytics(
//     session: Session,
//     Query(params): Query<AnalyticsQuery>,
//     Extension(analytics): Extension<AnalyticsService<'_>>,
// ) -> Result<impl IntoResponse, WebauthnError> {
//     // Check if user is authenticated
//     let _user_id = session
//         .get::<Uuid>("user_id")
//         .await?
//         .ok_or(WebauthnError::CorruptSession)?;
//
//     // TODO: Implement with new analytics service
//     todo!("Update to use new analytics service")
// }
//
// /// Get current user's activity (requires authentication)
// pub async fn get_user_activity(
//     session: Session,
//     Query(params): Query<UserActivityQuery>,
//     Extension(analytics): Extension<AnalyticsService<'_>>,
// ) -> Result<impl IntoResponse, WebauthnError> {
//     // Check if user is authenticated and get their ID
//     let user_id = session
//         .get::<Uuid>("user_id")
//         .await?
//         .ok_or(WebauthnError::CorruptSession)?;
//
//     // TODO: Implement with new analytics service
//     todo!("Update to use new analytics service")
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_analytics_params() {
        let query = AnalyticsQuery {
            hours: default_hours(),
            limit: default_limit(),
        };
        assert_eq!(query.hours, 24);
        assert_eq!(query.limit, 10);
    }

    #[test]
    fn test_default_activity_limit() {
        assert_eq!(default_activity_limit(), 20);
    }
}
