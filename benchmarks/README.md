# Benchmarks

Throughput and latency benchmarks for the SwiftWave Share Rust core.

## Planned Benchmarks (Phase 3)

| Benchmark | Metric | Target |
|-----------|--------|--------|
| `chunk_hash` | BLAKE3 throughput | > 3 GB/s |
| `chunk_encrypt` | ChaCha20-Poly1305 throughput | > 2 GB/s |
| `file_split` | Chunk pipeline (read → hash → encrypt) | > 500 MB/s |
| `noise_handshake` | Handshake latency | < 2 ms |
| `quic_rtt` | Round-trip time on loopback | < 1 ms |

## Running Benchmarks (Phase 3)

```bash
# Requires nightly Rust for criterion integration
cargo +nightly bench --all
```

Results will be logged to `benchmarks/results/`.
