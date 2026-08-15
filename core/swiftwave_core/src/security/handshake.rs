//! Noise Protocol handshake for SwiftWave peer authentication.
//!
//! # Protocol: Noise_XX_25519_ChaChaPoly_BLAKE2s
//!
//! We use the **XX** pattern because:
//! - Both initiator and responder are mutually authenticated.
//! - Neither side needs prior knowledge of the other's static key
//!   (unlike IK/NK patterns).
//! - After the handshake, both peers have a shared secret and have
//!   verified each other's long-term public keys.
//!
//! # Security properties
//! - **Mutual authentication** — both sides prove possession of their
//!   static private key.
//! - **Forward secrecy** — ephemeral keys are discarded after the handshake.
//! - **Identity hiding** — static keys are transmitted encrypted inside the
//!   handshake messages.
//! - **No PKI** — trust is established out-of-band via the SAS (Short
//!   Authentication String) that the user confirms on both devices.
//!
//! # Wire format
//! Messages are framed as `[u16 length][payload]` over the underlying stream.

use snow::{Builder, TransportState};

use crate::error::{Result, SwiftWaveError};

/// Noise parameters used across SwiftWave.
///
/// `BLAKE2s` is used inside the Noise handshake (as specified by the `snow`
/// crate's pattern string). BLAKE3 is used separately for file integrity.
const NOISE_PARAMS: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";

/// Maximum size of a single Noise handshake message (65535 bytes).
const MAX_MESSAGE: usize = 65535;

/// State after a completed Noise handshake.
///
/// Wraps the `snow` transport state which provides `read_message` /
/// `write_message` for encrypted stream data post-handshake.
pub struct HandshakeResult {
    /// Established symmetric transport state.
    pub transport: TransportState,
    /// The remote peer's static public key (32 bytes, X25519).
    ///
    /// The caller MUST verify this against a known-trusted fingerprint
    /// or prompt the user to confirm the SAS before using the transport.
    pub remote_static_public_key: Vec<u8>,
}

/// Perform the **initiator** side of a Noise XX handshake.
///
/// `local_static_key` must be the 32-byte X25519 private key of this device.
/// `transport` is any `AsyncRead + AsyncWrite` stream (e.g., a QUIC stream).
///
/// Returns a `HandshakeResult` on success, or `HandshakeFailed` on any error.
/// The opaque error prevents oracle attacks — never reveal why a handshake
/// failed to the remote peer.
pub async fn initiate_handshake(
    local_static_key: &[u8; 32],
    stream: &mut (impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin),
) -> Result<HandshakeResult> {
    let builder = Builder::new(NOISE_PARAMS.parse().map_err(|_| SwiftWaveError::HandshakeFailed)?)
        .local_private_key(local_static_key)
        .build_initiator()
        .map_err(|_| SwiftWaveError::HandshakeFailed)?;

    run_handshake(builder, stream, true).await
}

/// Perform the **responder** side of a Noise XX handshake.
pub async fn respond_handshake(
    local_static_key: &[u8; 32],
    stream: &mut (impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin),
) -> Result<HandshakeResult> {
    let builder = Builder::new(NOISE_PARAMS.parse().map_err(|_| SwiftWaveError::HandshakeFailed)?)
        .local_private_key(local_static_key)
        .build_responder()
        .map_err(|_| SwiftWaveError::HandshakeFailed)?;

    run_handshake(builder, stream, false).await
}

/// Drive the Noise state machine to completion, reading and writing framed
/// messages over `stream`.
async fn run_handshake(
    mut state: snow::HandshakeState,
    stream: &mut (impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin),
    initiator: bool,
) -> Result<HandshakeResult> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = vec![0u8; MAX_MESSAGE];
    let mut msg = vec![0u8; MAX_MESSAGE];

    // XX pattern: initiator sends → responder replies → initiator sends
    // Noise state machine tracks whose turn it is via `is_my_turn()`.
    while !state.is_handshake_finished() {
        if state.is_my_turn() {
            // Write a handshake message.
            let len = state
                .write_message(&[], &mut msg)
                .map_err(|_| SwiftWaveError::HandshakeFailed)?;

            // Frame as u16 big-endian length prefix.
            let frame_len = (len as u16).to_be_bytes();
            stream
                .write_all(&frame_len)
                .await
                .map_err(SwiftWaveError::Io)?;
            stream
                .write_all(&msg[..len])
                .await
                .map_err(SwiftWaveError::Io)?;
        } else {
            // Read a framed handshake message.
            let mut len_buf = [0u8; 2];
            stream
                .read_exact(&mut len_buf)
                .await
                .map_err(|_| SwiftWaveError::HandshakeFailed)?;
            let payload_len = u16::from_be_bytes(len_buf) as usize;

            if payload_len > MAX_MESSAGE {
                return Err(SwiftWaveError::HandshakeFailed);
            }

            stream
                .read_exact(&mut buf[..payload_len])
                .await
                .map_err(|_| SwiftWaveError::HandshakeFailed)?;

            state
                .read_message(&buf[..payload_len], &mut msg)
                .map_err(|_| SwiftWaveError::HandshakeFailed)?;
        }
    }

    // Extract the remote static public key before converting to transport.
    let remote_static_public_key = state
        .get_remote_static()
        .ok_or(SwiftWaveError::HandshakeFailed)?
        .to_vec();

    let transport = state
        .into_transport_mode()
        .map_err(|_| SwiftWaveError::HandshakeFailed)?;

    let _ = initiator; // suppress unused warning; role is tracked by snow

    Ok(HandshakeResult {
        transport,
        remote_static_public_key,
    })
}
