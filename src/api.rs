use crate::analytics::AnalyticsService;
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

/// Analytics response structure
#[derive(Serialize)]
pub struct AnalyticsResponse {
    pub stats: crate::analytics::AnalyticsStats,
    pub top_paths: Vec<crate::analytics::PathStats>,
    pub time_period_hours: i32,
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

/// User activity response
#[derive(Serialize)]
pub struct UserActivityResponse {
    pub user_id: Uuid,
    pub requests: Vec<crate::analytics::RequestAnalytics>,
    pub total_shown: usize,
}

/// Get analytics data (requires authentication)
pub async fn get_analytics(
    session: Session,
    Query(params): Query<AnalyticsQuery>,
    Extension(analytics): Extension<AnalyticsService>,
) -> Result<impl IntoResponse, WebauthnError> {
    // Check if user is authenticated
    let _user_id = session
        .get::<Uuid>("user_id")
        .await?
        .ok_or(WebauthnError::CorruptSession)?;

    // Get analytics stats
    let stats = analytics.get_stats(params.hours).await?;
    let top_paths = analytics.get_top_paths(params.hours, params.limit).await?;

    let response = AnalyticsResponse {
        stats,
        top_paths,
        time_period_hours: params.hours,
    };

    Ok(Json(response))
}

/// Get current user's activity (requires authentication)
pub async fn get_user_activity(
    session: Session,
    Query(params): Query<UserActivityQuery>,
    Extension(analytics): Extension<AnalyticsService>,
) -> Result<impl IntoResponse, WebauthnError> {
    // Check if user is authenticated and get their ID
    let user_id = session
        .get::<Uuid>("user_id")
        .await?
        .ok_or(WebauthnError::CorruptSession)?;

    // Get user's request history
    let requests = analytics.get_user_requests(user_id, params.limit).await?;
    let total_shown = requests.len();

    let response = UserActivityResponse {
        user_id,
        requests,
        total_shown,
    };

    Ok(Json(response))
}

/// Health check endpoint that includes basic analytics
pub async fn health_check(
    Extension(analytics): Extension<AnalyticsService>,
) -> Result<impl IntoResponse, WebauthnError> {
    // Get basic stats for the last hour
    let stats = analytics.get_stats(1).await?;

    let health_response = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "analytics": {
            "requests_last_hour": stats.total_requests,
            "errors_last_hour": stats.error_count,
            "avg_response_time_ms": stats.avg_duration_ms.unwrap_or(0.0)
        }
    });

    Ok(Json(health_response))
}

/// Get system metrics (public endpoint with basic info)
pub async fn get_metrics(
    Extension(analytics): Extension<AnalyticsService>,
) -> Result<impl IntoResponse, WebauthnError> {
    // Get stats for different time periods
    let last_hour = analytics.get_stats(1).await?;
    let last_24h = analytics.get_stats(24).await?;
    let last_week = analytics.get_stats(168).await?; // 7 * 24 hours

    let metrics = serde_json::json!({
        "system": {
            "name": "WebAuthn Demo",
            "version": "1.0.0",
            "uptime": "Available via process metrics"
        },
        "requests": {
            "last_hour": {
                "total": last_hour.total_requests,
                "success_rate": if last_hour.total_requests > 0 {
                    (last_hour.success_count as f64 / last_hour.total_requests as f64) * 100.0
                } else { 0.0 },
                "avg_duration_ms": last_hour.avg_duration_ms.unwrap_or(0.0)
            },
            "last_24h": {
                "total": last_24h.total_requests,
                "success_rate": if last_24h.total_requests > 0 {
                    (last_24h.success_count as f64 / last_24h.total_requests as f64) * 100.0
                } else { 0.0 },
                "avg_duration_ms": last_24h.avg_duration_ms.unwrap_or(0.0),
                "unique_users": last_24h.unique_users
            },
            "last_week": {
                "total": last_week.total_requests,
                "success_rate": if last_week.total_requests > 0 {
                    (last_week.success_count as f64 / last_week.total_requests as f64) * 100.0
                } else { 0.0 },
                "avg_duration_ms": last_week.avg_duration_ms.unwrap_or(0.0),
                "unique_users": last_week.unique_users
            }
        }
    });

    Ok(Json(metrics))
}

/// Admin-only endpoint to get detailed analytics (requires authentication)
pub async fn get_admin_analytics(
    session: Session,
    Query(params): Query<AnalyticsQuery>,
    Extension(analytics): Extension<AnalyticsService>,
) -> Result<impl IntoResponse, WebauthnError> {
    // Check if user is authenticated
    let _user_id = session
        .get::<Uuid>("user_id")
        .await?
        .ok_or(WebauthnError::CorruptSession)?;

    // In a real app, you'd check if the user has admin privileges here
    // For this demo, any authenticated user can access this

    let stats = analytics.get_stats(params.hours).await?;
    let top_paths = analytics.get_top_paths(params.hours, 50).await?; // More paths for admin

    // Additional detailed analytics
    let detailed_response = serde_json::json!({
        "overview": stats,
        "top_paths": top_paths,
        "time_period_hours": params.hours,
        "system_info": {
            "database_analytics_available": true,
            "tracing_enabled": true,
            "retention_policy": "30 days default"
        }
    });

    Ok(Json(detailed_response))
}

/// Prometheus-style metrics endpoint
pub async fn get_prometheus_metrics(
    Extension(analytics): Extension<AnalyticsService>,
) -> Result<impl IntoResponse, WebauthnError> {
    let stats_1h = analytics.get_stats(1).await?;
    let stats_24h = analytics.get_stats(24).await?;

    // Simple Prometheus-style metrics format
    let metrics = format!(
        r#"# HELP webauthn_requests_total Total number of HTTP requests
# TYPE webauthn_requests_total counter
webauthn_requests_total{{period="1h"}} {}
webauthn_requests_total{{period="24h"}} {}

# HELP webauthn_request_duration_ms Average request duration in milliseconds
# TYPE webauthn_request_duration_ms gauge
webauthn_request_duration_ms{{period="1h"}} {}
webauthn_request_duration_ms{{period="24h"}} {}

# HELP webauthn_errors_total Total number of error responses
# TYPE webauthn_errors_total counter
webauthn_errors_total{{period="1h"}} {}
webauthn_errors_total{{period="24h"}} {}

# HELP webauthn_unique_users Total number of unique users
# TYPE webauthn_unique_users gauge
webauthn_unique_users{{period="24h"}} {}
"#,
        stats_1h.total_requests,
        stats_24h.total_requests,
        stats_1h.avg_duration_ms.unwrap_or(0.0),
        stats_24h.avg_duration_ms.unwrap_or(0.0),
        stats_1h.error_count,
        stats_24h.error_count,
        stats_24h.unique_users
    );

    Ok((
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        metrics,
    ))
}

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
