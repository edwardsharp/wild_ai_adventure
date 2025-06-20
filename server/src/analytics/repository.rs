use super::models::{
    AnalyticsError, PathMetric, RequestAnalytics, RequestMetrics, TimeSeriesPoint,
};
use crate::database::DatabaseConnection;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

/// Repository for analytics-related database operations
pub struct AnalyticsRepository<'a> {
    db: &'a DatabaseConnection,
}

impl<'a> AnalyticsRepository<'a> {
    /// Create a new AnalyticsRepository
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self { db }
    }

    /// Record a new request analytics entry
    pub async fn record_request(&self, analytics: &RequestAnalytics) -> Result<(), AnalyticsError> {
        sqlx::query(
            r#"
            INSERT INTO request_analytics (
                request_id, timestamp, user_id, method, path, status_code,
                duration_ms, user_agent, ip_address, request_data, response_size,
                error_message, trace_id, span_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(&analytics.request_id)
        .bind(analytics.timestamp)
        .bind(analytics.user_id)
        .bind(&analytics.method)
        .bind(&analytics.path)
        .bind(analytics.status_code)
        .bind(analytics.duration_ms)
        .bind(&analytics.user_agent)
        .bind(&analytics.ip_address)
        .bind(&analytics.request_data)
        .bind(analytics.response_size)
        .bind(&analytics.error_message)
        .bind(&analytics.trace_id)
        .bind(&analytics.span_id)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    /// Get request analytics for a specific time range
    pub async fn get_requests_in_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<RequestAnalytics>, AnalyticsError> {
        let rows = sqlx::query(
            r#"
            SELECT id, request_id, timestamp, user_id, method, path, status_code,
                   duration_ms, user_agent, ip_address, request_data, response_size,
                   error_message, trace_id, span_id
            FROM request_analytics
            WHERE timestamp >= $1 AND timestamp <= $2
            ORDER BY timestamp DESC
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| RequestAnalytics {
                id: r.get("id"),
                request_id: r.get("request_id"),
                timestamp: r.get("timestamp"),
                user_id: r.get("user_id"),
                method: r.get("method"),
                path: r.get("path"),
                status_code: r.get("status_code"),
                duration_ms: r.get("duration_ms"),
                user_agent: r.get("user_agent"),
                ip_address: r.get("ip_address"),
                request_data: r.get("request_data"),
                response_size: r.get("response_size"),
                error_message: r.get("error_message"),
                trace_id: r.get("trace_id"),
                span_id: r.get("span_id"),
            })
            .collect())
    }

    /// Get request analytics for a specific user
    pub async fn get_user_requests(
        &self,
        user_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<RequestAnalytics>, AnalyticsError> {
        let rows = sqlx::query(
            r#"
            SELECT id, request_id, timestamp, user_id, method, path, status_code,
                   duration_ms, user_agent, ip_address, request_data, response_size,
                   error_message, trace_id, span_id
            FROM request_analytics
            WHERE user_id = $1 AND timestamp >= $2 AND timestamp <= $3
            ORDER BY timestamp DESC
            "#,
        )
        .bind(user_id)
        .bind(from)
        .bind(to)
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| RequestAnalytics {
                id: r.get("id"),
                request_id: r.get("request_id"),
                timestamp: r.get("timestamp"),
                user_id: r.get("user_id"),
                method: r.get("method"),
                path: r.get("path"),
                status_code: r.get("status_code"),
                duration_ms: r.get("duration_ms"),
                user_agent: r.get("user_agent"),
                ip_address: r.get("ip_address"),
                request_data: r.get("request_data"),
                response_size: r.get("response_size"),
                error_message: r.get("error_message"),
                trace_id: r.get("trace_id"),
                span_id: r.get("span_id"),
            })
            .collect())
    }

    /// Get aggregated request metrics for a time range
    pub async fn get_request_metrics(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<RequestMetrics, AnalyticsError> {
        // Get total requests and unique users
        let summary_row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_requests,
                COUNT(DISTINCT user_id) as unique_users,
                AVG(duration_ms) as avg_response_time,
                COUNT(CASE WHEN status_code >= 400 THEN 1 END)::FLOAT / COUNT(*)::FLOAT as error_rate
            FROM request_analytics
            WHERE timestamp >= $1 AND timestamp <= $2
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_one(self.db.pool())
        .await?;

        // Get most active paths
        let path_rows = sqlx::query(
            r#"
            SELECT
                path,
                COUNT(*) as request_count,
                AVG(duration_ms) as avg_response_time,
                COUNT(CASE WHEN status_code >= 400 THEN 1 END) as error_count
            FROM request_analytics
            WHERE timestamp >= $1 AND timestamp <= $2
            GROUP BY path
            ORDER BY request_count DESC
            LIMIT 10
            "#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(self.db.pool())
        .await?;

        let most_active_paths = path_rows
            .into_iter()
            .map(|r| PathMetric {
                path: r.get("path"),
                request_count: r.get("request_count"),
                average_response_time: r.get::<Option<f64>, _>("avg_response_time").unwrap_or(0.0),
                error_count: r.get("error_count"),
            })
            .collect();

        Ok(RequestMetrics {
            total_requests: summary_row.get("total_requests"),
            unique_users: summary_row.get("unique_users"),
            average_response_time: summary_row
                .get::<Option<f64>, _>("avg_response_time")
                .unwrap_or(0.0),
            error_rate: summary_row
                .get::<Option<f64>, _>("error_rate")
                .unwrap_or(0.0),
            most_active_paths,
        })
    }

    /// Get time-series data for request volume
    pub async fn get_request_volume_timeseries(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        interval_minutes: i32,
    ) -> Result<Vec<TimeSeriesPoint>, AnalyticsError> {
        let rows = sqlx::query(
            r#"
            SELECT
                DATE_TRUNC('minute', timestamp) +
                (EXTRACT(MINUTE FROM timestamp)::INTEGER / $3) * INTERVAL '1 minute' * $3 as time_bucket,
                COUNT(*) as request_count
            FROM request_analytics
            WHERE timestamp >= $1 AND timestamp <= $2
            GROUP BY time_bucket
            ORDER BY time_bucket
            "#,
        )
        .bind(from)
        .bind(to)
        .bind(interval_minutes)
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| TimeSeriesPoint {
                timestamp: r.get("time_bucket"),
                value: r.get::<i64, _>("request_count") as f64,
                label: None,
            })
            .collect())
    }

    /// Get error rate time-series data
    pub async fn get_error_rate_timeseries(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        interval_minutes: i32,
    ) -> Result<Vec<TimeSeriesPoint>, AnalyticsError> {
        let rows = sqlx::query(
            r#"
            SELECT
                DATE_TRUNC('minute', timestamp) +
                (EXTRACT(MINUTE FROM timestamp)::INTEGER / $3) * INTERVAL '1 minute' * $3 as time_bucket,
                COUNT(CASE WHEN status_code >= 400 THEN 1 END)::FLOAT / COUNT(*)::FLOAT as error_rate
            FROM request_analytics
            WHERE timestamp >= $1 AND timestamp <= $2
            GROUP BY time_bucket
            ORDER BY time_bucket
            "#,
        )
        .bind(from)
        .bind(to)
        .bind(interval_minutes)
        .fetch_all(self.db.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| TimeSeriesPoint {
                timestamp: r.get("time_bucket"),
                value: r.get::<Option<f64>, _>("error_rate").unwrap_or(0.0),
                label: None,
            })
            .collect())
    }

    /// Clean up old analytics data
    pub async fn cleanup_old_data(&self, older_than: DateTime<Utc>) -> Result<u64, AnalyticsError> {
        let result = sqlx::query(
            r#"
            DELETE FROM request_analytics
            WHERE timestamp < $1
            "#,
        )
        .bind(older_than)
        .execute(self.db.pool())
        .await?;

        Ok(result.rows_affected())
    }
}
