//! Storage module: file I/O and metadata.

pub mod file_reader;
pub mod metadata;

pub use file_reader::{ChunkReader, ChunkWriter, compute_chunk_hashes};
pub use metadata::{FileMetadata, sanitise_filename};
