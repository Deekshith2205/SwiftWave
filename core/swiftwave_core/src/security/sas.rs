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
//!
//! # Algorithm
//! 1. Both peers hash: `BLAKE3(local_pk || remote_pk || session_transcript)`.
//! 2. The resulting 32 bytes are mapped to a short code:
//!    - **Numeric**: two 4-digit groups, e.g. `7312 9041`.
//!    - **Emoji**: four emoji from a fixed 256-emoji palette.
//!
//! # Security note
//! The SAS provides ~20 bits of security against MITM attacks (1-in-1 000 000
//! chance of a false positive). This is sufficient for interactive
//! short-window verification but is not a substitute for a proper PKI.
//!
//! Users MUST be instructed to verify the SAS on a trusted side-channel
//! (face-to-face, phone call) before accepting a transfer.

use crate::security::hashing::hash_bytes;

/// Derive the SAS commitment hash from both peers' static public keys and
/// the handshake transcript hash provided by the Noise state machine.
///
/// Both sides must call this function with the keys in the SAME order:
/// always `(local_pk, remote_pk)` relative to the *initiator*.
/// The responder must swap the argument order accordingly.
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
///
/// Uses the first 4 bytes of the hash to produce two 0–9999 values.
/// Total entropy: log2(10000 * 10000) ≈ 26.6 bits.
pub fn sas_numeric(hash: &[u8; 32]) -> String {
    let a = u16::from_be_bytes([hash[0], hash[1]]) % 10_000;
    let b = u16::from_be_bytes([hash[2], hash[3]]) % 10_000;
    format!("{a:04} {b:04}")
}

/// Format the SAS as four emoji from a fixed 64-symbol palette.
///
/// Each of the first 4 bytes (mod 64) selects one emoji.
/// Total entropy: log2(64^4) = 24 bits.
pub fn sas_emoji(hash: &[u8; 32]) -> String {
    // 64 visually-distinct, cross-platform emoji (no skin-tone variants).
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn fake_pk(seed: u8) -> [u8; 32] {
        [seed; 32]
    }

    #[test]
    fn sas_hash_is_deterministic() {
        let init_pk = fake_pk(0x01);
        let resp_pk = fake_pk(0x02);
        let transcript = b"fake transcript";

        let h1 = derive_sas_hash(&init_pk, &resp_pk, transcript);
        let h2 = derive_sas_hash(&init_pk, &resp_pk, transcript);
        assert_eq!(h1, h2);
    }

    #[test]
    fn sas_hash_differs_with_different_keys() {
        let h1 = derive_sas_hash(&fake_pk(0x01), &fake_pk(0x02), b"t");
        let h2 = derive_sas_hash(&fake_pk(0x03), &fake_pk(0x04), b"t");
        assert_ne!(h1, h2);
    }

    #[test]
    fn sas_hash_is_not_symmetric() {
        // Order matters: initiator/responder must not produce the same hash
        // when swapped, or a MITM could confuse both sides.
        let h1 = derive_sas_hash(&fake_pk(0x01), &fake_pk(0x02), b"t");
        let h2 = derive_sas_hash(&fake_pk(0x02), &fake_pk(0x01), b"t");
        assert_ne!(h1, h2);
    }

    #[test]
    fn numeric_sas_format() {
        let hash = [0u8; 32];
        let sas = sas_numeric(&hash);
        assert_eq!(sas.len(), 9); // "0000 0000"
        assert!(sas.chars().all(|c| c.is_ascii_digit() || c == ' '));
    }

    #[test]
    fn emoji_sas_has_four_parts() {
        let hash = [42u8; 32];
        let sas = sas_emoji(&hash);
        let parts: Vec<&str> = sas.split(' ').collect();
        assert_eq!(parts.len(), 4);
    }
}
