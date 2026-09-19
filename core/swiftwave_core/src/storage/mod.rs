//! Storage module: file I/O and metadata.

pub mod file_reader;
pub mod metadata;

pub use file_reader::{compute_chunk_hashes, ChunkReader, ChunkWriter};
pub use metadata::{sanitise_filename, FileMetadata};
