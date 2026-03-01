//! Performance benchmarks
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_file_service(c: &mut Criterion) {
    // TODO: Add benchmarks
}

criterion_group!(benches, bench_file_service);
criterion_main!(benches);

