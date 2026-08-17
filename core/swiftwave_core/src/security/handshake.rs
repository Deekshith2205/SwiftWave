//! Noise Protocol handshake and HandshakeState for SwiftWave.
//!
//! # Security properties
//! - **Mutual authentication**: Proves possession of static X25519 keys.
//! - **Forward secrecy**: Ephemeral keys are discarded after handshake.
//! - **Identity hiding**: Static keys are transmitted encrypted.

use snow::{Builder, HandshakeState as SnowHandshakeState};

use crate::error::{Result, SwiftWaveError};
use crate::security::session::SecureSession;
use crate::device::identity::{PeerIdentity, PublicKeyFingerprint};

const NOISE_PARAMS: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";
const MAX_MESSAGE: usize = 65535;

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
        Ok(Self { state: builder, is_initiator: true })
    }

    /// Initialize a new handshake state as the responder.
    pub fn new_responder(local_static_key: &[u8; 32]) -> Result<Self> {
        let builder = Builder::new(NOISE_PARAMS.parse().unwrap())
            .local_private_key(local_static_key)
            .build_responder()
            .map_err(|_| SwiftWaveError::HandshakeFailed)?;
        Ok(Self { state: builder, is_initiator: false })
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

        let remote_static = self.state
            .get_remote_static()
            .ok_or(SwiftWaveError::HandshakeFailed)?;
            
        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(remote_static);
        
        let transcript = self.state.get_handshake_hash();

        let peer_identity = PeerIdentity::from_public_key(public_key, "Unknown Peer");
        let transport = self.state
            .into_transport_mode()
            .map_err(|_| SwiftWaveError::HandshakeFailed)?;

        Ok((SecureSession::new(transport), peer_identity, transcript.to_vec()))
    }
}

// ---------------------------------------------------------------------------
// Adversarial Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use x25519_dalek::{StaticSecret, PublicKey};
    use rand_core::OsRng;
    use crate::security::sas::SASGenerator;

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
        
        // Responder should reject the tampered message
        let res = resp.read_message(&msg[..len], &mut out);
        assert!(matches!(res, Err(SwiftWaveError::HandshakeFailed)));
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
        assert!(resp.read_message(&vec![0x00; MAX_MESSAGE + 1], &mut out).is_err());
    }
}
