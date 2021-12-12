use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use bench::{Buffer, external_compare, internal_compare, BufferTest};

/// Benchmarks a single query with the copy time not included
/// This measures the raw comparison speed
#[allow(dead_code)]
pub fn buf_benchmark_single_query(c: &mut Criterion) {
    let query_size = 256;
    let repeat_len = 32;
    let match_lens = [64, 128, 192, 256];

    let mut buf = Buffer::new(1024 * 8);
    buf.head = buf.buf.len() - query_size;

    let tests = match_lens.map(|match_len| {
        BufferTest::default()
            .query_size(query_size)
            .overlapping_match(repeat_len, match_len)
            .setup(&buf)
    });

    let mut group = c.benchmark_group("direct_compare");
    for (dist, len, query) in tests.iter() {
        group.throughput(Throughput::Bytes(*len as u64));
        group.bench_with_input(BenchmarkId::from_parameter(len), dist, |b, dist: &usize| {
            b.iter(|| {
                assert_eq!(*len, external_compare(&buf, *dist, query));
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("copy_compare");
    for (dist, len, query) in tests.iter() {
        buf.copy_la(query);
        group.throughput(Throughput::Bytes(*len as u64));
        group.bench_with_input(BenchmarkId::from_parameter(len), dist, |b, dist: &usize| {
            b.iter(|| {
                assert_eq!(*len, internal_compare(&buf, *dist));
            });
        });
    }
    group.finish();
}

criterion_group!(benches, buf_benchmark_single_query);
criterion_main!(benches);
