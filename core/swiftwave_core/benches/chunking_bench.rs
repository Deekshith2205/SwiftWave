use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use swiftwave_core::security::hashing::hash_chunk;
use swiftwave_core::transfer::chunk::generate_chunk_plan;

fn bench_chunking(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_chunking");

    for (file_size, chunk_size) in [
        (1024 * 1024u64, 256 * 1024usize),     // 1 MiB file, 256 KiB chunks
        (100 * 1024 * 1024, 256 * 1024),        // 100 MiB file, 256 KiB chunks
    ] {
        let data = vec![0x55u8; file_size as usize];
        let label = format!("{}mib_{}k_chunks", file_size / 1024 / 1024, chunk_size / 1024);
        group.throughput(Throughput::Bytes(file_size));
        group.bench_function(&label, |b| {
            b.iter(|| {
                // Hash each chunk (this is the hot path during sends).
                let hashes: Vec<_> = data
                    .chunks(chunk_size)
                    .map(|c| hash_chunk(black_box(c)))
                    .collect();
                let plan = generate_chunk_plan("tid", "fid", file_size, chunk_size, &hashes);
                black_box(plan)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_chunking);
criterion_main!(benches);
