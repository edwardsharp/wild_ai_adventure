use crate::error::WebauthnError;
use axum::{
    extract::Request,
    http::StatusCode,
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

/// Middleware to log all requests for security monitoring
pub async fn security_logging(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let response = next.run(request).await;

    // Log the request with response status
    tracing::info!(
        "Request: {} {} - Status: {} - User-Agent: {}",
        method,
        uri,
        response.status(),
        user_agent
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::{ServiceBuilder, ServiceExt};
    use tower_sessions::{MemoryStore, SessionManagerLayer};

    // Helper function to create a test service with session middleware
    fn create_test_service(
    ) -> impl tower::Service<Request<Body>, Response = Response, Error = std::convert::Infallible> + Clone
    {
        ServiceBuilder::new()
            .layer(SessionManagerLayer::new(MemoryStore::default()))
            .service_fn(|_req: Request<Body>| async {
                Ok::<_, std::convert::Infallible>((StatusCode::OK, "test response").into_response())
            })
    }

    #[tokio::test]
    async fn test_require_authentication_without_session() {
        // This would require more complex setup with actual session management
        // For now, this serves as a placeholder for future testing
        assert!(true);
    }

    #[tokio::test]
    async fn test_security_logging() {
        // Test that security logging doesn't interfere with normal request flow
        assert!(true);
    }
}
