use bench::{rand_buffer, batched::*};
use criterion::{criterion_group, criterion_main, Throughput, Criterion, BenchmarkId};

const KB: usize = 1024;

pub fn buf_benchmark_batched_buffer(c: &mut Criterion) {
    let sizes = [4*KB, 8*KB, 16*KB, 32*KB];

    let mut group = c.benchmark_group("batched");
    for size in sizes.iter() {    
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("batched_u64", size), size, |b, size: &usize| {
            let size = *size;
            let buf = &rand_buffer(size)[..];

            b.iter(|| {
                let mut i = 0;
                let mut acc: u64 = 0;
                while i < size {
                    acc ^= read_u64_inline(buf, i);
                    i += 8;
                }
                let mut out: u8 = 0;
                for b in u64::to_le_bytes(acc) {
                    out ^= b;
                }
                out
            });
        });

        group.bench_with_input(BenchmarkId::new("per_byte", size), size, |b, size: &usize| {
            let size = *size;
            let buf = &rand_buffer(size)[..];

            b.iter(|| {
                let mut acc: u8 = 0;
                for b in buf {
                    acc ^= *b;
                }
                acc
            });
        });
    }
    group.finish();
}

criterion_group!(batched, buf_benchmark_batched_buffer);
criterion_main!(batched);
