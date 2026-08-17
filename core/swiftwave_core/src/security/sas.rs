//! Short Authentication String (SAS) generation for peer verification.
//!
//! # Purpose
//! After the Noise handshake, both peers have each other's static public keys.
//! However, without further out-of-band verification, a MITM could have
//! substituted their own key during the handshake.
//!
//! The SAS gives users a simple human-verifiable commitment:
//! both devices display the same 4-word or numeric code derived from the
//! session transcript hash, and the user confirms visually that they match.

use crate::security::hashing::hash_bytes;

/// Encapsulates the generation of Short Authentication Strings (SAS).
pub struct SASGenerator;

impl SASGenerator {
    /// Derive the SAS commitment hash from both peers' static public keys and
    /// the handshake transcript hash provided by the Noise state machine.
    ///
    /// Both sides must call this function with the keys in the SAME order:
    /// always `(initiator_pk, responder_pk)`.
    pub fn derive_sas_hash(
        initiator_pk: &[u8; 32],
        responder_pk: &[u8; 32],
        transcript_hash: &[u8],
    ) -> [u8; 32] {
        // Domain-separated input to prevent cross-protocol attacks.
        let mut input = Vec::with_capacity(2 + 32 + 32 + transcript_hash.len());
        input.extend_from_slice(b"swiftwave-sas-v1:");
        input.extend_from_slice(initiator_pk);
        input.extend_from_slice(responder_pk);
        input.extend_from_slice(transcript_hash);
        hash_bytes(&input)
    }

    /// Format the SAS as two 4-digit numeric groups (e.g. `"7312 9041"`).
    pub fn numeric(hash: &[u8; 32]) -> String {
        let a = u16::from_be_bytes([hash[0], hash[1]]) % 10_000;
        let b = u16::from_be_bytes([hash[2], hash[3]]) % 10_000;
        format!("{a:04} {b:04}")
    }

    /// Format the SAS as four emoji from a fixed 64-symbol palette.
    pub fn emoji(hash: &[u8; 32]) -> String {
        const PALETTE: [&str; 64] = [
            "🐶", "🐱", "🐭", "🐹", "🐰", "🦊", "🐻", "🐼",
            "🐨", "🐯", "🦁", "🐮", "🐷", "🐸", "🐵", "🐔",
            "🐧", "🐦", "🦆", "🦅", "🦉", "🦇", "🐺", "🐗",
            "🐴", "🦄", "🐝", "🐛", "🦋", "🐌", "🐞", "🐜",
            "🦟", "🦗", "🕷", "🦂", "🐢", "🐍", "🦎", "🦖",
            "🐙", "🦑", "🦐", "🦀", "🐡", "🐠", "🐟", "🐬",
            "🐳", "🦈", "🐊", "🐅", "🦓", "🦏", "🦛", "🐘",
            "🦒", "🦘", "🦙", "🐪", "🦔", "🌵", "🎄", "🌴",
        ];
        (0..4)
            .map(|i| PALETTE[hash[i] as usize % 64])
            .collect::<Vec<_>>()
            .join(" ")
    }
}
