use crate::analytics::{AnalyticsBuilder, AnalyticsService};
use crate::error::WebauthnError;
use axum::{
    extract::{Extension, Request},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tower_sessions::Session;

/// Authentication middleware that checks if a user is logged in
/// Returns 401 Unauthorized if no valid session is found
pub async fn require_authentication(
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response, WebauthnError> {
    // Check if user_id exists in session (set during successful authentication)
    match session.get::<uuid::Uuid>("user_id").await? {
        Some(user_id) => {
            // User is authenticated, log the access and continue
            tracing::info!("Authenticated user {} accessing private content", user_id);
            Ok(next.run(request).await)
        }
        None => {
            // No valid session found
            tracing::warn!(
                "Unauthenticated access attempt to private content from {:?}",
                request.headers().get("user-agent")
            );

            // Return 401 with a helpful message
            Ok((
                StatusCode::UNAUTHORIZED,
                "Authentication required. Please log in to access this content.",
            )
                .into_response())
        }
    }
}

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
    let status_code = response.status().as_u16() as i32;
    let duration_ms = start_time.elapsed().as_millis();

    // Build basic analytics record
    let analytics = AnalyticsBuilder::new(
        uuid::Uuid::new_v4().to_string(),
        method.clone(),
        path.clone(),
        status_code,
    )
    .user_id(user_id)
    .duration_ms(duration_ms)
    .user_agent(user_agent.clone())
    .ip_address(Some("127.0.0.1".to_string())) // Simplified for now
    .build();

    // Log analytics to database (spawn task to not block response)
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

/// Optional authentication middleware that adds user info to request extensions
/// but doesn't block access for unauthenticated users
pub async fn optional_authentication(
    session: Session,
    mut request: Request,
    next: Next,
) -> Result<Response, WebauthnError> {
    // Try to get user_id from session
    if let Some(user_id) = session.get::<uuid::Uuid>("user_id").await? {
        // User is authenticated, add user_id to request extensions
        request.extensions_mut().insert(user_id);
        tracing::debug!("Authenticated user {} accessing content", user_id);
    } else {
        tracing::debug!("Anonymous user accessing content");
    }

    Ok(next.run(request).await)
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
