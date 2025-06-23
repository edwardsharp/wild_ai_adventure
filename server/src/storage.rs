use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use time::OffsetDateTime;
use uuid::Uuid;

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Memory,
    Postgres,
}

impl Default for StorageBackend {
    fn default() -> Self {
        Self::Memory
    }
}

/// Configuration for storage backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Analytics storage backend
    #[serde(default)]
    pub analytics: StorageBackend,
    /// Session storage backend
    #[serde(default)]
    pub sessions: StorageBackend,
    /// Cache storage backend
    #[serde(default)]
    pub cache: StorageBackend,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            analytics: StorageBackend::Memory,
            sessions: StorageBackend::Memory,
            cache: StorageBackend::Memory,
        }
    }
}

/// Analytics data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestAnalytics {
    pub id: Uuid,
    pub request_id: String,
    pub timestamp: OffsetDateTime,
    pub user_id: Option<Uuid>,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: i64,
    pub user_agent: Option<String>,
    pub ip_address: String,
    pub request_data: Option<JsonValue>,
    pub response_size: Option<i64>,
    pub error_message: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

/// Analytics statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsStats {
    pub total_requests: i64,
    pub unique_users: i64,
    pub avg_duration_ms: f64,
    pub error_count: i64,
    pub success_count: i64,
}

/// Path statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathStats {
    pub path: String,
    pub request_count: i64,
    pub avg_duration_ms: f64,
}

/// In-memory analytics storage
pub struct MemoryAnalyticsStore {
    data: Arc<RwLock<Vec<RequestAnalytics>>>,
}

impl MemoryAnalyticsStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn log_request(
        &self,
        analytics: RequestAnalytics,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut data = self.data.write().unwrap();
        data.push(analytics);
        Ok(())
    }

    pub async fn get_user_requests(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RequestAnalytics>, Box<dyn std::error::Error>> {
        let data = self.data.read().unwrap();
        let filtered: Vec<RequestAnalytics> = data
            .iter()
            .filter(|req| req.user_id == Some(user_id))
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(filtered)
    }

    pub async fn get_stats(
        &self,
        hours: i32,
    ) -> Result<AnalyticsStats, Box<dyn std::error::Error>> {
        let data = self.data.read().unwrap();
        let cutoff = OffsetDateTime::now_utc() - time::Duration::hours(hours as i64);

        let recent_requests: Vec<&RequestAnalytics> =
            data.iter().filter(|req| req.timestamp >= cutoff).collect();

        let total_requests = recent_requests.len() as i64;
        let unique_users = recent_requests
            .iter()
            .filter_map(|req| req.user_id)
            .collect::<std::collections::HashSet<_>>()
            .len() as i64;

        let total_duration: i64 = recent_requests.iter().map(|req| req.duration_ms).sum();
        let avg_duration_ms = if total_requests > 0 {
            total_duration as f64 / total_requests as f64
        } else {
            0.0
        };

        let error_count = recent_requests
            .iter()
            .filter(|req| req.status_code >= 400)
            .count() as i64;
        let success_count = total_requests - error_count;

        Ok(AnalyticsStats {
            total_requests,
            unique_users,
            avg_duration_ms,
            error_count,
            success_count,
        })
    }

    pub async fn get_top_paths(
        &self,
        hours: i32,
        limit: i64,
    ) -> Result<Vec<PathStats>, Box<dyn std::error::Error>> {
        let data = self.data.read().unwrap();
        let cutoff = OffsetDateTime::now_utc() - time::Duration::hours(hours as i64);

        let mut path_stats: HashMap<String, (i64, i64)> = HashMap::new();

        for req in data.iter().filter(|req| req.timestamp >= cutoff) {
            let entry = path_stats.entry(req.path.clone()).or_insert((0, 0));
            entry.0 += 1; // count
            entry.1 += req.duration_ms; // total duration
        }

        let mut result: Vec<PathStats> = path_stats
            .into_iter()
            .map(|(path, (count, total_duration))| PathStats {
                path,
                request_count: count,
                avg_duration_ms: if count > 0 {
                    total_duration as f64 / count as f64
                } else {
                    0.0
                },
            })
            .collect();

        result.sort_by(|a, b| b.request_count.cmp(&a.request_count));
        result.truncate(limit as usize);

        Ok(result)
    }

    pub async fn cleanup_old_data(&self, days: i32) -> Result<u64, Box<dyn std::error::Error>> {
        let mut data = self.data.write().unwrap();
        let cutoff = OffsetDateTime::now_utc() - time::Duration::days(days as i64);
        let original_len = data.len();
        data.retain(|req| req.timestamp >= cutoff);
        let removed = original_len - data.len();
        Ok(removed as u64)
    }
}

impl Default for MemoryAnalyticsStore {
    fn default() -> Self {
        Self::new()
    }
}

/// PostgreSQL analytics storage
pub struct PostgresAnalyticsStore {
    pool: PgPool,
}

impl PostgresAnalyticsStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn log_request(
        &self,
        analytics: RequestAnalytics,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            r#"
            INSERT INTO request_analytics
            (id, request_id, timestamp, user_id, method, path, status_code,
             duration_ms, user_agent, ip_address, request_data, response_size,
             error_message, trace_id, span_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            analytics.id,
            analytics.request_id,
            analytics.timestamp,
            analytics.user_id,
            analytics.method,
            analytics.path,
            analytics.status_code as i32,
            analytics.duration_ms as i32,
            analytics.user_agent,
            analytics.ip_address,
            analytics.request_data,
            analytics.response_size,
            analytics.error_message,
            analytics.trace_id,
            analytics.span_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_requests(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RequestAnalytics>, Box<dyn std::error::Error>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, request_id, timestamp, user_id, method, path, status_code,
                   duration_ms, user_agent, ip_address, request_data, response_size,
                   error_message, trace_id, span_id
            FROM request_analytics
            WHERE user_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| RequestAnalytics {
                id: row.id,
                request_id: row.request_id,
                timestamp: row.timestamp,
                user_id: row.user_id,
                method: row.method,
                path: row.path,
                status_code: row.status_code as u16,
                duration_ms: row.duration_ms.unwrap_or(0) as i64,
                user_agent: row.user_agent,
                ip_address: row.ip_address.unwrap_or_default(),
                request_data: row.request_data,
                response_size: row.response_size,
                error_message: row.error_message,
                trace_id: row.trace_id,
                span_id: row.span_id,
            })
            .collect();

        Ok(results)
    }

    pub async fn get_stats(
        &self,
        hours: i32,
    ) -> Result<AnalyticsStats, Box<dyn std::error::Error>> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_requests,
                COUNT(DISTINCT user_id) as unique_users,
                AVG(duration_ms) as avg_duration_ms,
                COUNT(*) FILTER (WHERE status_code >= 400) as error_count,
                COUNT(*) FILTER (WHERE status_code < 400) as success_count
            FROM request_analytics
            WHERE timestamp >= NOW() - INTERVAL '1 hour' * $1
            "#,
            hours as f64
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AnalyticsStats {
            total_requests: row.total_requests.unwrap_or(0),
            unique_users: row.unique_users.unwrap_or(0),
            avg_duration_ms: row.avg_duration_ms.and_then(|d| d.to_f64()).unwrap_or(0.0),
            error_count: row.error_count.unwrap_or(0),
            success_count: row.success_count.unwrap_or(0),
        })
    }

    pub async fn get_top_paths(
        &self,
        hours: i32,
        limit: i64,
    ) -> Result<Vec<PathStats>, Box<dyn std::error::Error>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                path,
                COUNT(*) as request_count,
                AVG(duration_ms) as avg_duration_ms
            FROM request_analytics
            WHERE timestamp >= NOW() - INTERVAL '1 hour' * $1
            GROUP BY path
            ORDER BY request_count DESC
            LIMIT $2
            "#,
            hours as f64,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let results = rows
            .into_iter()
            .map(|row| PathStats {
                path: row.path,
                request_count: row.request_count.unwrap_or(0),
                avg_duration_ms: row.avg_duration_ms.and_then(|d| d.to_f64()).unwrap_or(0.0),
            })
            .collect();

        Ok(results)
    }

    pub async fn cleanup_old_data(&self, days: i32) -> Result<u64, Box<dyn std::error::Error>> {
        let result = sqlx::query!(
            "DELETE FROM request_analytics WHERE timestamp < NOW() - INTERVAL '1 day' * $1",
            days as f64
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}

/// Analytics storage enum - replaces trait object pattern
#[derive(Clone)]
pub enum AnalyticsStore {
    Memory(Arc<MemoryAnalyticsStore>),
    Postgres(Arc<PostgresAnalyticsStore>),
}

impl AnalyticsStore {
    pub fn new_memory() -> Self {
        Self::Memory(Arc::new(MemoryAnalyticsStore::new()))
    }

    pub fn new_postgres(pool: PgPool) -> Self {
        Self::Postgres(Arc::new(PostgresAnalyticsStore::new(pool)))
    }

    pub async fn log_request(
        &self,
        analytics: RequestAnalytics,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::Memory(store) => store.log_request(analytics).await,
            Self::Postgres(store) => store.log_request(analytics).await,
        }
    }

    pub async fn get_user_requests(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RequestAnalytics>, Box<dyn std::error::Error>> {
        match self {
            Self::Memory(store) => store.get_user_requests(user_id, limit).await,
            Self::Postgres(store) => store.get_user_requests(user_id, limit).await,
        }
    }

    pub async fn get_stats(
        &self,
        hours: i32,
    ) -> Result<AnalyticsStats, Box<dyn std::error::Error>> {
        match self {
            Self::Memory(store) => store.get_stats(hours).await,
            Self::Postgres(store) => store.get_stats(hours).await,
        }
    }

    pub async fn get_top_paths(
        &self,
        hours: i32,
        limit: i64,
    ) -> Result<Vec<PathStats>, Box<dyn std::error::Error>> {
        match self {
            Self::Memory(store) => store.get_top_paths(hours, limit).await,
            Self::Postgres(store) => store.get_top_paths(hours, limit).await,
        }
    }

    pub async fn cleanup_old_data(&self, days: i32) -> Result<u64, Box<dyn std::error::Error>> {
        match self {
            Self::Memory(store) => store.cleanup_old_data(days).await,
            Self::Postgres(store) => store.cleanup_old_data(days).await,
        }
    }
}

/// Analytics service using enum-based storage
#[derive(Clone)]
pub struct AnalyticsService {
    store: AnalyticsStore,
}

impl AnalyticsService {
    pub fn new_memory() -> Self {
        Self {
            store: AnalyticsStore::new_memory(),
        }
    }

    pub fn new_postgres(pool: PgPool) -> Self {
        Self {
            store: AnalyticsStore::new_postgres(pool),
        }
    }

    pub async fn log_request(
        &self,
        analytics: RequestAnalytics,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.store.log_request(analytics).await
    }

    pub async fn get_user_requests(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RequestAnalytics>, Box<dyn std::error::Error>> {
        self.store.get_user_requests(user_id, limit).await
    }

    pub async fn get_stats(
        &self,
        hours: i32,
    ) -> Result<AnalyticsStats, Box<dyn std::error::Error>> {
        self.store.get_stats(hours).await
    }

    pub async fn get_top_paths(
        &self,
        hours: i32,
        limit: i64,
    ) -> Result<Vec<PathStats>, Box<dyn std::error::Error>> {
        self.store.get_top_paths(hours, limit).await
    }

    pub async fn cleanup_old_data(&self, days: i32) -> Result<u64, Box<dyn std::error::Error>> {
        self.store.cleanup_old_data(days).await
    }
}

/// Session storage enum
#[derive(Clone)]
pub enum SessionStore {
    Memory(tower_sessions::MemoryStore),
    Postgres(tower_sessions_sqlx_store::PostgresStore),
}

impl SessionStore {
    pub fn new_memory() -> Self {
        Self::Memory(tower_sessions::MemoryStore::default())
    }

    pub async fn new_postgres(pool: PgPool) -> Result<Self, Box<dyn std::error::Error>> {
        let store = tower_sessions_sqlx_store::PostgresStore::new(pool)
            .with_schema_name("public")?
            .with_table_name("tower_sessions")?;

        store.migrate().await?;
        Ok(Self::Postgres(store))
    }
}

/// Analytics builder for constructing RequestAnalytics
#[derive(Debug)]
pub struct AnalyticsBuilder {
    request_id: String,
    user_id: Option<Uuid>,
    method: String,
    path: String,
    status_code: u16,
    duration_ms: i64,
    user_agent: Option<String>,
    ip_address: String,
    request_data: Option<JsonValue>,
    response_size: Option<i64>,
    error_message: Option<String>,
    trace_id: Option<String>,
    span_id: Option<String>,
}

impl AnalyticsBuilder {
    pub fn new(
        request_id: String,
        method: String,
        path: String,
        status_code: u16,
        ip_address: String,
    ) -> Self {
        Self {
            request_id,
            user_id: None,
            method,
            path,
            status_code,
            duration_ms: 0,
            user_agent: None,
            ip_address,
            request_data: None,
            response_size: None,
            error_message: None,
            trace_id: None,
            span_id: None,
        }
    }

    pub fn user_id(mut self, user_id: Option<Uuid>) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn duration_ms(mut self, duration_ms: i64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn user_agent(mut self, user_agent: Option<String>) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = ip_address;
        self
    }

    pub fn build(self) -> RequestAnalytics {
        RequestAnalytics {
            id: Uuid::new_v4(),
            request_id: self.request_id,
            timestamp: OffsetDateTime::now_utc(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_builder() {
        let analytics = AnalyticsBuilder::new(
            "req-123".to_string(),
            "GET".to_string(),
            "/api/test".to_string(),
            200,
            "127.0.0.1".to_string(),
        )
        .user_id(Some(Uuid::new_v4()))
        .duration_ms(150)
        .user_agent(Some("test-agent".to_string()))
        .build();

        assert_eq!(analytics.request_id, "req-123");
        assert_eq!(analytics.method, "GET");
        assert_eq!(analytics.path, "/api/test");
        assert_eq!(analytics.status_code, 200);
        assert_eq!(analytics.duration_ms, 150);
    }

    #[tokio::test]
    async fn test_memory_analytics_store() {
        let store = AnalyticsStore::new_memory();

        let analytics = AnalyticsBuilder::new(
            "req-123".to_string(),
            "GET".to_string(),
            "/api/test".to_string(),
            200,
            "127.0.0.1".to_string(),
        )
        .build();

        store.log_request(analytics).await.unwrap();
        let stats = store.get_stats(24).await.unwrap();
        assert_eq!(stats.total_requests, 1);
    }
}
