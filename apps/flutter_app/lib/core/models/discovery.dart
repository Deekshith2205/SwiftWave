enum DiscoveryMedium {
  mdnsUdp,
  wifiAware,
  wifiDirect,
  ble,
  manual,
}

class DiscoveredPeer {
  final String fingerprint;
  final String displayName;
  final String address;
  final DiscoveryMedium medium;
  final int? rssi;
  final int protocolVersion;
  final int lastSeen;

  const DiscoveredPeer({
    required this.fingerprint,
    required this.displayName,
    required this.address,
    required this.medium,
    this.rssi,
    required this.protocolVersion,
    required this.lastSeen,
  });
}

abstract class DiscoveryEvent {
  const DiscoveryEvent();

  factory DiscoveryEvent.peerFound(DiscoveredPeer peer) = PeerFound;
  factory DiscoveryEvent.peerLost(String fingerprint) = PeerLost;
}

class PeerFound extends DiscoveryEvent {
  final DiscoveredPeer peer;
  const PeerFound(this.peer);
}

class PeerLost extends DiscoveryEvent {
  final String fingerprint;
  const PeerLost(this.fingerprint);
}
