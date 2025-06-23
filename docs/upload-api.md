# Large File Upload API

This document describes the large file upload API for handling files >10MB. This feature is restricted to admin users only.

## Overview

The upload system stores large files (>10MB) to disk in the configured upload directory and creates database records with `local_path` references instead of storing binary data in the `data` column.

## Configuration

Add upload directory configuration to your config file:

```jsonc
{
  "static_files": {
    "public_directory": "assets/public",
    "private_directory": "assets/private",
    "assets_directory": "assets",
    "upload_directory": "assets/private/uploads", // <- Add this
  },
}
```

## API Endpoints

### Upload Large File

**POST** `/api/upload`

Uploads a large file (>10MB) and stores it to disk.

**Requirements:**

- User must have `Admin` role (enforced by middleware)
- File size must be ≥ 10MB and ≤ 1GB
- Content-Type: `multipart/form-data`

**Request Body:**

- `metadata` (JSON): Upload metadata
- `file` (binary): File data

**Metadata JSON Structure:**

```json
{
  "filename": "large-video.mp4",
  "mime_type": "video/mp4",
  "sha256": "abc123...", // 64-char SHA256 hash
  "size": 52428800, // File size in bytes
  "metadata": {
    // Optional additional metadata
    "description": "My video file",
    "tags": ["work", "presentation"]
  }
}
```

**Response (201 Created):**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "local_path": "private/uploads/abc123...def.mp4",
  "sha256": "abc123...def",
  "size": 52428800,
  "mime_type": "video/mp4",
  "created_at": "2024-01-15T10:30:00Z"
}
```

**Error Responses:**

- `400 Bad Request`: Invalid request data or file validation failed
- `401 Unauthorized`: User not authenticated
- `403 Forbidden`: Non-admin user attempted upload (handled by middleware)
- `409 Conflict`: File with same SHA256 already exists
- `413 Payload Too Large`: File exceeds 1GB limit

### Get Upload Info

**GET** `/api/upload/:id`

Retrieves information about an uploaded file.

**Requirements:**

- User must be authenticated

**Response (200 OK):**

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "local_path": "private/uploads/abc123...def.mp4",
  "sha256": "abc123...def",
  "size": 52428800,
  "mime_type": "video/mp4",
  "source_client_id": "admin_upload_user123",
  "metadata": {
    "description": "My video file"
  },
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

### List Uploads

**GET** `/api/uploads`

Lists all uploaded files with pagination.

**Requirements:**

- User must be authenticated

**Query Parameters:**

- `limit` (optional): Maximum number of results (max 100, default varies)
- `offset` (optional): Number of results to skip (default 0)

**Response (200 OK):**

```json
{
  "uploads": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "local_path": "private/uploads/abc123...def.mp4",
      "sha256": "abc123...def",
      "size": 52428800,
      "mime_type": "video/mp4",
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total_count": 1,
  "limit": 50,
  "offset": 0
}
```

### Delete Upload

**DELETE** `/api/upload/:id`

Deletes an uploaded file from both disk and database.

**Requirements:**

- User must have `Admin` role

**Response (204 No Content):**
Empty response body on successful deletion.

**Error Responses:**

- `400 Bad Request`: Cannot delete non-uploaded media blob
- `401 Unauthorized`: User not authenticated
- `403 Forbidden`: Non-admin user attempted deletion (handled by middleware)
- `404 Not Found`: Upload not found

## File Access

Uploaded files are accessible through the static file server at:
`/private/uploads/filename.ext`

Since they're in the private directory, authentication is required to access them.

## File Storage

- Files are stored with SHA256-based names to avoid conflicts
- Original extensions are preserved when possible
- Files are stored in the configured `upload_directory`
- Relative paths are stored in the database (e.g., `private/uploads/abc123.mp4`)

## Security Considerations

1. **Role-based Access**:
   - Upload and delete operations require admin role (enforced by `require_admin` middleware)
   - View operations (GET endpoints) require authentication only (any authenticated user)
2. **Size Limits**: 10MB minimum, 1GB maximum
3. **Hash Verification**: SHA256 hash is verified on upload
4. **Path Safety**: Filenames are sanitized to prevent path traversal
5. **Deduplication**: Duplicate files (same SHA256) are rejected
6. **Authentication Required**: File access requires authentication

## Example Usage

### Upload with curl

```bash
# First, prepare metadata
cat > metadata.json << EOF
{
  "filename": "presentation.mp4",
  "mime_type": "video/mp4",
  "sha256": "$(sha256sum large-file.mp4 | cut -d' ' -f1)",
  "size": $(stat -c%s large-file.mp4),
  "metadata": {"description": "Company presentation"}
}
EOF

# Upload file
curl -X POST http://localhost:3000/api/upload \
  -H "Cookie: webauthnrs=YOUR_SESSION_COOKIE" \
  -F "metadata=@metadata.json;type=application/json" \
  -F "file=@large-file.mp4;type=video/mp4"
```

### JavaScript Upload

```javascript
async function uploadLargeFile(file) {
  // Calculate SHA256 hash
  const arrayBuffer = await file.arrayBuffer();
  const hashBuffer = await crypto.subtle.digest("SHA-256", arrayBuffer);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const sha256 = hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");

  // Prepare metadata
  const metadata = {
    filename: file.name,
    mime_type: file.type,
    sha256: sha256,
    size: file.size,
    metadata: {
      lastModified: new Date(file.lastModified).toISOString(),
    },
  };

  // Create form data
  const formData = new FormData();
  formData.append("metadata", JSON.stringify(metadata));
  formData.append("file", file);

  // Upload
  const response = await fetch("/api/upload", {
    method: "POST",
    body: formData,
    credentials: "include", // Include session cookie
  });

  if (!response.ok) {
    throw new Error(`Upload failed: ${response.status}`);
  }

  return await response.json();
}
```

## File Size Considerations

- **Small files (< 10MB)**: Continue using the existing WebSocket-based system with BYTEA storage
- **Large files (≥ 10MB)**: Use this upload API with disk storage
- **Maximum size**: 1GB per file
- **Storage location**: Configurable via `upload_directory` setting

## Migration Notes

Existing small files stored as BYTEA will continue to work normally. The system supports both storage methods:

- `data` field populated = small file stored as BYTEA
- `local_path` field populated = large file stored on disk

Both types can coexist in the same `media_blobs` table.
