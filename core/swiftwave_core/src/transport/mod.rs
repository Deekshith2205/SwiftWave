//! Transport layer — reliable, ordered, multiplexed byte-stream delivery.
//!
//! SwiftWave Share targets **QUIC** (via `quinn`) as its primary transport because:
//! - Multiplexed streams without head-of-line blocking.
//! - Built-in TLS 1.3 (though SwiftWave adds its own Noise layer on top).
//! - 0-RTT connection resumption.
//! - Works over UDP — survives NAT and network changes better than TCP.
//!
//! # TODO (Phase 2)
//! - Implement `QuicTransport` using `quinn 0.11`.
//! - Implement `TcpTransport` as a fallback for platforms where QUIC is
//!   unavailable or blocked.
//! - Add stream multiplexing helpers (one stream per file chunk).
//! - Measure and optimise for zero-copy with `bytes::Bytes`.

use crate::Result;
use async_trait::async_trait;

/// An abstract bidirectional byte stream opened over a `Connection`.
///
/// Both `send` and `recv` are async and cancel-safe.
#[async_trait]
pub trait Stream: Send + Sync {
    /// Send bytes. Implementations SHOULD avoid copying where possible.
    ///
    /// # TODO (Phase 2): accept `bytes::Bytes` for zero-copy.
    async fn send(&mut self, data: &[u8]) -> Result<()>;

    /// Receive up to `max_len` bytes.  Returns `None` on clean close.
    async fn recv(&mut self, max_len: usize) -> Result<Option<Vec<u8>>>;

    /// Close this stream gracefully.
    async fn close(&mut self) -> Result<()>;
}

/// An abstract multiplexed connection to a single remote peer.
#[async_trait]
pub trait Connection: Send + Sync {
    /// Open a new outgoing stream on this connection.
    async fn open_stream(&self) -> Result<Box<dyn Stream>>;

    /// Accept an incoming stream opened by the remote peer.
    async fn accept_stream(&self) -> Result<Box<dyn Stream>>;

    /// Close the connection, dropping all streams.
    async fn close(&self) -> Result<()>;

    /// Return the remote address as a string (e.g. `"192.168.1.5:7777"`).
    fn remote_address(&self) -> &str;
}

/// The `Transport` trait — implemented by every transport backend.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to a remote peer at the given address.
    ///
    /// The caller is responsible for completing the security handshake
    /// **after** the transport connection is established.
    async fn connect(&self, address: &str) -> Result<Box<dyn Connection>>;

    /// Listen for incoming connections.
    ///
    /// `on_connection` is called for each new peer.  Should run until
    /// the transport is stopped.
    ///
    /// # TODO (Phase 2): Return a stream of connections instead.
    async fn listen(
        &self,
        bind_address: &str,
        on_connection: Box<dyn Fn(Box<dyn Connection>) + Send + Sync>,
    ) -> Result<()>;

    /// Stop listening and release all OS resources.
    async fn shutdown(&self) -> Result<()>;

    /// Human-readable transport name (e.g. `"quic"`, `"tcp"`).
    fn transport_name(&self) -> &'static str;
}
