/// The state of a file transfer, mirroring `TransferState` from swiftwave_core.
enum TransferStatus {
  /// Offer sent / received, waiting for acceptance.
  pending,

  /// Data is actively streaming.
  active,

  /// Transfer was paused.
  paused,

  /// Transfer completed successfully.
  completed,

  /// Transfer failed.
  failed,

  /// Transfer was cancelled.
  cancelled,
}

/// Represents an in-progress or completed file transfer.
///
/// Mirrors the `FileOffer` + `TransferState` combination from swiftwave_core.
/// Will be populated from the FFI bridge in Phase 2.
class Transfer {
  /// Unique transfer session ID from swiftwave_core.
  final String id;

  /// Original filename.
  final String fileName;

  /// Total file size in bytes.
  final int fileSize;

  /// Number of bytes transferred so far.
  final int bytesTransferred;

  /// Direction: true if this device is the sender.
  final bool isSender;

  /// Remote peer device ID.
  final String remotePeerId;

  /// Current status of the transfer.
  final TransferStatus status;

  /// Optional error message if [status] is [TransferStatus.failed].
  final String? errorMessage;

  const Transfer({
    required this.id,
    required this.fileName,
    required this.fileSize,
    required this.bytesTransferred,
    required this.isSender,
    required this.remotePeerId,
    required this.status,
    this.errorMessage,
  });

  /// Transfer progress as a value between 0.0 and 1.0.
  double get progress =>
      fileSize > 0 ? (bytesTransferred / fileSize).clamp(0.0, 1.0) : 0.0;

  @override
  bool operator ==(Object other) =>
      identical(this, other) || (other is Transfer && other.id == id);

  @override
  int get hashCode => id.hashCode;

  @override
  String toString() =>
      'Transfer(id: $id, file: $fileName, status: $status)';
}
