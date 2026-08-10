//! Storage abstraction — reading and writing files on the local device.
//!
//! The `Storage` trait insulates the rest of the engine from platform-specific
//! filesystem APIs (Android content-provider, iOS sandbox, etc.).
//!
//! # TODO (Phase 2)
//! - Android: Implement `ContentProviderStorage` using JNI / flutter_file_picker.
//! - iOS: Implement sandboxed `NSFileManager` adapter via FFI.
//! - Add a quota / available-space check before accepting a transfer.

use crate::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Metadata about a file in storage.
#[derive(Debug, Clone)]
pub struct StorageFile {
    /// Absolute path on the local filesystem (may be a content URI on Android).
    pub path: PathBuf,
    /// File size in bytes.
    pub size: u64,
    /// MIME type if determinable (e.g. `"image/jpeg"`).
    pub mime_type: Option<String>,
}

/// The `Storage` trait — provides filesystem access to the engine.
#[async_trait]
pub trait Storage: Send + Sync {
    /// Open a file for sequential reading.
    ///
    /// Returns a `tokio::fs::File` handle.
    ///
    /// # TODO (Phase 2): Return `bytes::Bytes` chunks for zero-copy pipeline.
    async fn open_for_read(&self, path: &Path) -> Result<tokio::fs::File>;

    /// Create or overwrite a file for sequential writing.
    async fn open_for_write(&self, path: &Path) -> Result<tokio::fs::File>;

    /// Return available free space in bytes at the given path.
    ///
    /// # TODO (Phase 2): implement via `statvfs` / `GetDiskFreeSpaceEx`.
    async fn available_space(&self, path: &Path) -> Result<u64>;

    /// List files in a directory (non-recursive).
    async fn list_dir(&self, dir: &Path) -> Result<Vec<StorageFile>>;

    /// Delete a file.
    async fn delete(&self, path: &Path) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Local filesystem implementation (thin tokio::fs wrapper)
// ---------------------------------------------------------------------------

/// `LocalStorage` — delegates to `tokio::fs` for standard OS file access.
pub struct LocalStorage;

#[async_trait]
impl Storage for LocalStorage {
    async fn open_for_read(&self, path: &Path) -> Result<tokio::fs::File> {
        Ok(tokio::fs::File::open(path).await?)
    }

    async fn open_for_write(&self, path: &Path) -> Result<tokio::fs::File> {
        Ok(tokio::fs::File::create(path).await?)
    }

    async fn available_space(&self, _path: &Path) -> Result<u64> {
        // TODO (Phase 2): implement with platform-specific syscall.
        Ok(u64::MAX)
    }

    async fn list_dir(&self, dir: &Path) -> Result<Vec<StorageFile>> {
        let mut entries = tokio::fs::read_dir(dir).await?;
        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                files.push(StorageFile {
                    path: entry.path(),
                    size: metadata.len(),
                    mime_type: None, // TODO (Phase 2): infer from extension.
                });
            }
        }

        Ok(files)
    }

    async fn delete(&self, path: &Path) -> Result<()> {
        Ok(tokio::fs::remove_file(path).await?)
    }
}
