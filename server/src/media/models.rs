//! Media blob data models
//!
//! This module defines the data structures for media blobs stored in the database
//! and used throughout the WebSocket system for file sharing.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// Media blob data structure matching the database schema
#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct MediaBlob {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<u8>>, // BYTEA - often omitted in responses for size
    pub sha256: String,
    pub size: Option<i64>,
    pub mime: Option<String>,
    pub source_client_id: Option<String>,
    pub local_path: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value, // JSONB
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

/// Parameters for creating a new media blob
#[derive(Clone, Serialize, Deserialize)]
pub struct CreateMediaBlob {
    pub data: Option<Vec<u8>>,
    pub sha256: String,
    pub size: Option<i64>,
    pub mime: Option<String>,
    pub source_client_id: Option<String>,
    pub local_path: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Parameters for querying media blobs
#[derive(Debug, Clone, Default)]
pub struct MediaBlobQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sha256: Option<String>,
    pub source_client_id: Option<String>,
    pub mime_pattern: Option<String>,
}

/// Media blob statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaBlobStats {
    pub total_count: i64,
    pub total_size: Option<i64>,
    pub unique_sha256_count: i64,
    pub mime_type_distribution: Vec<MimeTypeCount>,
}

/// Count of blobs by MIME type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimeTypeCount {
    pub mime_type: Option<String>,
    pub count: i64,
}

impl MediaBlob {
    /// Create a new MediaBlob from CreateMediaBlob parameters
    pub fn new(params: CreateMediaBlob) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            data: params.data,
            sha256: params.sha256,
            size: params.size,
            mime: params.mime,
            source_client_id: params.source_client_id,
            local_path: params.local_path,
            metadata: params.metadata,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get the blob without the binary data (for efficient serialization)
    pub fn without_data(&self) -> Self {
        let mut blob = self.clone();
        blob.data = None;
        blob
    }

    /// Check if this blob has binary data
    pub fn has_data(&self) -> bool {
        self.data.is_some() && !self.data.as_ref().unwrap().is_empty()
    }

    /// Get the file extension from MIME type
    pub fn file_extension(&self) -> Option<&str> {
        match self.mime.as_deref() {
            Some("image/jpeg") => Some("jpg"),
            Some("image/png") => Some("png"),
            Some("image/gif") => Some("gif"),
            Some("image/webp") => Some("webp"),
            Some("video/mp4") => Some("mp4"),
            Some("video/webm") => Some("webm"),
            Some("video/quicktime") => Some("mov"),
            Some("audio/mpeg") => Some("mp3"),
            Some("audio/mp3") => Some("mp3"),
            Some("audio/wav") => Some("wav"),
            Some("audio/wave") => Some("wav"),
            Some("audio/ogg") => Some("ogg"),
            Some("audio/aac") => Some("aac"),
            Some("audio/flac") => Some("flac"),
            Some("audio/m4a") => Some("m4a"),
            Some("audio/webm") => Some("webm"),
            Some("application/pdf") => Some("pdf"),
            Some("text/plain") => Some("txt"),
            Some("application/json") => Some("json"),
            _ => None,
        }
    }

    /// Validate that required fields are present
    pub fn validate(&self) -> Result<(), String> {
        if self.sha256.is_empty() {
            return Err("SHA256 hash is required".to_string());
        }

        if self.sha256.len() != 64 {
            return Err("SHA256 hash must be 64 characters".to_string());
        }

        // Check that we have either data or local_path
        if !self.has_data() && self.local_path.is_none() {
            return Err("Either data or local_path must be provided".to_string());
        }

        // Check file size limit (10MB = 10 * 1024 * 1024 bytes)
        const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
        if let Some(ref data) = self.data {
            if data.len() > MAX_FILE_SIZE {
                return Err(format!(
                    "File size {} bytes exceeds maximum allowed size of {} bytes (10MB)",
                    data.len(),
                    MAX_FILE_SIZE
                ));
            }
        }

        // Also check the size field if provided
        if let Some(size) = self.size {
            if size > MAX_FILE_SIZE as i64 {
                return Err(format!(
                    "File size {} bytes exceeds maximum allowed size of {} bytes (10MB)",
                    size, MAX_FILE_SIZE
                ));
            }
        }

        Ok(())
    }
}

impl std::fmt::Debug for MediaBlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MediaBlob")
            .field("id", &self.id)
            .field("data_size", &self.data.as_ref().map(|d| d.len()))
            .field("sha256", &format!("{}...", &self.sha256[..8]))
            .field("size", &self.size)
            .field("mime", &self.mime)
            .field("source_client_id", &self.source_client_id)
            .field("local_path", &self.local_path)
            .field("metadata", &self.metadata)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

impl CreateMediaBlob {
    /// Validate creation parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.sha256.is_empty() {
            return Err("SHA256 hash is required".to_string());
        }

        if self.sha256.len() != 64 {
            return Err("SHA256 hash must be 64 characters".to_string());
        }

        // Check that we have either data or local_path
        let has_data = self.data.is_some() && !self.data.as_ref().unwrap().is_empty();
        if !has_data && self.local_path.is_none() {
            return Err("Either data or local_path must be provided".to_string());
        }

        // Check file size limit (10MB = 10 * 1024 * 1024 bytes)
        const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
        if let Some(ref data) = self.data {
            if data.len() > MAX_FILE_SIZE {
                return Err(format!(
                    "File size {} bytes exceeds maximum allowed size of {} bytes (10MB)",
                    data.len(),
                    MAX_FILE_SIZE
                ));
            }
        }

        // Also check the size field if provided
        if let Some(size) = self.size {
            if size > MAX_FILE_SIZE as i64 {
                return Err(format!(
                    "File size {} bytes exceeds maximum allowed size of {} bytes (10MB)",
                    size, MAX_FILE_SIZE
                ));
            }
        }

        Ok(())
    }
}

impl std::fmt::Debug for CreateMediaBlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateMediaBlob")
            .field("data_size", &self.data.as_ref().map(|d| d.len()))
            .field("sha256", &format!("{}...", &self.sha256[..8]))
            .field("size", &self.size)
            .field("mime", &self.mime)
            .field("source_client_id", &self.source_client_id)
            .field("local_path", &self.local_path)
            .field("metadata", &self.metadata)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_blob_new() {
        let params = CreateMediaBlob {
            data: Some(vec![1, 2, 3, 4]),
            sha256: "a".repeat(64),
            size: Some(4),
            mime: Some("image/png".to_string()),
            source_client_id: Some("test-client".to_string()),
            local_path: None,
            metadata: serde_json::json!({"test": true}),
        };

        let blob = MediaBlob::new(params);
        assert_eq!(blob.sha256.len(), 64);
        assert_eq!(blob.size, Some(4));
        assert!(blob.has_data());
    }

    #[test]
    fn test_media_blob_without_data() {
        let blob = MediaBlob::new(CreateMediaBlob {
            data: Some(vec![1, 2, 3, 4]),
            sha256: "a".repeat(64),
            size: Some(4),
            mime: Some("image/png".to_string()),
            source_client_id: None,
            local_path: None,
            metadata: serde_json::Value::Null,
        });

        assert!(blob.has_data());

        let without_data = blob.without_data();
        assert!(!without_data.has_data());
        assert_eq!(blob.sha256, without_data.sha256);
    }

    #[test]
    fn test_file_extension() {
        let blob = MediaBlob::new(CreateMediaBlob {
            data: None,
            sha256: "a".repeat(64),
            size: None,
            mime: Some("image/png".to_string()),
            source_client_id: None,
            local_path: Some("/path/to/file".to_string()),
            metadata: serde_json::Value::Null,
        });

        assert_eq!(blob.file_extension(), Some("png"));
    }

    #[test]
    fn test_validation() {
        // Valid blob
        let valid_blob = MediaBlob::new(CreateMediaBlob {
            data: Some(vec![1, 2, 3]),
            sha256: "a".repeat(64),
            size: Some(3),
            mime: Some("image/png".to_string()),
            source_client_id: None,
            local_path: None,
            metadata: serde_json::Value::Null,
        });
        assert!(valid_blob.validate().is_ok());

        // Invalid SHA256
        let mut invalid_blob = valid_blob.clone();
        invalid_blob.sha256 = "short".to_string();
        assert!(invalid_blob.validate().is_err());

        // No data or path
        let mut no_data_blob = valid_blob.clone();
        no_data_blob.data = None;
        no_data_blob.local_path = None;
        assert!(no_data_blob.validate().is_err());
    }
}
