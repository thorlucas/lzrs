use criterion::{criterion_group, criterion_main, Throughput, Criterion, BenchmarkId, black_box};
use lzrs2::buffer::*;
use rand::{Fill, thread_rng};

// prime
const LEN: usize = 251;

pub fn fastcmp_u8(c: &mut Criterion) {
    let mut rng = thread_rng();

    let haystack: &mut [u8] = &mut vec![0; 512];
    haystack.try_fill(&mut rng).unwrap();

    let needle: &mut [u8] = &mut vec![0; 256];
    needle.try_fill(&mut rng).unwrap();
    needle[..LEN].copy_from_slice(&haystack[..LEN]);
    needle[LEN] = !haystack[LEN];

    let mut group = c.benchmark_group("fastcmp_u8");
    group.throughput(Throughput::Bytes(LEN as u64));

    group.bench_function("fastcmp_u8", |b| {
        b.iter(|| {
            let needle: &[u8] = black_box(&needle);
            assert_eq!(LEN, haystack.match_length(black_box(needle)))
        });
    });

    group.bench_function("naive", |b| {
        b.iter(|| {
            let mut len = 0;
            let needle: &[u8] = black_box(&needle);
            let max_len = std::cmp::min(needle.len(), haystack.len());
            while len < max_len && haystack[len] == needle[len] {
                len += 1
            }
            assert_eq!(LEN, len);
        });
    });

    group.finish();
}

criterion_group!(fastcmp, fastcmp_u8);
criterion_main!(fastcmp);
