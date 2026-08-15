//! Real-time transfer progress reporting.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::sync::watch;

use crate::transfer::session::TransferId;

/// A snapshot of transfer progress at a single point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSnapshot {
    /// Session ID this progress belongs to.
    pub transfer_id: TransferId,
    /// Total bytes to transfer (0 if not yet known).
    pub total_bytes: u64,
    /// Bytes confirmed transferred.
    pub transferred_bytes: u64,
    /// Total chunks.
    pub total_chunks: u64,
    /// Chunks confirmed complete.
    pub completed_chunks: u64,
    /// Estimated transfer speed in bytes per second.
    pub bytes_per_second: f64,
    /// Estimated time remaining in seconds (None if speed is zero).
    pub eta_seconds: Option<f64>,
}

impl ProgressSnapshot {
    /// Completion fraction in [0.0, 1.0].
    pub fn fraction(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            self.transferred_bytes as f64 / self.total_bytes as f64
        }
    }

    /// Percentage complete (0–100).
    pub fn percent(&self) -> f64 {
        self.fraction() * 100.0
    }
}

/// Tracks bytes transferred and computes speed using a sliding window.
pub struct ProgressTracker {
    transfer_id: TransferId,
    total_bytes: u64,
    total_chunks: u64,
    transferred_bytes: u64,
    completed_chunks: u64,
    /// (timestamp, cumulative bytes) pairs for the sliding window.
    window: std::collections::VecDeque<(Instant, u64)>,
    window_duration: Duration,
    /// `watch` sender for broadcasting snapshots.
    sender: watch::Sender<ProgressSnapshot>,
}

impl ProgressTracker {
    /// Create a new tracker and return it alongside the receiver end.
    pub fn new(
        transfer_id: TransferId,
        total_bytes: u64,
        total_chunks: u64,
    ) -> (Self, watch::Receiver<ProgressSnapshot>) {
        let initial = ProgressSnapshot {
            transfer_id: transfer_id.clone(),
            total_bytes,
            transferred_bytes: 0,
            total_chunks,
            completed_chunks: 0,
            bytes_per_second: 0.0,
            eta_seconds: None,
        };
        let (sender, receiver) = watch::channel(initial);
        let tracker = Self {
            transfer_id,
            total_bytes,
            total_chunks,
            transferred_bytes: 0,
            completed_chunks: 0,
            window: std::collections::VecDeque::new(),
            window_duration: Duration::from_secs(5),
            sender,
        };
        (tracker, receiver)
    }

    /// Record `bytes` transferred and optionally mark a chunk as complete.
    pub fn record(&mut self, bytes: u64, chunk_complete: bool) {
        self.transferred_bytes += bytes;
        if chunk_complete {
            self.completed_chunks += 1;
        }

        let now = Instant::now();
        self.window.push_back((now, self.transferred_bytes));

        // Evict entries outside the sliding window.
        while let Some(&(ts, _)) = self.window.front() {
            if now.duration_since(ts) > self.window_duration {
                self.window.pop_front();
            } else {
                break;
            }
        }

        let bps = self.compute_speed(now);
        let eta = if bps > 0.0 {
            let remaining = self.total_bytes.saturating_sub(self.transferred_bytes);
            Some(remaining as f64 / bps)
        } else {
            None
        };

        let snap = ProgressSnapshot {
            transfer_id: self.transfer_id.clone(),
            total_bytes: self.total_bytes,
            transferred_bytes: self.transferred_bytes,
            total_chunks: self.total_chunks,
            completed_chunks: self.completed_chunks,
            bytes_per_second: bps,
            eta_seconds: eta,
        };
        // Ignore send error — receiver may have been dropped.
        let _ = self.sender.send(snap);
    }

    fn compute_speed(&self, now: Instant) -> f64 {
        if self.window.len() < 2 {
            return 0.0;
        }
        let (oldest_ts, oldest_bytes) = self.window.front().unwrap();
        let elapsed = now.duration_since(*oldest_ts).as_secs_f64();
        if elapsed < 0.001 {
            return 0.0;
        }
        let delta_bytes = self.transferred_bytes.saturating_sub(*oldest_bytes);
        delta_bytes as f64 / elapsed
    }
}
