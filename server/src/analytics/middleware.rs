use crate::error::WebauthnError;
use crate::storage::{AnalyticsBuilder, AnalyticsService};
use axum::{
    extract::{Extension, Request},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use tower_sessions::Session;

/// Analytics middleware that logs all requests to the database
pub async fn analytics_middleware(
    session: Session,
    Extension(analytics_service): Extension<AnalyticsService>,
    request: Request,
    next: Next,
) -> Result<Response, WebauthnError> {
    // Extract basic request information
    let method = request.method().to_string();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let user_agent = extract_user_agent(request.headers());

    // Get start time
    let start_time = std::time::Instant::now();

    // Try to get user_id from session
    let user_id = session.get::<uuid::Uuid>("user_id").await.ok().flatten();

    // Run the request
    let response = next.run(request).await;

    // Extract response information
    let status_code = response.status().as_u16();
    let duration_ms = start_time.elapsed().as_millis() as i64;

    // Build basic analytics record
    let analytics = AnalyticsBuilder::new(
        uuid::Uuid::new_v4().to_string(),
        method.clone(),
        path.clone(),
        status_code,
        "127.0.0.1".to_string(), // Simplified for now
    )
    .user_id(user_id)
    .duration_ms(duration_ms)
    .user_agent(user_agent.clone())
    .build();

    // Log analytics to storage backend (spawn task to not block response)
    let service = analytics_service.clone();
    let analytics_clone = analytics.clone();
    tokio::spawn(async move {
        if let Err(e) = service.log_request(analytics_clone).await {
            tracing::error!("Failed to log analytics: {}", e);
        }
    });

    // Traditional security logging (keep for now)
    tracing::info!(
        "Analytics: {} {} - Status: {} - Duration: {}ms - User: {}",
        method,
        path,
        status_code,
        duration_ms,
        user_id
            .map(|u| u.to_string())
            .unwrap_or_else(|| "anonymous".to_string())
    );

    Ok(response)
}

/// Legacy security logging middleware (kept for compatibility)
pub async fn security_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = extract_user_agent(request.headers());

    let response = next.run(request).await;

    // Log the request with response status
    tracing::info!(
        "Legacy Log: {} {} - Status: {} - User-Agent: {}",
        method,
        uri,
        response.status(),
        user_agent.unwrap_or_else(|| "unknown".to_string())
    );

    response
}

/// Extract user agent from headers
fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_user_agent() {
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", "test-agent".parse().unwrap());

        let user_agent = extract_user_agent(&headers);
        assert_eq!(user_agent, Some("test-agent".to_string()));
    }

    #[test]
    fn test_extract_user_agent_missing() {
        let headers = HeaderMap::new();
        let user_agent = extract_user_agent(&headers);
        assert_eq!(user_agent, None);
    }
}
