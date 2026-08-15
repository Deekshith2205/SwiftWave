//! Transport abstraction: QUIC connection and stream traits.

pub mod connection;
pub mod quic;

pub use connection::{Connection, IncomingStream, OutgoingStream};
