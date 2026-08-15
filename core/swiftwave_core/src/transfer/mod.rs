//! Transfer module: sessions, chunks, state machine, resume, and progress.

pub mod chunk;
pub mod progress;
pub mod resume;
pub mod scheduler;
pub mod session;

pub use chunk::{ChunkMetadata, generate_chunk_plan};
pub use progress::{ProgressSnapshot, ProgressTracker};
pub use resume::ResumeBitmap;
pub use scheduler::{TransferScheduler, TransferState};
pub use session::{FileId, TransferDirection, TransferId, TransferSession};
