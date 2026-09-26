//! Noise Protocol handshake and HandshakeState for SwiftWave.
//!
//! # Security properties
//! - **Mutual authentication**: Proves possession of static X25519 keys.
//! - **Forward secrecy**: Ephemeral keys are discarded after handshake.
//! - **Identity hiding**: Static keys are transmitted encrypted.

use snow::{Builder, HandshakeState as SnowHandshakeState};

use crate::device::identity::{PeerIdentity, PublicKeyFingerprint};
use crate::error::{Result, SwiftWaveError};
use crate::security::session::SecureSession;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::time::timeout;

const NOISE_PARAMS: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";
const MAX_MESSAGE: usize = 4096;
const HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub(crate) async fn read_framed_message<R>(reader: &mut R) -> Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
{
    timeout(HANDSHAKE_TIMEOUT, async {
        let mut len_buf = [0u8; 2];
        reader.read_exact(&mut len_buf).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                SwiftWaveError::UnexpectedEof
            } else {
                SwiftWaveError::Io(e)
            }
        })?;

        let len = u16::from_be_bytes(len_buf) as usize;
        if len > MAX_MESSAGE {
            return Err(SwiftWaveError::FrameTooLarge);
        }

        let mut payload = vec![0u8; len];
        if len > 0 {
            reader.read_exact(&mut payload).await.map_err(|e| {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    SwiftWaveError::UnexpectedEof
                } else {
                    SwiftWaveError::Io(e)
                }
            })?;
        }
        Ok(payload)
    })
    .await
    .unwrap_or(Err(SwiftWaveError::HandshakeTimeout))
}

pub(crate) async fn write_framed_message<W>(writer: &mut W, message: &[u8]) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    if message.len() > MAX_MESSAGE {
        return Err(SwiftWaveError::FrameTooLarge);
    }
    let len_buf = (message.len() as u16).to_be_bytes();

    let mut frame = Vec::with_capacity(2 + message.len());
    frame.extend_from_slice(&len_buf);
    frame.extend_from_slice(message);

    match timeout(HANDSHAKE_TIMEOUT, writer.write_all(&frame)).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(SwiftWaveError::Io(e)),
        Err(_) => Err(SwiftWaveError::HandshakeTimeout),
    }
}

/// Represents an active, incomplete Noise handshake.
pub struct HandshakeState {
    state: SnowHandshakeState,
    is_initiator: bool,
}

impl HandshakeState {
    /// Initialize a new handshake state as the initiator.
    pub fn new_initiator(local_static_key: &[u8; 32]) -> Result<Self> {
        let builder = Builder::new(NOISE_PARAMS.parse().unwrap())
            .local_private_key(local_static_key)
            .build_initiator()
            .map_err(|_| SwiftWaveError::HandshakeFailed)?;
        Ok(Self {
            state: builder,
            is_initiator: true,
        })
    }

    /// Initialize a new handshake state as the responder.
    pub fn new_responder(local_static_key: &[u8; 32]) -> Result<Self> {
        let builder = Builder::new(NOISE_PARAMS.parse().unwrap())
            .local_private_key(local_static_key)
            .build_responder()
            .map_err(|_| SwiftWaveError::HandshakeFailed)?;
        Ok(Self {
            state: builder,
            is_initiator: false,
        })
    }

    /// Read an incoming handshake message from the peer.
    pub fn read_message(&mut self, payload: &[u8], out: &mut [u8]) -> Result<usize> {
        if payload.len() > MAX_MESSAGE {
            return Err(SwiftWaveError::HandshakeFailed);
        }
        self.state
            .read_message(payload, out)
            .map_err(|_| SwiftWaveError::HandshakeFailed)
    }

    /// Write the next outgoing handshake message.
    pub fn write_message(&mut self, payload: &[u8], out: &mut [u8]) -> Result<usize> {
        self.state
            .write_message(payload, out)
            .map_err(|_| SwiftWaveError::HandshakeFailed)
    }

    /// Read an incoming framed handshake message from the async reader.
    pub async fn read_framed<R>(&mut self, reader: &mut R, out: &mut [u8]) -> Result<usize>
    where
        R: AsyncRead + Unpin,
    {
        let payload = read_framed_message(reader).await?;
        self.read_message(&payload, out)
    }

    /// Write the next outgoing framed handshake message to the async writer.
    pub async fn write_framed<W>(&mut self, writer: &mut W, out: &mut [u8]) -> Result<usize>
    where
        W: AsyncWrite + Unpin,
    {
        let len = self.write_message(&[], out)?;
        write_framed_message(writer, &out[..len]).await?;
        Ok(len)
    }

    /// Returns `true` if the handshake is complete.
    pub fn is_finished(&self) -> bool {
        self.state.is_handshake_finished()
    }

    /// Returns `true` if it is our turn to write a message.
    pub fn is_my_turn(&self) -> bool {
        self.state.is_my_turn()
    }

    /// Consume the state to extract the SecureSession and PeerIdentity.
    /// MUST be called only after `is_finished() == true`.
    pub fn into_secure_session(self) -> Result<(SecureSession, PeerIdentity, Vec<u8>)> {
        if !self.is_finished() {
            return Err(SwiftWaveError::HandshakeFailed);
        }

        let remote_static = self
            .state
            .get_remote_static()
            .ok_or(SwiftWaveError::HandshakeFailed)?;

        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(remote_static);

        let transcript = self.state.get_handshake_hash().to_vec();

        let peer_identity = PeerIdentity::from_public_key(public_key, "Unknown Peer");
        let transport = self
            .state
            .into_transport_mode()
            .map_err(|_| SwiftWaveError::HandshakeFailed)?;

        Ok((SecureSession::new(transport), peer_identity, transcript))
    }
}

// ---------------------------------------------------------------------------
// Adversarial Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::sas::SASGenerator;
    use rand_core::OsRng;
    use x25519_dalek::{PublicKey, StaticSecret};

    fn generate_keypair() -> ([u8; 32], [u8; 32]) {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        let mut sec_bytes = [0u8; 32];
        sec_bytes.copy_from_slice(secret.as_bytes());
        (sec_bytes, *public.as_bytes())
    }

    // Helper to simulate a network transport between two states.
    fn drive_handshake(init: &mut HandshakeState, resp: &mut HandshakeState) -> Result<()> {
        let mut msg = vec![0u8; MAX_MESSAGE];
        let mut out = vec![0u8; MAX_MESSAGE];

        while !init.is_finished() || !resp.is_finished() {
            if init.is_my_turn() {
                let len = init.write_message(&[], &mut msg)?;
                resp.read_message(&msg[..len], &mut out)?;
            } else if resp.is_my_turn() {
                let len = resp.write_message(&[], &mut msg)?;
                init.read_message(&msg[..len], &mut out)?;
            }
        }
        Ok(())
    }

    #[test]
    fn test_two_peers_successfully_authenticating() {
        let (init_sec, init_pub) = generate_keypair();
        let (resp_sec, resp_pub) = generate_keypair();

        let mut init = HandshakeState::new_initiator(&init_sec).unwrap();
        let mut resp = HandshakeState::new_responder(&resp_sec).unwrap();

        drive_handshake(&mut init, &mut resp).expect("Handshake should succeed");

        let (_, init_peer_id, init_transcript) = init.into_secure_session().unwrap();
        let (_, resp_peer_id, resp_transcript) = resp.into_secure_session().unwrap();

        // Verify mutual authentication
        assert_eq!(init_peer_id.public_key, resp_pub);
        assert_eq!(resp_peer_id.public_key, init_pub);

        // Transcript hash must match for SAS
        assert_eq!(init_transcript, resp_transcript);

        // SAS must match
        let init_sas = SASGenerator::derive_sas_hash(&init_pub, &resp_pub, &init_transcript);
        let resp_sas = SASGenerator::derive_sas_hash(&init_pub, &resp_pub, &resp_transcript);
        assert_eq!(init_sas, resp_sas);
    }

    #[test]
    fn test_authentication_failure_altered_public_key() {
        let (init_sec, init_pub) = generate_keypair();
        let (resp_sec, resp_pub) = generate_keypair();
        let (mitm_sec, _) = generate_keypair();

        let mut init = HandshakeState::new_initiator(&init_sec).unwrap();
        let mut resp = HandshakeState::new_responder(&resp_sec).unwrap();

        // MITM modifies the first message
        let mut msg = vec![0u8; MAX_MESSAGE];
        let mut out = vec![0u8; MAX_MESSAGE];
        let len = init.write_message(&[], &mut msg).unwrap();

        // Alter payload
        msg[0] ^= 0xff;

        // Responder reads the tampered first message (this might succeed since 'e' is unauthenticated in XX first message)
        let res1 = resp.read_message(&msg[..len], &mut out);

        if res1.is_ok() {
            // If it succeeded, the handshake must fail on the next message
            let len2 = resp.write_message(&[], &mut msg).unwrap();
            let res2 = init.read_message(&msg[..len2], &mut out);
            assert!(matches!(res2, Err(SwiftWaveError::HandshakeFailed)));
        } else {
            assert!(matches!(res1, Err(SwiftWaveError::HandshakeFailed)));
        }
    }

    #[test]
    fn test_wrong_sas_detection() {
        let (init_sec, init_pub) = generate_keypair();
        let (resp_sec, resp_pub) = generate_keypair();

        let mut init = HandshakeState::new_initiator(&init_sec).unwrap();
        let mut resp = HandshakeState::new_responder(&resp_sec).unwrap();

        drive_handshake(&mut init, &mut resp).unwrap();

        let (_, _, init_transcript) = init.into_secure_session().unwrap();
        let (_, _, resp_transcript) = resp.into_secure_session().unwrap();

        // MitM attempts to substitute a public key but cannot fake the transcript
        let fake_pub = generate_keypair().1;

        let init_sas = SASGenerator::derive_sas_hash(&init_pub, &resp_pub, &init_transcript);
        let fake_sas = SASGenerator::derive_sas_hash(&init_pub, &fake_pub, &resp_transcript);

        assert_ne!(init_sas, fake_sas);
    }

    #[test]
    fn test_replayed_handshake_data() {
        let (init_sec, _) = generate_keypair();
        let (resp_sec, _) = generate_keypair();

        let mut init = HandshakeState::new_initiator(&init_sec).unwrap();
        let mut resp = HandshakeState::new_responder(&resp_sec).unwrap();

        let mut msg = vec![0u8; MAX_MESSAGE];
        let mut out = vec![0u8; MAX_MESSAGE];
        let len = init.write_message(&[], &mut msg).unwrap();

        // Responder reads first message
        resp.read_message(&msg[..len], &mut out).unwrap();

        // Replay attack: MITM sends the exact same first message again
        let replay_res = resp.read_message(&msg[..len], &mut out);

        // Noise state machine rejects out-of-order or duplicate messages for its current state
        assert!(replay_res.is_err());
    }

    #[test]
    fn test_invalid_message_boundaries() {
        let (resp_sec, _) = generate_keypair();
        let mut resp = HandshakeState::new_responder(&resp_sec).unwrap();
        let mut out = vec![0u8; MAX_MESSAGE];

        // Too short / garbage
        assert!(resp.read_message(&[0x00, 0x01, 0x02], &mut out).is_err());

        // Exceeds max
        assert!(resp
            .read_message(&vec![0x00; MAX_MESSAGE + 1], &mut out)
            .is_err());
    }
    #[tokio::test]
    async fn test_async_handshake() {
        let (init_sec, init_pub) = generate_keypair();
        let (resp_sec, resp_pub) = generate_keypair();

        let mut init = HandshakeState::new_initiator(&init_sec).unwrap();
        let mut resp = HandshakeState::new_responder(&resp_sec).unwrap();

        let (mut client, mut server) = tokio::io::duplex(65536);

        let init_task = tokio::spawn(async move {
            let mut out = vec![0u8; MAX_MESSAGE];
            while !init.is_finished() {
                if init.is_my_turn() {
                    init.write_framed(&mut client, &mut out).await.unwrap();
                } else {
                    init.read_framed(&mut client, &mut out).await.unwrap();
                }
            }
            init.into_secure_session().unwrap()
        });

        let resp_task = tokio::spawn(async move {
            let mut out = vec![0u8; MAX_MESSAGE];
            while !resp.is_finished() {
                if resp.is_my_turn() {
                    resp.write_framed(&mut server, &mut out).await.unwrap();
                } else {
                    resp.read_framed(&mut server, &mut out).await.unwrap();
                }
            }
            resp.into_secure_session().unwrap()
        });

        let ((_, init_peer, _), (_, resp_peer, _)) =
            tokio::try_join!(init_task, resp_task).unwrap();

        assert_eq!(init_peer.public_key, resp_pub);
        assert_eq!(resp_peer.public_key, init_pub);
    }

    #[tokio::test]
    async fn test_framing_normal_cases() {
        let (mut client, mut server) = tokio::io::duplex(65536);

        // 1. Empty payload
        write_framed_message(&mut client, &[]).await.unwrap();
        let res = read_framed_message(&mut server).await.unwrap();
        assert_eq!(res.len(), 0);

        // 2. One-byte payload
        write_framed_message(&mut client, &[42]).await.unwrap();
        let res = read_framed_message(&mut server).await.unwrap();
        assert_eq!(res, vec![42]);

        // 3. Typical Noise-sized payload (approx 96 bytes)
        let typical = vec![0xab; 96];
        write_framed_message(&mut client, &typical).await.unwrap();
        let res = read_framed_message(&mut server).await.unwrap();
        assert_eq!(res, typical);

        // 4. Payload exactly at maximum
        let max_payload = vec![0xcd; MAX_MESSAGE];
        write_framed_message(&mut client, &max_payload)
            .await
            .unwrap();
        let res = read_framed_message(&mut server).await.unwrap();
        assert_eq!(res, max_payload);

        // 5. Multiple consecutive frames
        write_framed_message(&mut client, &[1, 2]).await.unwrap();
        write_framed_message(&mut client, &[3, 4, 5]).await.unwrap();
        let r1 = read_framed_message(&mut server).await.unwrap();
        let r2 = read_framed_message(&mut server).await.unwrap();
        assert_eq!(r1, vec![1, 2]);
        assert_eq!(r2, vec![3, 4, 5]);
    }

    #[tokio::test]
    async fn test_framing_boundaries() {
        // 6. One byte over maximum -> reject
        let (mut client, _) = tokio::io::duplex(65536);
        let oversized = vec![0xef; MAX_MESSAGE + 1];
        let err = write_framed_message(&mut client, &oversized)
            .await
            .unwrap_err();
        assert!(matches!(err, SwiftWaveError::FrameTooLarge));

        // 7. Header split across multiple reads
        let (mut client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            client.write_all(&[0x00]).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            client.write_all(&[0x02, 0xaa, 0xbb]).await.unwrap();
        });
        let res = read_framed_message(&mut server).await.unwrap();
        assert_eq!(res, vec![0xaa, 0xbb]);

        // 8. Payload split across many reads
        let (mut client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            client.write_all(&[0x00, 0x03, 0x11]).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            client.write_all(&[0x22]).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            client.write_all(&[0x33]).await.unwrap();
        });
        let res = read_framed_message(&mut server).await.unwrap();
        assert_eq!(res, vec![0x11, 0x22, 0x33]);

        // 9. EOF during first header byte
        let (mut client, mut server) = tokio::io::duplex(65536);
        drop(client);
        let err = read_framed_message(&mut server).await.unwrap_err();
        assert!(matches!(err, SwiftWaveError::UnexpectedEof));

        // 10. EOF after first header byte
        let (mut client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            client.write_all(&[0x00]).await.unwrap();
        });
        let err = read_framed_message(&mut server).await.unwrap_err();
        assert!(matches!(err, SwiftWaveError::UnexpectedEof));

        // 11. EOF in the middle of payload
        let (mut client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            client.write_all(&[0x00, 0x04, 0xaa, 0xbb]).await.unwrap();
        });
        let err = read_framed_message(&mut server).await.unwrap_err();
        assert!(matches!(err, SwiftWaveError::UnexpectedEof));

        // 12. Declared length larger than remaining stream
        // (same as 11, handled correctly by read_exact throwing UnexpectedEof)

        // 13. Zero-length frame handling
        // (tested in test_framing_normal_cases)
    }

    #[tokio::test]
    async fn test_framing_malicious_input() {
        // 14. Declared oversized length
        let (mut client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            // Declare length MAX_MESSAGE + 1
            let len = (MAX_MESSAGE + 1) as u16;
            client.write_all(&len.to_be_bytes()).await.unwrap();
        });
        let err = read_framed_message(&mut server).await.unwrap_err();
        assert!(matches!(err, SwiftWaveError::FrameTooLarge));

        // 15. Repeated large frames must not cause unbounded allocation
        // Handled because read_framed_message returns error on length > MAX_MESSAGE

        // 16. Timeout while waiting for header
        tokio::time::pause();
        let (_client, mut server) = tokio::io::duplex(65536);
        let err = read_framed_message(&mut server).await.unwrap_err();
        assert!(matches!(err, SwiftWaveError::HandshakeTimeout));
        tokio::time::resume();

        // 17. Timeout while waiting for payload
        tokio::time::pause();
        let (mut client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            client.write_all(&[0x00, 0x05, 0xaa]).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        });
        let err = read_framed_message(&mut server).await.unwrap_err();
        assert!(matches!(err, SwiftWaveError::HandshakeTimeout));
        tokio::time::resume();
    }

    #[tokio::test]
    async fn test_framing_write_partial() {
        // 18. Partial writer must still receive complete frame
        let (mut client, mut server) = tokio::io::duplex(10); // Small buffer to force partial writes internally
        tokio::spawn(async move {
            let msg = vec![0xcc; 50];
            write_framed_message(&mut client, &msg).await.unwrap();
        });
        let res = read_framed_message(&mut server).await.unwrap();
        assert_eq!(res.len(), 50);

        // 19. Oversized write must fail before writing a partial frame
        let (mut client, mut server) = tokio::io::duplex(65536);
        let oversized = vec![0xef; MAX_MESSAGE + 1];
        let err = write_framed_message(&mut client, &oversized)
            .await
            .unwrap_err();
        assert!(matches!(err, SwiftWaveError::FrameTooLarge));

        // Ensure nothing was written
        tokio::time::pause();
        let read_err = read_framed_message(&mut server).await.unwrap_err();
        assert!(matches!(read_err, SwiftWaveError::HandshakeTimeout));
        tokio::time::resume();
    }
}
