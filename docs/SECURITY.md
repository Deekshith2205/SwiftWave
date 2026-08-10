# SwiftWave — Security Design

## Threat Model

| Threat | Mitigation |
|--------|-----------|
| Passive eavesdropping on Wi-Fi | ChaCha20-Poly1305 / AES-256-GCM encryption for all data |
| Active MITM during transfer | Noise_XX mutual authentication; TOFU key pinning |
| Malicious peer impersonation | Device identity bound to X25519 keypair; key fingerprint visible to user |
| Compromised session key | Perfect forward secrecy via ephemeral X25519; session keys never reused |
| Private key theft from OS | Keys stored in platform secure enclave (Keystore / Keychain / DPAPI) |
| File integrity tampering | BLAKE3 hash verified per chunk and for whole file |
| Replay attacks | Noise protocol includes nonce progression; replayed packets rejected |
| Denial of service | Rate limiting on incoming connection attempts (Phase 3) |

---

## Protocol Stack

```
┌─────────────────────────────────────────┐
│         Application Data (chunks)        │
├─────────────────────────────────────────┤
│  ChaCha20-Poly1305 / AES-256-GCM AEAD   │  ← Session layer
├─────────────────────────────────────────┤
│  Noise_XX Handshake (snow crate)         │  ← Mutual auth + key exchange
├─────────────────────────────────────────┤
│  QUIC (Quinn) — TLS 1.3                 │  ← Transport encryption (defence in depth)
├─────────────────────────────────────────┤
│  UDP / IP                                │
└─────────────────────────────────────────┘
```

Note: QUIC already includes TLS 1.3. The additional Noise layer provides
end-to-end application-level authentication independent of the transport.

---

## Noise_XX Handshake Pattern

SwiftWave uses **Noise_XX_25519_ChaChaPoly_BLAKE2s** (or AES-GCM variant):

```
→ e
← e, ee, s, es
→ s, se
```

- `→ e` — initiator sends ephemeral key
- `← e, ee, s, es` — responder sends ephemeral + static, mixes ECDH
- `→ s, se` — initiator sends static + final ECDH mix

Result: both parties are mutually authenticated; a shared 256-bit session key is derived.

---

## Cryptographic Primitives

| Primitive | Crate | Use |
|-----------|-------|-----|
| X25519 | `x25519-dalek` | Ephemeral + static key exchange in Noise |
| ChaCha20-Poly1305 | `chacha20poly1305` | Preferred AEAD (software-friendly) |
| AES-256-GCM | `aes-gcm` | Fallback AEAD (hardware-accelerated) |
| BLAKE3 | `blake3` | File and chunk integrity hashing |
| BLAKE2s | (via snow) | Noise handshake hash function |
| UUID v4 | `uuid` | Temporary DeviceId (Phase 1); replaced by public key hash in Phase 2 |

---

## Key Storage

| Platform | Mechanism |
|----------|-----------|
| Android | Android Keystore System (`KeyPairGenerator` with `AndroidKeyStore` provider) |
| macOS | Keychain Services (`security-framework` crate) |
| Windows | Data Protection API (`DPAPI` — `CryptProtectData`) |
| Linux | libsecret / GNOME Keyring (`secret-service` crate) |

Private keys **never** appear in log output, crash reports, or analytics.

---

## Trust-on-First-Use (TOFU)

1. On first connection to a peer, the peer's X25519 public key is displayed as a
   short human-readable fingerprint (emoji or word list — Phase 4).
2. The user can optionally verify the fingerprint out-of-band (e.g., by reading it aloud).
3. After acceptance, the key is stored locally. Subsequent connections to the same
   `DeviceId` must use the same key; a mismatch triggers a prominent warning.

---

## Out-of-Scope Security Concerns

- **Physical access attacks** — if an attacker has physical access to the device,
  OS-level protections apply (screen lock, disk encryption).
- **Long-term key compromise** — if a device's private key is exfiltrated, past
  sessions with PFS are still safe; future sessions require re-keying.
- **iOS support** — not in scope for current phases.

---

## Responsible Disclosure

To report a security vulnerability, email: **security@swiftwave.example** (placeholder).
Please do not open public GitHub issues for security matters.
