use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Row};
use std::collections::HashMap;

use std::time::Instant;
use uuid::Uuid;

/// Request analytics data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestAnalytics {
    pub id: Uuid,
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub method: String,
    pub path: String,
    pub status_code: i32,
    pub duration_ms: Option<i32>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub request_data: Option<JsonValue>,
    pub response_size: Option<i64>,
    pub error_message: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

/// Builder pattern for creating analytics records
#[derive(Debug, Default)]
pub struct AnalyticsBuilder {
    request_id: String,
    user_id: Option<Uuid>,
    method: String,
    path: String,
    status_code: i32,
    duration_ms: Option<i32>,
    user_agent: Option<String>,
    ip_address: Option<String>,
    request_data: Option<JsonValue>,
    response_size: Option<i64>,
    error_message: Option<String>,
    trace_id: Option<String>,
    span_id: Option<String>,
}

impl AnalyticsBuilder {
    pub fn new(request_id: String, method: String, path: String, status_code: i32) -> Self {
        Self {
            request_id,
            method,
            path,
            status_code,
            ..Default::default()
        }
    }

    pub fn user_id(mut self, user_id: Option<Uuid>) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn duration_ms(mut self, duration_ms: u128) -> Self {
        self.duration_ms = Some(duration_ms as i32);
        self
    }

    pub fn user_agent(mut self, user_agent: Option<String>) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn ip_address(mut self, ip_address: Option<String>) -> Self {
        self.ip_address = ip_address;
        self
    }

    pub fn request_data(mut self, data: HashMap<String, serde_json::Value>) -> Self {
        self.request_data = Some(serde_json::Value::Object(
            data.into_iter()
                .collect::<serde_json::Map<String, serde_json::Value>>(),
        ));
        self
    }

    pub fn response_size(mut self, size: Option<i64>) -> Self {
        self.response_size = size;
        self
    }

    pub fn error_message(mut self, error: Option<String>) -> Self {
        self.error_message = error;
        self
    }

    pub fn trace_id(mut self, trace_id: Option<String>) -> Self {
        self.trace_id = trace_id;
        self
    }

    pub fn span_id(mut self, span_id: Option<String>) -> Self {
        self.span_id = span_id;
        self
    }

    pub fn build(self) -> RequestAnalytics {
        RequestAnalytics {
            id: Uuid::new_v4(),
            request_id: self.request_id,
            timestamp: Utc::now(),
            user_id: self.user_id,
            method: self.method,
            path: self.path,
            status_code: self.status_code,
            duration_ms: self.duration_ms,
            user_agent: self.user_agent,
            ip_address: self.ip_address,
            request_data: self.request_data,
            response_size: self.response_size,
            error_message: self.error_message,
            trace_id: self.trace_id,
            span_id: self.span_id,
        }
    }
}

/// Analytics service for database operations
#[derive(Clone)]
pub struct AnalyticsService {
    pool: PgPool,
}

impl AnalyticsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Log a request to the analytics table
    pub async fn log_request(&self, analytics: RequestAnalytics) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO request_analytics (
                id, request_id, timestamp, user_id, method, path, status_code,
                duration_ms, user_agent, ip_address, request_data, response_size,
                error_message, trace_id, span_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(&analytics.id)
        .bind(&analytics.request_id)
        .bind(&analytics.timestamp)
        .bind(&analytics.user_id)
        .bind(&analytics.method)
        .bind(&analytics.path)
        .bind(&analytics.status_code)
        .bind(&analytics.duration_ms)
        .bind(&analytics.user_agent)
        .bind(&analytics.ip_address)
        .bind(&analytics.request_data)
        .bind(&analytics.response_size)
        .bind(&analytics.error_message)
        .bind(&analytics.trace_id)
        .bind(&analytics.span_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get recent requests for a user
    pub async fn get_user_requests(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RequestAnalytics>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, request_id, timestamp, user_id, method, path, status_code,
                   duration_ms, user_agent, ip_address, request_data, response_size,
                   error_message, trace_id, span_id
            FROM request_analytics
            WHERE user_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut analytics = Vec::new();
        for row in rows {
            analytics.push(RequestAnalytics {
                id: row.get("id"),
                request_id: row.get("request_id"),
                timestamp: row.get("timestamp"),
                user_id: row.get("user_id"),
                method: row.get("method"),
                path: row.get("path"),
                status_code: row.get("status_code"),
                duration_ms: row.get("duration_ms"),
                user_agent: row.get("user_agent"),
                ip_address: row.get("ip_address"),
                request_data: row.get("request_data"),
                response_size: row.get("response_size"),
                error_message: row.get("error_message"),
                trace_id: row.get("trace_id"),
                span_id: row.get("span_id"),
            });
        }

        Ok(analytics)
    }

    /// Get request statistics for a time period
    pub async fn get_stats(&self, hours: i32) -> Result<AnalyticsStats, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_requests,
                COUNT(DISTINCT user_id) as unique_users,
                AVG(duration_ms)::FLOAT8 as avg_duration_ms,
                COUNT(*) FILTER (WHERE status_code >= 400) as error_count,
                COUNT(*) FILTER (WHERE status_code < 400) as success_count
            FROM request_analytics
            WHERE timestamp > NOW() - ($1 || ' hours')::INTERVAL
            "#,
        )
        .bind(hours.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(AnalyticsStats {
            total_requests: row.get::<i64, _>("total_requests") as u64,
            unique_users: row.get::<i64, _>("unique_users") as u64,
            avg_duration_ms: row.get::<Option<f64>, _>("avg_duration_ms"),
            error_count: row.get::<i64, _>("error_count") as u64,
            success_count: row.get::<i64, _>("success_count") as u64,
        })
    }

    /// Get top paths by request count
    pub async fn get_top_paths(
        &self,
        hours: i32,
        limit: i64,
    ) -> Result<Vec<PathStats>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT path, COUNT(*) as request_count, AVG(duration_ms)::FLOAT8 as avg_duration_ms
            FROM request_analytics
            WHERE timestamp > NOW() - ($1 || ' hours')::INTERVAL
            GROUP BY path
            ORDER BY request_count DESC
            LIMIT $2
            "#,
        )
        .bind(hours.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(PathStats {
                path: row.get("path"),
                request_count: row.get::<i64, _>("request_count") as u64,
                avg_duration_ms: row.get::<Option<f64>, _>("avg_duration_ms"),
            });
        }

        Ok(stats)
    }

    /// Clean up old analytics data
    pub async fn cleanup_old_data(&self, days: i32) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM request_analytics
            WHERE timestamp < NOW() - ($1 || ' days')::INTERVAL
            "#,
        )
        .bind(days.to_string())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

/// Analytics statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsStats {
    pub total_requests: u64,
    pub unique_users: u64,
    pub avg_duration_ms: Option<f64>,
    pub error_count: u64,
    pub success_count: u64,
}

/// Path-specific statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct PathStats {
    pub path: String,
    pub request_count: u64,
    pub avg_duration_ms: Option<f64>,
}

/// Request context for tracking
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub start_time: Instant,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            start_time: Instant::now(),
            trace_id: None,
            span_id: None,
        }
    }

    pub fn with_trace(trace_id: String, span_id: String) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            start_time: Instant::now(),
            trace_id: Some(trace_id),
            span_id: Some(span_id),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start_time.elapsed().as_millis()
    }
}

/// Utility functions for OpenTelemetry-style trace/span generation
pub mod otel_utils {
    use rand::{thread_rng, Rng};

    /// Generate a trace ID (128-bit hex string)
    pub fn generate_trace_id() -> String {
        let mut rng = thread_rng();
        format!("{:032x}", rng.gen::<u128>())
    }

    /// Generate a span ID (64-bit hex string)
    pub fn generate_span_id() -> String {
        let mut rng = thread_rng();
        format!("{:016x}", rng.gen::<u64>())
    }

    /// Extract trace context from headers (W3C Trace Context format)
    pub fn extract_trace_context(headers: &axum::http::HeaderMap) -> Option<(String, String)> {
        if let Some(traceparent) = headers.get("traceparent") {
            if let Ok(traceparent_str) = traceparent.to_str() {
                // Parse W3C traceparent header: 00-{trace_id}-{span_id}-{flags}
                let parts: Vec<&str> = traceparent_str.split('-').collect();
                if parts.len() == 4 && parts[0] == "00" {
                    return Some((parts[1].to_string(), parts[2].to_string()));
                }
            }
        }
        None
    }

    /// Create a W3C traceparent header
    pub fn create_traceparent_header(trace_id: &str, span_id: &str) -> String {
        format!("00-{}-{}-01", trace_id, span_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_builder() {
        let analytics = AnalyticsBuilder::new(
            "test-id".to_string(),
            "GET".to_string(),
            "/test".to_string(),
            200,
        )
        .user_id(Some(Uuid::new_v4()))
        .duration_ms(150)
        .user_agent(Some("test-agent".to_string()))
        .build();

        assert_eq!(analytics.method, "GET");
        assert_eq!(analytics.path, "/test");
        assert_eq!(analytics.status_code, 200);
        assert_eq!(analytics.duration_ms, Some(150));
    }

    #[test]
    fn test_trace_id_generation() {
        let trace_id = otel_utils::generate_trace_id();
        assert_eq!(trace_id.len(), 32);
        assert!(trace_id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_span_id_generation() {
        let span_id = otel_utils::generate_span_id();
        assert_eq!(span_id.len(), 16);
        assert!(span_id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_traceparent_creation() {
        let trace_id = "12345678901234567890123456789012";
        let span_id = "1234567890123456";
        let header = otel_utils::create_traceparent_header(trace_id, span_id);
        assert_eq!(
            header,
            "00-12345678901234567890123456789012-1234567890123456-01"
        );
    }
}
