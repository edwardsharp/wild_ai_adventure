/**
 * Media Blob Manager
 *
 * Handles media blob data management, caching, thumbnail generation,
 * and display formatting for WebSocket-received media blobs.
 */

export interface MediaBlob {
  id: string;
  data?: number[];
  sha256: string;
  size: number;
  mime: string;
  source_client_id?: string;
  local_path?: string;
  metadata?: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface MediaBlobData {
  id: string;
  data: number[];
  mime: string;
  size: number;
}

export interface BlobDisplayInfo {
  id: string;
  mime: string;
  size: string;
  sha256: string;
  clientId: string;
  path: string;
  createdAt: string;
  metadata: string;
  thumbnailHtml: string;
}

export class MediaBlobManager extends EventTarget {
  private blobs: MediaBlob[] = [];
  private blobDataCache = new Map<string, string>(); // blob ID -> data URL
  private loadingBlobs = new Set<string>();

  constructor() {
    super();
  }

  /**
   * Update the list of media blobs
   */
  updateBlobs(blobs: MediaBlob[]): void {
    this.blobs = [...blobs];

    this.dispatchEvent(new CustomEvent('blobs-updated', {
      detail: { blobs: this.blobs, count: this.blobs.length }
    }));
  }

  /**
   * Get all blobs
   */
  getBlobs(): MediaBlob[] {
    return [...this.blobs];
  }

  /**
   * Get a specific blob by ID
   */
  getBlob(id: string): MediaBlob | undefined {
    return this.blobs.find(blob => blob.id === id);
  }

  /**
   * Add blob data to cache
   */
  cacheBlobData(blobData: MediaBlobData): void {
    if (!blobData.id || !blobData.data) return;

    // Convert data array to Uint8Array and create blob
    const uint8Array = new Uint8Array(blobData.data);
    const blob = new Blob([uint8Array], {
      type: blobData.mime || 'application/octet-stream',
    });
    const dataUrl = URL.createObjectURL(blob);

    // Cache the data URL
    this.blobDataCache.set(blobData.id, dataUrl);
    this.loadingBlobs.delete(blobData.id);

    this.dispatchEvent(new CustomEvent('blob-data-cached', {
      detail: { id: blobData.id, dataUrl, mime: blobData.mime }
    }));
  }

  /**
   * Check if blob data is cached
   */
  isCached(blobId: string): boolean {
    return this.blobDataCache.has(blobId);
  }

  /**
   * Get cached data URL for a blob
   */
  getCachedDataUrl(blobId: string): string | undefined {
    return this.blobDataCache.get(blobId);
  }

  /**
   * Check if blob is currently loading
   */
  isLoading(blobId: string): boolean {
    return this.loadingBlobs.has(blobId);
  }

  /**
   * Mark blob as loading
   */
  markAsLoading(blobId: string): void {
    this.loadingBlobs.add(blobId);
  }

  /**
   * Request blob data (emits event for external handler)
   */
  requestBlobData(blobId: string): void {
    if (this.isCached(blobId) || this.isLoading(blobId)) {
      return;
    }

    this.markAsLoading(blobId);

    this.dispatchEvent(new CustomEvent('blob-data-requested', {
      detail: { id: blobId }
    }));
  }

  /**
   * Generate display information for a blob
   */
  getBlobDisplayInfo(blob: MediaBlob): BlobDisplayInfo {
    return {
      id: blob.id,
      mime: blob.mime || 'Unknown type',
      size: this.formatFileSize(blob.size),
      sha256: blob.sha256,
      clientId: blob.source_client_id || 'Unknown',
      path: blob.local_path || 'None',
      createdAt: new Date(blob.created_at).toLocaleString(),
      metadata: Object.keys(blob.metadata || {}).length > 0
        ? JSON.stringify(blob.metadata)
        : '',
      thumbnailHtml: this.generateThumbnailHtml(blob),
    };
  }

  /**
   * Generate thumbnail HTML for a blob
   */
  generateThumbnailHtml(blob: MediaBlob): string {
    const mime = blob.mime || '';
    const cachedData = this.getCachedDataUrl(blob.id);
    const isLoading = this.isLoading(blob.id);

    const baseStyle = 'width: 80px; height: 80px; border-radius: 4px; object-fit: cover;';
    const placeholderStyle = 'display: flex; align-items: center; justify-content: center; background: #f0f0f0; font-size: 0.7em; border-radius: 4px; cursor: pointer;';

    if (mime.startsWith('image/')) {
      if (cachedData) {
        return `<img src="${cachedData}" alt="Thumbnail" style="${baseStyle}" loading="lazy">`;
      } else if (isLoading) {
        return `<div style="${baseStyle} ${placeholderStyle}">Loading...</div>`;
      } else {
        return `<div style="${baseStyle} ${placeholderStyle}" data-blob-id="${blob.id}">LOAD IMAGE</div>`;
      }
    } else if (mime.startsWith('video/')) {
      if (cachedData) {
        return `<video style="${baseStyle}" controls muted><source src="${cachedData}" type="${mime}"></video>`;
      } else if (isLoading) {
        return `<div style="${baseStyle} ${placeholderStyle}">Loading...</div>`;
      } else {
        return `<div style="${baseStyle} ${placeholderStyle}" data-blob-id="${blob.id}">LOAD VIDEO</div>`;
      }
    } else if (mime.startsWith('audio/')) {
      if (cachedData) {
        return `<audio style="${baseStyle}" controls><source src="${cachedData}" type="${mime}"></audio>`;
      } else if (isLoading) {
        return `<div style="${baseStyle} ${placeholderStyle}">Loading...</div>`;
      } else {
        return `<div style="${baseStyle} ${placeholderStyle}" data-blob-id="${blob.id}">LOAD AUDIO</div>`;
      }
    } else if (mime === 'application/pdf') {
      return `<div style="${baseStyle} ${placeholderStyle}">PDF</div>`;
    } else {
      return `<div style="${baseStyle} ${placeholderStyle}">FILE</div>`;
    }
  }

  /**
   * Download a cached blob
   */
  downloadBlob(blobId: string, filename?: string): boolean {
    const cachedData = this.getCachedDataUrl(blobId);
    if (!cachedData) {
      this.requestBlobData(blobId);
      return false;
    }

    const blob = this.getBlob(blobId);
    const downloadName = filename || blob?.local_path || `blob-${blobId}`;

    // Create download link
    const a = document.createElement('a');
    a.href = cachedData;
    a.download = downloadName;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);

    this.dispatchEvent(new CustomEvent('blob-downloaded', {
      detail: { id: blobId, filename: downloadName }
    }));

    return true;
  }

  /**
   * View a cached blob in new tab
   */
  viewBlob(blobId: string): boolean {
    const cachedData = this.getCachedDataUrl(blobId);
    if (!cachedData) {
      this.requestBlobData(blobId);
      return false;
    }

    window.open(cachedData, '_blank');

    this.dispatchEvent(new CustomEvent('blob-viewed', {
      detail: { id: blobId }
    }));

    return true;
  }

  /**
   * Format file size in human-readable format
   */
  formatFileSize(bytes: number): string {
    if (!bytes) return 'Unknown size';

    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }

  /**
   * Clear all cached data
   */
  clearCache(): void {
    // Revoke all object URLs to free memory
    for (const dataUrl of this.blobDataCache.values()) {
      URL.revokeObjectURL(dataUrl);
    }

    this.blobDataCache.clear();
    this.loadingBlobs.clear();

    this.dispatchEvent(new CustomEvent('cache-cleared', {
      detail: { timestamp: Date.now() }
    }));
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): { cachedCount: number; loadingCount: number; totalBlobs: number } {
    return {
      cachedCount: this.blobDataCache.size,
      loadingCount: this.loadingBlobs.size,
      totalBlobs: this.blobs.length,
    };
  }

  /**
   * Clean up resources
   */
  destroy(): void {
    this.clearCache();
    this.blobs = [];

    // Remove all event listeners
    const events = ['blobs-updated', 'blob-data-cached', 'blob-data-requested', 'blob-downloaded', 'blob-viewed', 'cache-cleared'];
    events.forEach(event => {
      const listeners = (this as any)._listeners?.[event] || [];
      listeners.forEach((listener: any) => {
        this.removeEventListener(event, listener);
      });
    });
  }
}
