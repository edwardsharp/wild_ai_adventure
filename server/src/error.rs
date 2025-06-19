use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebauthnError {
    #[error("unknown webauthn error")]
    Unknown,
    #[error("Corrupt Session")]
    CorruptSession,
    #[error("User Not Found")]
    UserNotFound,
    #[error("User Has No Credentials")]
    UserHasNoCredentials,
    #[error("Invalid Invite Code")]
    InvalidInviteCode,
    #[error("User Already Exists")]
    UserAlreadyExists,
    #[error("Database Error")]
    DatabaseError,
    #[error("Deserialising Session failed: {0}")]
    InvalidSessionState(#[from] tower_sessions::session::Error),
    #[error("Database operation failed: {0}")]
    SqlxError(#[from] sqlx::Error),
}
impl IntoResponse for WebauthnError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            WebauthnError::CorruptSession => (StatusCode::BAD_REQUEST, "Corrupt Session"),
            WebauthnError::UserNotFound => (StatusCode::NOT_FOUND, "User Not Found"),
            WebauthnError::Unknown => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown Error"),
            WebauthnError::UserHasNoCredentials => {
                (StatusCode::BAD_REQUEST, "User Has No Credentials")
            }
            WebauthnError::InvalidInviteCode => {
                (StatusCode::BAD_REQUEST, "Invalid or expired invite code")
            }
            WebauthnError::UserAlreadyExists => (StatusCode::CONFLICT, "Username already exists"),
            WebauthnError::DatabaseError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            WebauthnError::InvalidSessionState(_) => {
                (StatusCode::BAD_REQUEST, "Invalid session state")
            }
            WebauthnError::SqlxError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database operation failed",
            ),
        };

        // its often easiest to implement `IntoResponse` by calling other implementations
        (status, body).into_response()
    }
}
