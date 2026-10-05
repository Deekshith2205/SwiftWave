class AuthenticatedPeer {
  final String fingerprint;
  final String displayName;

  const AuthenticatedPeer({
    required this.fingerprint,
    required this.displayName,
  });
}

abstract class AuthenticatedPeerEvent {
  const AuthenticatedPeerEvent();

  factory AuthenticatedPeerEvent.incoming(AuthenticatedPeer peer) =
      IncomingAuthenticatedPeer;
}

class IncomingAuthenticatedPeer extends AuthenticatedPeerEvent {
  final AuthenticatedPeer peer;
  const IncomingAuthenticatedPeer(this.peer);
}
