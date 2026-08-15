//! Device capability negotiation.
//!
//! Before any file transfer begins, two SwiftWave peers exchange their
//! `Capabilities` struct to agree on a common feature set. The intersection
//! of both peers' capabilities determines what is actually used.

use serde::{Deserialize, Serialize};

/// Transport backends a device can use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    /// QUIC over UDP (primary transport).
    Quic,
    /// TCP fallback (future Phase 3).
    Tcp,
    /// Wi-Fi Direct (Android — Phase 4).
    WifiDirect,
    /// Wi-Fi Aware (Android — Phase 4).
    WifiAware,
    /// Bluetooth Low Energy (Phase 4).
    Ble,
}

/// Compression codecs a device supports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionCodec {
    /// No compression.
    None,
    /// Zstandard (Phase 3).
    Zstd,
}

/// The set of features advertised by a SwiftWave device.
///
/// Serialised and exchanged during the capability-negotiation phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    /// Protocol version of this build.
    pub protocol_version: u16,

    /// Transports this device can initiate or accept.
    pub transports: Vec<Transport>,

    /// Compression codecs available on this device.
    pub compression: Vec<CompressionCodec>,

    /// Maximum chunk size this device can handle (bytes).
    pub max_chunk_size: usize,

    /// Whether this device supports transfer resumption.
    pub supports_resume: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            protocol_version: 1,
            transports: vec![Transport::Quic],
            compression: vec![CompressionCodec::None],
            max_chunk_size: crate::config::MAX_CHUNK_SIZE,
            supports_resume: true,
        }
    }
}

/// The result of intersecting two peers' `Capabilities`.
#[derive(Debug, Clone)]
pub struct NegotiatedCapabilities {
    /// Agreed transport.
    pub transport: Transport,
    /// Agreed compression codec.
    pub compression: CompressionCodec,
    /// Agreed chunk size (min of both peers' `max_chunk_size`, at least `MIN_CHUNK_SIZE`).
    pub chunk_size: usize,
    /// Whether both peers support resume.
    pub resume_enabled: bool,
}

/// Negotiate a common capability set from two peers' advertisements.
///
/// Returns `Err` if no common transport or protocol version can be agreed.
pub fn negotiate(
    local: &Capabilities,
    remote: &Capabilities,
) -> crate::error::Result<NegotiatedCapabilities> {
    if local.protocol_version != remote.protocol_version {
        return Err(crate::error::SwiftWaveError::CapabilityMismatch(format!(
            "Protocol version mismatch: local={} remote={}",
            local.protocol_version, remote.protocol_version
        )));
    }

    // Choose the highest-priority common transport.
    // Priority order: Quic > WifiDirect > WifiAware > Ble > Tcp
    let priority = [
        Transport::Quic,
        Transport::WifiDirect,
        Transport::WifiAware,
        Transport::Ble,
        Transport::Tcp,
    ];
    let transport = priority
        .iter()
        .find(|t| local.transports.contains(t) && remote.transports.contains(t))
        .cloned()
        .ok_or_else(|| {
            crate::error::SwiftWaveError::CapabilityMismatch(
                "No common transport".to_string(),
            )
        })?;

    // Choose the best common compression: Zstd > None
    let compression = if local.compression.contains(&CompressionCodec::Zstd)
        && remote.compression.contains(&CompressionCodec::Zstd)
    {
        CompressionCodec::Zstd
    } else {
        CompressionCodec::None
    };

    // Use the smaller of both peers' max chunk sizes, clamped to our min.
    let chunk_size = local
        .max_chunk_size
        .min(remote.max_chunk_size)
        .max(crate::config::MIN_CHUNK_SIZE);

    let resume_enabled = local.supports_resume && remote.supports_resume;

    Ok(NegotiatedCapabilities {
        transport,
        compression,
        chunk_size,
        resume_enabled,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_caps(transports: Vec<Transport>) -> Capabilities {
        Capabilities {
            transports,
            ..Default::default()
        }
    }

    #[test]
    fn negotiation_succeeds_with_common_quic() {
        let local = Capabilities::default();
        let remote = Capabilities::default();
        let n = negotiate(&local, &remote).unwrap();
        assert_eq!(n.transport, Transport::Quic);
        assert!(n.resume_enabled);
    }

    #[test]
    fn negotiation_prefers_quic_over_tcp() {
        let local = make_caps(vec![Transport::Quic, Transport::Tcp]);
        let remote = make_caps(vec![Transport::Tcp, Transport::Quic]);
        let n = negotiate(&local, &remote).unwrap();
        assert_eq!(n.transport, Transport::Quic);
    }

    #[test]
    fn negotiation_falls_back_to_tcp() {
        let local = make_caps(vec![Transport::Tcp]);
        let remote = make_caps(vec![Transport::Tcp]);
        let n = negotiate(&local, &remote).unwrap();
        assert_eq!(n.transport, Transport::Tcp);
    }

    #[test]
    fn negotiation_fails_with_no_common_transport() {
        let local = make_caps(vec![Transport::Quic]);
        let remote = make_caps(vec![Transport::Tcp]);
        assert!(negotiate(&local, &remote).is_err());
    }

    #[test]
    fn negotiation_fails_version_mismatch() {
        let mut remote = Capabilities::default();
        remote.protocol_version = 99;
        assert!(negotiate(&Capabilities::default(), &remote).is_err());
    }

    #[test]
    fn chunk_size_is_min_of_both() {
        let mut local = Capabilities::default();
        local.max_chunk_size = 1024 * 1024; // 1 MiB
        let mut remote = Capabilities::default();
        remote.max_chunk_size = 512 * 1024; // 512 KiB
        let n = negotiate(&local, &remote).unwrap();
        assert_eq!(n.chunk_size, 512 * 1024);
    }

    #[test]
    fn capabilities_serialise_roundtrip() {
        let caps = Capabilities::default();
        let json = serde_json::to_string(&caps).unwrap();
        let back: Capabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(back.protocol_version, caps.protocol_version);
    }
}
