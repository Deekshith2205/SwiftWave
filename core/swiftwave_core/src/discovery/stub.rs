//! `StubDiscovery` — a no-op discovery backend used in tests.
//!
//! Emits a synthetic `PeerFound` event after a short delay so that unit
//! tests can exercise the event pipeline without real networking.

use super::{DiscoveredPeer, DiscoveryEvent, Discovery};
use crate::{
    identity::{DeviceId, DeviceInfo},
    Result,
};
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

/// Stub discovery that emits synthetic events for testing.
pub struct StubDiscovery {
    running: Arc<AtomicBool>,
}

impl StubDiscovery {
    /// Create a new `StubDiscovery`.
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for StubDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Discovery for StubDiscovery {
    async fn start(
        &self,
        local_info: &DeviceInfo,
        tx: tokio::sync::mpsc::Sender<DiscoveryEvent>,
    ) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);

        info!(
            device_id = %local_info.id,
            "StubDiscovery: starting synthetic peer discovery"
        );

        // Spawn a background task that emits one fake peer after 100 ms.
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            if running.load(Ordering::SeqCst) {
                let fake_peer = DiscoveredPeer {
                    device_info: DeviceInfo {
                        id: DeviceId::generate(),
                        display_name: "Stub Peer Device".into(),
                        app_version: "0.1.0".into(),
                    },
                    address: "127.0.0.1:7777".into(),
                    rssi: Some(-65),
                };

                let _ = tx.send(DiscoveryEvent::PeerFound(fake_peer)).await;
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        info!("StubDiscovery: stopped");
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "stub"
    }
}
