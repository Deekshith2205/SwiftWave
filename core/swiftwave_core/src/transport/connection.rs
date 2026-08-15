//! Transport connection and stream abstractions.
//!
//! The `Connection` trait wraps a QUIC connection and provides
//! `open_stream()` for the sender and `accept_stream()` for the receiver.
//! This trait-based design allows the engine to be tested with in-memory
//! transports without needing a real QUIC stack.

use async_trait::async_trait;
use bytes::Bytes;

use crate::error::Result;

/// A single outgoing QUIC stream (or any ordered, reliable byte channel).
#[async_trait]
pub trait OutgoingStream: Send {
    /// Send bytes on this stream.
    async fn send(&mut self, data: Bytes) -> Result<()>;
    /// Gracefully close the stream.
    async fn finish(&mut self) -> Result<()>;
}

/// A single incoming QUIC stream (or any ordered, reliable byte channel).
#[async_trait]
pub trait IncomingStream: Send {
    /// Read the next chunk of bytes (returns `None` at end of stream).
    async fn recv(&mut self) -> Result<Option<Bytes>>;
}

/// A multiplexed peer connection that can open or accept independent streams.
#[async_trait]
pub trait Connection: Send + Sync {
    /// Open a new outgoing stream to the peer.
    async fn open_stream(&mut self) -> Result<Box<dyn OutgoingStream>>;

    /// Accept the next incoming stream from the peer.
    async fn accept_stream(&mut self) -> Result<Option<Box<dyn IncomingStream>>>;

    /// Close the connection gracefully.
    async fn close(&mut self) -> Result<()>;

    /// Remote peer address.
    fn remote_address(&self) -> std::net::SocketAddr;
}
