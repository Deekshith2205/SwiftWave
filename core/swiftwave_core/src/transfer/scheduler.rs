//! Transfer state machine.
//!
//! A `TransferScheduler` is the single authoritative owner of a transfer's
//! state. All state changes go through `transition_to`, which validates the
//! transition before applying it.
//!
//! # Valid transitions
//!
//! ```text
//! Idle → Discovering → Connecting → Authenticating → AwaitingVerification
//!                                                         ↓
//!                                                      Preparing
//!                                                         ↓
//!                                             ┌─────── Transferring ──────┐
//!                                             ↓           ↓               ↓
//!                                           Paused   Interrupted     Completed
//!                                             ↓           ↓
//!                                        Resuming ←────── ┘
//!                                             ↓
//!                                         Transferring (again)
//!
//! Any state → Cancelled
//! Any state → Failed
//! ```

use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{info, warn};

use crate::error::{Result, SwiftWaveError};

/// All possible states of a transfer session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferState {
    /// No active transfer.
    Idle,
    /// Broadcasting presence and scanning for the target peer.
    Discovering,
    /// TCP/QUIC connection being established.
    Connecting,
    /// Noise XX handshake in progress.
    Authenticating,
    /// SAS displayed; waiting for user to confirm both devices match.
    AwaitingVerification,
    /// Building chunk plan and opening streams.
    Preparing,
    /// Actively sending or receiving chunks.
    Transferring,
    /// Transfer intentionally paused by local user.
    Paused,
    /// Transfer interrupted by a network error.
    Interrupted,
    /// Resuming a previously interrupted transfer.
    Resuming,
    /// All chunks transferred and verified.
    Completed,
    /// Transfer cancelled by local or remote user.
    Cancelled,
    /// Unrecoverable failure.
    Failed,
}

impl std::fmt::Display for TransferState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Idle => "Idle",
            Self::Discovering => "Discovering",
            Self::Connecting => "Connecting",
            Self::Authenticating => "Authenticating",
            Self::AwaitingVerification => "AwaitingVerification",
            Self::Preparing => "Preparing",
            Self::Transferring => "Transferring",
            Self::Paused => "Paused",
            Self::Interrupted => "Interrupted",
            Self::Resuming => "Resuming",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
            Self::Failed => "Failed",
        };
        write!(f, "{s}")
    }
}

impl TransferState {
    /// Returns `true` if this state represents a terminal (non-recoverable) state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }

    /// Returns `true` if a transition from `self` to `next` is permitted.
    pub fn can_transition_to(&self, next: &TransferState) -> bool {
        // Cancelled and Failed are always reachable from any non-terminal state.
        if matches!(next, TransferState::Cancelled | TransferState::Failed) {
            return !self.is_terminal();
        }
        matches!(
            (self, next),
            (TransferState::Idle, TransferState::Discovering)
            | (TransferState::Discovering, TransferState::Connecting)
            | (TransferState::Connecting, TransferState::Authenticating)
            | (TransferState::Authenticating, TransferState::AwaitingVerification)
            | (TransferState::AwaitingVerification, TransferState::Preparing)
            | (TransferState::Preparing, TransferState::Transferring)
            | (TransferState::Transferring, TransferState::Paused)
            | (TransferState::Transferring, TransferState::Interrupted)
            | (TransferState::Transferring, TransferState::Completed)
            | (TransferState::Paused, TransferState::Transferring)
            | (TransferState::Paused, TransferState::Resuming)
            | (TransferState::Interrupted, TransferState::Resuming)
            | (TransferState::Resuming, TransferState::Transferring)
        )
    }
}

/// Manages the state and scheduling of a single transfer.
pub struct TransferScheduler {
    /// Transfer session ID.
    pub transfer_id: String,
    /// Current state.
    state: TransferState,
    /// When the current state was entered.
    state_entered_at: Instant,
    /// History of states for debugging.
    history: Vec<(TransferState, Instant)>,
}

impl TransferScheduler {
    /// Create a new scheduler for `transfer_id`, starting in `Idle`.
    pub fn new(transfer_id: impl Into<String>) -> Self {
        Self {
            transfer_id: transfer_id.into(),
            state: TransferState::Idle,
            state_entered_at: Instant::now(),
            history: Vec::new(),
        }
    }

    /// Return the current state.
    pub fn state(&self) -> &TransferState {
        &self.state
    }

    /// Attempt to transition to `next`.
    ///
    /// Returns `Err(InvalidTransition)` if the transition is not permitted.
    pub fn transition_to(&mut self, next: TransferState) -> Result<()> {
        if !self.state.can_transition_to(&next) {
            let err = SwiftWaveError::InvalidTransition {
                from: self.state.to_string(),
                to: next.to_string(),
            };
            warn!(
                transfer_id = %self.transfer_id,
                "Rejected state transition: {}", err
            );
            return Err(err);
        }
        let now = Instant::now();
        let prev = std::mem::replace(&mut self.state, next);
        self.history.push((prev, self.state_entered_at));
        self.state_entered_at = now;
        info!(
            transfer_id = %self.transfer_id,
            state = %self.state,
            "State transition"
        );
        Ok(())
    }

    /// How long the scheduler has been in the current state.
    pub fn time_in_state(&self) -> std::time::Duration {
        self.state_entered_at.elapsed()
    }

    /// Full history of states entered (oldest first).
    pub fn history(&self) -> &[(TransferState, Instant)] {
        &self.history
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn full_happy_path() -> Vec<TransferState> {
        vec![
            TransferState::Discovering,
            TransferState::Connecting,
            TransferState::Authenticating,
            TransferState::AwaitingVerification,
            TransferState::Preparing,
            TransferState::Transferring,
            TransferState::Completed,
        ]
    }

    #[test]
    fn happy_path_transitions_succeed() {
        let mut s = TransferScheduler::new("test-transfer");
        for next in full_happy_path() {
            s.transition_to(next).expect("transition should succeed");
        }
        assert_eq!(s.state(), &TransferState::Completed);
        assert!(s.state().is_terminal());
    }

    #[test]
    fn pause_and_resume() {
        let mut s = TransferScheduler::new("t");
        for st in [
            TransferState::Discovering,
            TransferState::Connecting,
            TransferState::Authenticating,
            TransferState::AwaitingVerification,
            TransferState::Preparing,
            TransferState::Transferring,
        ] {
            s.transition_to(st).unwrap();
        }
        s.transition_to(TransferState::Paused).unwrap();
        s.transition_to(TransferState::Resuming).unwrap();
        s.transition_to(TransferState::Transferring).unwrap();
        s.transition_to(TransferState::Completed).unwrap();
    }

    #[test]
    fn interrupt_and_resume() {
        let mut s = TransferScheduler::new("t");
        for st in [
            TransferState::Discovering,
            TransferState::Connecting,
            TransferState::Authenticating,
            TransferState::AwaitingVerification,
            TransferState::Preparing,
            TransferState::Transferring,
        ] {
            s.transition_to(st).unwrap();
        }
        s.transition_to(TransferState::Interrupted).unwrap();
        s.transition_to(TransferState::Resuming).unwrap();
        s.transition_to(TransferState::Transferring).unwrap();
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut s = TransferScheduler::new("t");
        // Idle → Transferring is invalid.
        assert!(s.transition_to(TransferState::Transferring).is_err());
    }

    #[test]
    fn cancelled_from_any_non_terminal_state() {
        let non_terminals = [
            TransferState::Idle,
            TransferState::Discovering,
            TransferState::Connecting,
            TransferState::Transferring,
            TransferState::Paused,
        ];
        for from in non_terminals {
            let mut s = TransferScheduler::new("t");
            s.state = from;
            s.transition_to(TransferState::Cancelled).unwrap();
        }
    }

    #[test]
    fn no_transition_from_terminal_state() {
        let mut s = TransferScheduler::new("t");
        s.state = TransferState::Completed;
        assert!(s.transition_to(TransferState::Cancelled).is_err());
        assert!(s.transition_to(TransferState::Failed).is_err());
    }

    #[test]
    fn history_is_recorded() {
        let mut s = TransferScheduler::new("t");
        s.transition_to(TransferState::Discovering).unwrap();
        assert_eq!(s.history().len(), 1);
        assert_eq!(s.history()[0].0, TransferState::Idle);
    }
}
