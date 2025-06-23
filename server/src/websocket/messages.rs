//! WebSocket message types and serialization
//!
//! Defines the message format for WebSocket communication between
//! client and server, with serde for JSON serialization.

use crate::media::MediaBlob;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Messages sent from client to server
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    /// Client sends a ping to check connection
    Ping,
    /// Client requests list of media blobs
    GetMediaBlobs {
        limit: Option<u32>,
        offset: Option<u32>,
    },
    /// Client uploads a new media blob
    UploadMediaBlob { blob: MediaBlob },
    /// Client requests specific media blob by ID
    GetMediaBlob { id: Uuid },
    /// Client requests media blob data by ID
    GetMediaBlobData { id: Uuid },
}

/// Messages sent from server to client
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketResponse {
    /// Server greeting on connection
    Welcome {
        message: String,
        user_id: Option<Uuid>,
        connection_id: String,
    },
    /// Server responds to ping
    Pong,
    /// Server sends list of media blobs
    MediaBlobs {
        blobs: Vec<MediaBlob>,
        total_count: u32,
    },
    /// Server sends single media blob
    MediaBlob { blob: MediaBlob },
    /// Server sends media blob data (binary content)
    MediaBlobData {
        id: Uuid,
        data: Vec<u8>,
        mime: Option<String>,
    },
    /// Server sends error message
    Error {
        message: String,
        code: Option<String>,
    },
    /// Server sends connection status update
    ConnectionStatus { connected: bool, user_count: u32 },
}

impl WebSocketMessage {
    /// Parse a WebSocket message from JSON text
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Serialize message to JSON text
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl std::fmt::Debug for WebSocketMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketMessage::Ping => f.debug_struct("Ping").finish(),
            WebSocketMessage::GetMediaBlobs { limit, offset } => f
                .debug_struct("GetMediaBlobs")
                .field("limit", limit)
                .field("offset", offset)
                .finish(),
            WebSocketMessage::UploadMediaBlob { blob } => f
                .debug_struct("UploadMediaBlob")
                .field("blob_id", &blob.id)
                .field("blob_size", &blob.size)
                .field("blob_mime", &blob.mime)
                .field("blob_sha256_prefix", &format!("{}...", &blob.sha256[..8]))
                .finish(),
            WebSocketMessage::GetMediaBlob { id } => {
                f.debug_struct("GetMediaBlob").field("id", id).finish()
            }
            WebSocketMessage::GetMediaBlobData { id } => {
                f.debug_struct("GetMediaBlobData").field("id", id).finish()
            }
        }
    }
}

impl WebSocketResponse {
    /// Serialize response to JSON text
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Create a welcome message
    pub fn welcome(user_id: Option<Uuid>, connection_id: String) -> Self {
        Self::Welcome {
            message: "Connected to WebSocket server".to_string(),
            user_id,
            connection_id,
        }
    }

    /// Create an error response
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            code: None,
        }
    }

    /// Create an error response with code
    pub fn error_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            code: Some(code.into()),
        }
    }
}

impl std::fmt::Debug for WebSocketResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketResponse::Welcome {
                message,
                user_id,
                connection_id,
            } => f
                .debug_struct("Welcome")
                .field("message", message)
                .field("user_id", user_id)
                .field("connection_id", connection_id)
                .finish(),
            WebSocketResponse::Pong => f.debug_struct("Pong").finish(),
            WebSocketResponse::MediaBlobs { blobs, total_count } => f
                .debug_struct("MediaBlobs")
                .field("blob_count", &blobs.len())
                .field("total_count", total_count)
                .finish(),
            WebSocketResponse::MediaBlob { blob } => f
                .debug_struct("MediaBlob")
                .field("blob_id", &blob.id)
                .field("blob_size", &blob.size)
                .field("blob_mime", &blob.mime)
                .field("blob_sha256_prefix", &format!("{}...", &blob.sha256[..8]))
                .finish(),
            WebSocketResponse::MediaBlobData { id, data, mime } => f
                .debug_struct("MediaBlobData")
                .field("id", id)
                .field("data_size", &data.len())
                .field("mime", mime)
                .finish(),
            WebSocketResponse::Error { message, code } => f
                .debug_struct("Error")
                .field("message", message)
                .field("code", code)
                .finish(),
            WebSocketResponse::ConnectionStatus {
                connected,
                user_count,
            } => f
                .debug_struct("ConnectionStatus")
                .field("connected", connected)
                .field("user_count", user_count)
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use super::*;

    #[test]
    fn test_websocket_message_serialization() {
        let msg = WebSocketMessage::Ping;
        let json = msg.to_json().unwrap();
        assert!(json.contains("Ping"));

        let parsed = WebSocketMessage::from_json(&json).unwrap();
        matches!(parsed, WebSocketMessage::Ping);
    }

    #[test]
    fn test_websocket_response_serialization() {
        let response = WebSocketResponse::welcome(None, "test-123".to_string());
        let json = response.to_json().unwrap();
        assert!(json.contains("Welcome"));
        assert!(json.contains("test-123"));
    }

    #[test]
    fn test_media_blob_serialization() {
        let blob = MediaBlob {
            id: Uuid::new_v4(),
            data: None,
            sha256: "abc123".to_string(),
            size: Some(1024),
            mime: Some("image/png".to_string()),
            source_client_id: Some("client-1".to_string()),
            local_path: Some("/path/to/file.png".to_string()),
            metadata: serde_json::json!({"width": 800, "height": 600}),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };

        let json = serde_json::to_string(&blob).unwrap();
        let parsed: MediaBlob = serde_json::from_str(&json).unwrap();
        assert_eq!(blob.sha256, parsed.sha256);
        assert_eq!(blob.size, parsed.size);
    }
}
