use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::io::Write;
use tempfile::NamedTempFile;

fn bench_io_read_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_read_throughput");

    // Create a 10 MiB temp file.
    let file_size = 10 * 1024 * 1024u64;
    let mut tmp = NamedTempFile::new().unwrap();
    tmp.write_all(&vec![0xBBu8; file_size as usize]).unwrap();
    tmp.flush().unwrap();
    let path = tmp.path().to_path_buf();

    group.throughput(Throughput::Bytes(file_size));
    group.bench_function("10mib_read_256k_chunks", |b| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();
        b.iter(|| {
            rt.block_on(async {
                use swiftwave_core::storage::file_reader::ChunkReader;
                let mut reader = ChunkReader::open(&path, 256 * 1024).await.unwrap();
                let mut count = 0usize;
                while let Some(chunk) = reader.next_chunk().await.unwrap() {
                    count += chunk.len();
                }
                count
            })
        });
    });
    group.finish();
}

criterion_group!(benches, bench_io_read_throughput);
criterion_main!(benches);
