use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use swiftwave_core::security::hashing::{hash_bytes, StreamingHasher};

fn bench_blake3_oneshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_oneshot");

    for size in [4096usize, 65536, 262144, 1048576] {
        let data = vec![0xABu8; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(size),
            &data,
            |b, d| {
                b.iter(|| hash_bytes(black_box(d)));
            },
        );
    }
    group.finish();
}

fn bench_blake3_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_streaming");
    let total_size = 4 * 1024 * 1024usize; // 4 MiB
    let chunk_size = 256 * 1024usize;
    let data = vec![0x42u8; total_size];

    group.throughput(Throughput::Bytes(total_size as u64));
    group.bench_function("4mib_256k_chunks", |b| {
        b.iter(|| {
            let mut hasher = StreamingHasher::new();
            for chunk in data.chunks(chunk_size) {
                hasher.update(black_box(chunk));
            }
            black_box(hasher.finalize())
        });
    });
    group.finish();
}

criterion_group!(benches, bench_blake3_oneshot, bench_blake3_streaming);
criterion_main!(benches);
