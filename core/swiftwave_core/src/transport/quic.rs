//! QUIC transport implementation using `quinn`.
//!
//! # TLS and security layering
//! QUIC requires a TLS 1.3 certificate. Since SwiftWave does not use a CA,
//! we generate a self-signed certificate per device. The certificate's public
//! key is **not** used for peer authentication — that is handled by the Noise
//! XX handshake. The TLS layer is present only because QUIC mandates it.
//!
//! # Security note
//! We disable TLS certificate verification at the QUIC layer (`dangerous`)
//! and instead rely on the Noise handshake for all authentication.
//! This is intentional and well-documented. The Noise layer provides stronger
//! guarantees (mutual auth, forward secrecy, identity hiding) than TLS alone.
//!
//! # Binding
//! The endpoint binds to `0.0.0.0:0` by default (OS-assigned port).
//! Platform adapters may override the port for NAT traversal or firewall rules.

use std::net::SocketAddr;
use std::sync::Arc;

use quinn::{ClientConfig, Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

use crate::error::{Result, SwiftWaveError};

/// Generate a self-signed TLS certificate and private key for the QUIC layer.
///
/// # Security note
/// This certificate is used solely to satisfy QUIC's TLS requirement.
/// Peer identity is authenticated via Noise XX — not via this certificate.
pub fn generate_self_signed_cert() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["swiftwave.local".to_string()])
        .map_err(|e| SwiftWaveError::Internal(format!("TLS cert generation failed: {e}")))?;

    let cert_der = CertificateDer::from(cert.serialize_der()
        .map_err(|e| SwiftWaveError::Internal(format!("TLS cert serialization failed: {e}")))?);
    let key_der = PrivateKeyDer::try_from(cert.serialize_private_key_der())
        .map_err(|e| SwiftWaveError::Internal(format!("TLS key serialization failed: {e}")))?;

    Ok((cert_der, key_der))
}

/// Build a QUIC server `Endpoint` bound to `bind_addr`.
///
/// The endpoint accepts incoming QUIC connections from any peer.
pub async fn build_server_endpoint(bind_addr: SocketAddr) -> Result<Endpoint> {
    let (cert_der, key_der) = generate_self_signed_cert()?;

    let server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .map_err(|e| SwiftWaveError::Internal(format!("TLS server config: {e}")))?;

    let server_config = ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto)
            .map_err(|e| SwiftWaveError::Internal(format!("QUIC server config: {e}")))?
    ));

    Endpoint::server(server_config, bind_addr)
        .map_err(|e| SwiftWaveError::QuicConnection(e.to_string()))
}

/// Build a QUIC client `Endpoint` bound to `0.0.0.0:0`.
///
/// Certificate verification is disabled at the QUIC layer — see module docs.
pub async fn build_client_endpoint() -> Result<Endpoint> {
    // SAFETY: We intentionally skip TLS verification here because peer
    // authentication is handled by the Noise XX handshake layer.
    // This is documented and auditable.
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoopServerVerifier))
        .with_no_client_auth();

    let client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto)
            .map_err(|e| SwiftWaveError::Internal(format!("QUIC client config: {e}")))?
    ));

    let mut endpoint =
        Endpoint::client("0.0.0.0:0".parse().unwrap())
            .map_err(|e| SwiftWaveError::QuicConnection(e.to_string()))?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

/// A TLS certificate verifier that accepts any certificate.
///
/// # Security note
/// This is intentionally permissive. The Noise handshake (run immediately
/// after the QUIC connection is established) provides all security guarantees.
/// Accepting arbitrary TLS certificates does NOT create a security hole
/// because Noise authenticates both parties independently.
#[derive(Debug)]
struct NoopServerVerifier;

impl rustls::client::danger::ServerCertVerifier for NoopServerVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
