//! Bounded-memory async file reading and writing.
//!
//! # Memory budget
//! At most `READ_BUFFER_SIZE` bytes are held in memory at any time per
//! reader/writer instance. The transfer engine reads one buffer at a time,
//! hashes it, encrypts it, and sends it before reading the next.
//!
//! # Streaming reads
//! `ChunkReader` yields plaintext chunks sequentially. The caller is
//! responsible for encrypting and hashing each chunk before transmission.
//!
//! # Streaming writes
//! `ChunkWriter` receives decrypted, verified plaintext chunks and writes
//! them to the correct offset. Chunks may arrive out of order (QUIC streams
//! are independent), so the writer seeks to the correct offset.

use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom};

use crate::config::READ_BUFFER_SIZE;
use crate::error::{Result, SwiftWaveError};
use crate::security::hashing::{hash_chunk, Hash, StreamingHasher};

/// Async streaming file reader that yields fixed-size plaintext chunks.
pub struct ChunkReader {
    file: File,
    chunk_size: usize,
    file_size: u64,
    bytes_read: u64,
}

impl ChunkReader {
    /// Open a file for reading.
    pub async fn open(path: &Path, chunk_size: usize) -> Result<Self> {
        let file = File::open(path).await.map_err(SwiftWaveError::Io)?;
        let meta = file.metadata().await.map_err(SwiftWaveError::Io)?;
        Ok(Self {
            file,
            chunk_size,
            file_size: meta.len(),
            bytes_read: 0,
        })
    }

    /// File size in bytes.
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Read the next chunk. Returns `None` when the file is exhausted.
    ///
    /// # Memory note
    /// Allocates exactly `chunk_size` bytes per call (or less for the last chunk).
    pub async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>> {
        if self.bytes_read >= self.file_size {
            return Ok(None);
        }
        let remaining = (self.file_size - self.bytes_read).min(self.chunk_size as u64) as usize;
        let mut buf = vec![0u8; remaining];
        self.file
            .read_exact(&mut buf)
            .await
            .map_err(SwiftWaveError::Io)?;
        self.bytes_read += remaining as u64;
        Ok(Some(buf))
    }

    /// Compute the BLAKE3 hash of the entire file using streaming reads.
    ///
    /// Resets the file position to the beginning before reading and
    /// restores it to the start afterwards.
    pub async fn hash_file(&mut self) -> Result<Hash> {
        self.file
            .seek(SeekFrom::Start(0))
            .await
            .map_err(SwiftWaveError::Io)?;

        let mut hasher = StreamingHasher::new();
        let mut buf = vec![0u8; READ_BUFFER_SIZE];
        loop {
            let n = self.file.read(&mut buf).await.map_err(SwiftWaveError::Io)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }

        // Reset to start for subsequent reads.
        self.file
            .seek(SeekFrom::Start(0))
            .await
            .map_err(SwiftWaveError::Io)?;
        self.bytes_read = 0;

        Ok(hasher.finalize())
    }
}

/// Async streaming writer that writes verified plaintext chunks to disk.
pub struct ChunkWriter {
    path: PathBuf,
    file: BufWriter<File>,
}

impl ChunkWriter {
    /// Open (or create) a file for writing chunks.
    ///
    /// Pre-allocates the file to `file_size` bytes to avoid fragmentation.
    pub async fn create(path: &Path, file_size: u64) -> Result<Self> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(SwiftWaveError::Io)?;
        }
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path)
            .await
            .map_err(SwiftWaveError::Io)?;

        // Pre-allocate: seek to end and write a zero byte so the OS reserves space.
        // This prevents ENOSPC surprises mid-transfer.
        if file_size > 0 {
            file.set_len(file_size).await.map_err(SwiftWaveError::Io)?;
        }

        Ok(Self {
            path: path.to_path_buf(),
            file: BufWriter::new(file),
        })
    }

    /// Write a plaintext chunk at a specific byte offset.
    ///
    /// Chunks may be written out of order.
    pub async fn write_chunk(&mut self, offset: u64, data: &[u8]) -> Result<()> {
        use tokio::io::AsyncSeekExt;
        self.file
            .get_mut()
            .seek(SeekFrom::Start(offset))
            .await
            .map_err(SwiftWaveError::Io)?;
        self.file
            .get_mut()
            .write_all(data)
            .await
            .map_err(SwiftWaveError::Io)?;
        Ok(())
    }

    /// Flush all buffered data and close the file.
    pub async fn finish(mut self) -> Result<PathBuf> {
        self.file.flush().await.map_err(SwiftWaveError::Io)?;
        Ok(self.path)
    }
}

/// Compute per-chunk BLAKE3 hashes for an entire file with bounded memory.
///
/// Returns a vector of hashes in order, one per chunk.
pub async fn compute_chunk_hashes(
    path: &Path,
    chunk_size: usize,
) -> Result<(Vec<Hash>, Hash)> {
    let mut reader = ChunkReader::open(path, chunk_size).await?;
    let mut chunk_hashes = Vec::new();
    let mut file_hasher = StreamingHasher::new();

    while let Some(chunk) = reader.next_chunk().await? {
        let h = hash_chunk(&chunk);
        chunk_hashes.push(h);
        file_hasher.update(&chunk);
    }

    let file_hash = file_hasher.finalize();
    Ok((chunk_hashes, file_hash))
}
