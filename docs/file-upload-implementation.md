# File Upload Implementation Summary

This document provides a comprehensive overview of the dual file upload system implemented in the axum tutorial project.

## 🎯 Overview

The implementation provides a sophisticated file upload system that intelligently routes files based on size:

- **Small files (<10MB)**: Uploaded via WebSocket, stored as BYTEA in PostgreSQL database
- **Large files (≥10MB)**: Uploaded via HTTP API, stored on disk with file path references

## 🏗️ Architecture

### Storage Strategy

```
┌─────────────────┬─────────────────┬─────────────────┐
│   File Size     │    Method       │    Storage      │
├─────────────────┼─────────────────┼─────────────────┤
│   < 10MB        │   WebSocket     │   Database      │
│                 │   (existing)    │   (BYTEA)       │
├─────────────────┼─────────────────┼─────────────────┤
│   ≥ 10MB        │   HTTP API      │   Disk          │
│                 │   (new)         │   (local_path)  │
└─────────────────┴─────────────────┴─────────────────┘
```

### Database Schema

The `media_blobs` table supports both storage methods:

```sql
CREATE TABLE media_blobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    data BYTEA,                 -- For small files (WebSocket)
    sha256 TEXT NOT NULL,
    size BIGINT,
    mime TEXT,
    source_client_id TEXT,
    local_path TEXT,            -- For large files (HTTP)
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);
```

**Storage Logic:**
- Small files: `data` populated, `local_path` is NULL
- Large files: `local_path` populated, `data` is NULL

## 🚀 Server Implementation

### HTTP Upload API (Large Files)

**Endpoints:**
- `POST /api/upload` - Upload large file (Admin only)
- `GET /api/upload/:id` - Get upload info (Authenticated users)
- `GET /api/uploads` - List uploads (Authenticated users)
- `DELETE /api/upload/:id` - Delete upload (Admin only)

**Key Features:**
- Admin-only access via `require_admin` middleware
- Multipart form handling (metadata + file)
- SHA256 verification
- Duplicate detection
- File size validation (10MB-1GB)
- Disk storage in `assets/private/uploads/`

### Configuration

```jsonc
{
  "static_files": {
    "upload_directory": "assets/private/uploads" // Configurable upload path
  }
}
```

### File Naming

Files are stored with SHA256-based names to prevent conflicts:
- Original: `vacation-video.mp4`
- Stored as: `abc123def456...789.mp4`
- Database path: `private/uploads/abc123def456...789.mp4`

## 📱 Client Implementation

### Renamed Components

**Old → New Naming:**
- `file-upload.ts` → `websocket-file-upload.ts` (WebSocket uploads)
- `FileUploadHandler` → `WebSocketFileUploadHandler`
- New: `file-upload.ts` (HTTP uploads)
- New: `FileUploadHandler` (HTTP uploads)

### TypeScript Types & Schemas

**HTTP Upload Types:**
```typescript
interface UploadRequest {
  filename: string;
  mime_type?: string;
  sha256: string;
  size: number;
  metadata: Record<string, any>;
}

interface UploadResponse {
  id: string;
  local_path: string;
  sha256: string;
  size: number;
  mime_type?: string;
  created_at: string;
}
```

**WebSocket Upload Types:**
```typescript
interface WebSocketUploadFile {
  file: File;
  id: string;
  progress: number;
  status: "pending" | "processing" | "uploading" | "completed" | "error";
}

interface WebSocketProcessedBlob {
  id: string;
  data: number[];          // BYTEA as number array
  sha256: string;
  // ... other fields
}
```

### Smart File Upload Component

**New Web Component: `<smart-file-upload>`**

```typescript
interface SmartFileUploadProps {
  baseUrl?: string;           // HTTP API base URL
  websocketConnection?: any;  // WebSocket client instance
  sizeThreshold?: number;     // 10MB default
  showDebug?: boolean;
  multiple?: boolean;
  accept?: string;
  disabled?: boolean;
}
```

**Features:**
- Drag & drop support
- Automatic routing based on file size
- Progress tracking for both methods
- Error handling and retry logic
- Visual indicators for storage type

### Enhanced Media Blob Manager

**New Methods:**
```typescript
class MediaBlobManager {
  constructor(baseUrl: string = "http://localhost:8080")

  private getStorageType(blob: MediaBlob): "database" | "disk"
  private getFileUrl(blob: MediaBlob): string | undefined
  updateBaseUrl(baseUrl: string): void
}
```

**Enhanced Display Info:**
```typescript
interface BlobDisplayInfo {
  // ... existing fields
  fileUrl?: string;                    // Full URL for disk files
  storageType: "database" | "disk";    // Storage method
}
```

## 🎨 UI/UX Features

### Unified Media Browser

The WebSocket demo now displays both storage types:

**Visual Indicators:**
- 💾 Database (small files)
- 🗄️ Disk (large files)

**Smart Actions:**
- Database files: Load Data, View, Download (via WebSocket)
- Disk files: Open File, Download (direct HTTP access)

### URL Generation

Large files get full URLs for direct access:
- Database path: `private/uploads/abc123.jpg`
- Full URL: `http://localhost:8080/private/uploads/abc123.jpg`

## 🔐 Security & Permissions

### Access Control Matrix

```
┌─────────────────┬─────────────────┬─────────────────┐
│   Operation     │   WebSocket     │   HTTP API      │
├─────────────────┼─────────────────┼─────────────────┤
│   Upload        │   Any User      │   Admin Only    │
│   View/List     │   Any User      │   Any User      │
│   Delete        │   N/A           │   Admin Only    │
│   Download      │   Any User      │   Any User*     │
└─────────────────┴─────────────────┴─────────────────┘
```
*Through static file server with authentication

### Security Features

1. **Role-based Access**: Admin middleware for sensitive operations
2. **File Validation**: Size limits, SHA256 verification, filename sanitization
3. **Path Safety**: Relative paths prevent directory traversal
4. **Authentication**: All operations require authenticated users
5. **Deduplication**: SHA256-based duplicate detection

## 📁 File Structure

### Server Files
```
server/src/
├── upload/
│   ├── mod.rs              # Module exports
│   ├── models.rs           # Upload data structures
│   ├── handlers.rs         # HTTP handlers
│   └── routes.rs           # Route definitions
├── media/
│   └── models.rs           # Enhanced with URL helpers
├── config.rs               # Added upload_directory
└── startup.rs              # Directory creation
```

### Client Files
```
client/js/src/lib/
├── file-upload.ts          # HTTP upload handler
├── websocket-file-upload.ts # WebSocket upload handler
└── media-blob-manager.ts   # Enhanced with URL support

client/js/src/web-components/
├── smart-file-upload.tsx   # Unified upload component
└── websocket-demo.tsx      # Enhanced media browser
```

### Assets
```
assets/
├── private/
│   ├── uploads/            # Large file storage
│   └── file-upload-demo.html # Demo page
└── index.html              # Added demo link
```

## 🧪 Testing & Demo

### Demo Page Features

**Location:** `/private/file-upload-demo.html`

**Components:**
1. **WebSocket Connection**: Real-time connection management
2. **Smart Upload**: Unified file upload with automatic routing
3. **Media Browser**: Browse both storage types
4. **Debug Logs**: Real-time logging and monitoring

**Test Scenarios:**
- Upload files < 10MB (WebSocket → Database)
- Upload files ≥ 10MB (HTTP → Disk, admin only)
- View mixed media library
- Access files via direct URLs
- Error handling and retry

## 🔄 Migration & Compatibility

### Backward Compatibility

✅ **Preserved:**
- Existing WebSocket upload functionality
- Small file storage in database
- All existing API endpoints
- Current authentication flow

✅ **Enhanced:**
- Media browser shows both storage types
- Smart routing based on file size
- Unified user interface

### Database Migration

No migration required! The `media_blobs` table already supported both storage methods via the `local_path` column.

## 🚀 Usage Examples

### Basic Upload (Client)
```typescript
// HTTP upload for large files
const httpUploader = new FileUploadHandler({
  baseUrl: "http://localhost:8080",
  minFileSize: 10 * 1024 * 1024 // 10MB
});

const result = await httpUploader.uploadFile(file, {
  description: "My large video file"
});

// WebSocket upload for small files
const wsUploader = new WebSocketFileUploadHandler({
  maxFileSize: 10 * 1024 * 1024 // 10MB
});

await wsUploader.addFiles([file]);
```

### Smart Component (HTML)
```html
<smart-file-upload
  base-url="http://localhost:8080"
  websocket-connection="wsClient"
  size-threshold="10485760"
  show-debug="true"
  multiple="true">
</smart-file-upload>
```

### Media Display (Enhanced)
```typescript
// Get display info with storage type and URL
const displayInfo = mediaManager.getBlobDisplayInfo(blob);

console.log(displayInfo.storageType); // "database" or "disk"
console.log(displayInfo.fileUrl);     // Full URL for disk files
```

## 📈 Benefits

### Performance
- Small files: Fast WebSocket transfer, immediate availability
- Large files: Efficient HTTP upload, direct file access
- No database bloat from large binary data

### Scalability
- Disk storage for large files reduces database size
- Direct file serving bypasses application layer
- Configurable storage thresholds

### Developer Experience
- Unified API for both upload methods
- Automatic routing based on file size
- Comprehensive error handling
- Type-safe TypeScript interfaces

### User Experience
- Seamless upload regardless of file size
- Progress tracking for all uploads
- Visual indicators for storage type
- Direct file access for large files

## 🔮 Future Enhancements

### Potential Improvements
1. **Cloud Storage**: S3/CloudFlare R2 integration for large files
2. **Streaming Uploads**: Chunked upload for very large files
3. **Compression**: Automatic compression for specific file types
4. **Thumbnails**: Server-side thumbnail generation for images
5. **CDN Integration**: Asset delivery optimization
6. **File Versioning**: Track file history and changes

### Configuration Extensions
```jsonc
{
  "upload": {
    "small_file_threshold": "10MB",
    "large_file_storage": "disk", // "disk" | "s3" | "cloudflare"
    "compression": {
      "images": true,
      "videos": false
    },
    "thumbnails": {
      "enabled": true,
      "sizes": ["150x150", "300x300"]
    }
  }
}
```

## ✅ Implementation Checklist

- [x] HTTP upload API with admin authentication
- [x] Disk storage with configurable directory
- [x] Enhanced client upload handlers
- [x] Smart file upload web component
- [x] Media browser with dual storage support
- [x] URL generation for disk files
- [x] Comprehensive error handling
- [x] TypeScript types and Zod schemas
- [x] Demo page with full functionality
- [x] Documentation and examples
- [x] Backward compatibility maintained
- [x] Security features implemented

## 🎉 Conclusion

This implementation successfully provides a robust, scalable file upload system that intelligently handles files of all sizes while maintaining excellent developer and user experiences. The dual-storage approach optimizes for both performance and storage efficiency, while the unified client interface makes the complexity transparent to end users.

The system is production-ready with proper authentication, validation, error handling, and comprehensive documentation.
