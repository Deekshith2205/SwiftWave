package com.swiftwave.native

/**
 * SwiftWaveDiscovery — Kotlin interface for Android peer discovery backends.
 *
 * Implementations are selected at runtime based on device capabilities:
 * 1. Wi-Fi Aware (preferred — lowest latency, no infrastructure)
 * 2. Wi-Fi Direct (fallback — requires group negotiation)
 * 3. BLE GATT (last resort — low throughput but maximum range)
 *
 * TODO (Phase 2): Implement WifiAwareDiscovery, WifiDirectDiscovery, BleDiscovery.
 * TODO (Phase 2): Wire to swiftwave_core via JNI (SwiftWaveJni.kt).
 */
interface SwiftWaveDiscovery {

    /**
     * Start advertising this device and scanning for peers.
     *
     * @param deviceId Stable device identifier from swiftwave_core identity module.
     * @param displayName Human-readable name to broadcast.
     * @param listener Callback interface for discovery events.
     */
    fun start(deviceId: String, displayName: String, listener: DiscoveryListener)

    /** Stop all discovery activities and release Wi-Fi / BLE resources. */
    fun stop()

    /** Human-readable name of this backend (e.g. "wifi_aware", "ble"). */
    val backendName: String
}

/**
 * Callback interface for peer discovery events.
 * Delivered on the main thread unless otherwise noted.
 */
interface DiscoveryListener {
    /** Called when a new SwiftWave peer is found. */
    fun onPeerFound(peer: DiscoveredPeer)

    /** Called when a previously visible peer disappears. */
    fun onPeerLost(deviceId: String)

    /** Called on non-fatal discovery errors. */
    fun onError(message: String)
}

/**
 * Data class representing a discovered peer device.
 *
 * @param deviceId     Stable identifier from swiftwave_core.
 * @param displayName  Human-readable name announced by the peer.
 * @param address      Transport address (IP:port for QUIC, MAC for BLE).
 * @param rssi         Signal strength in dBm, if available.
 * @param backendName  Which discovery backend found this peer.
 */
data class DiscoveredPeer(
    val deviceId: String,
    val displayName: String,
    val address: String,
    val rssi: Int?,
    val backendName: String,
)

/**
 * Factory that selects the best available discovery backend at runtime.
 *
 * TODO (Phase 2): Implement capability checks and return real backends.
 */
object SwiftWaveDiscoveryFactory {

    /**
     * Returns the best discovery backend available on this device.
     *
     * Priority: Wi-Fi Aware → Wi-Fi Direct → BLE
     *
     * @param context Android application context.
     * @return The selected [SwiftWaveDiscovery] implementation.
     */
    @JvmStatic
    fun create(context: android.content.Context): SwiftWaveDiscovery {
        // TODO (Phase 2): Check WifiAwareManager.isAvailable()
        // TODO (Phase 2): Check WifiP2pManager availability
        // TODO (Phase 2): Check BluetoothAdapter.isEnabled()
        return StubDiscovery()
    }
}

/**
 * Stub discovery backend — emits no events. Used until real backends land.
 * MUST NOT be shipped in production.
 */
internal class StubDiscovery : SwiftWaveDiscovery {
    override val backendName = "stub"

    override fun start(deviceId: String, displayName: String, listener: DiscoveryListener) {
        // TODO (Phase 2): Replace with WifiAwareDiscovery.start()
        listener.onError("StubDiscovery: no real backend implemented yet")
    }

    override fun stop() {
        // TODO (Phase 2): Release resources
    }
}
