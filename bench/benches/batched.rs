use bench::{rand_buffer, batched::*};
use criterion::{criterion_group, criterion_main, Throughput, Criterion, BenchmarkId};

const KB: usize = 1024;

pub fn buf_benchmark_batched_buffer(c: &mut Criterion) {
    let sizes = [4*KB, 8*KB, 16*KB, 32*KB];

    let mut group = c.benchmark_group("batched");
    for size in sizes.iter() {    
        let buf = &rand_buffer(*size)[..];
        let to = &rand_buffer(*size)[..];

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("batched_u64", size), size, |b, size: &usize| {
            b.iter(|| {
                let mut acc: usize = 0;
                for i in (0..*size).step_by(8) {
                    if read_u64_inline(buf, i) == read_u64_inline(to, i) {
                        for j in i..i+8 {
                            if buf[j] == to[j] {
                                acc += 1;
                            }
                        }
                    }
                }
                acc
            });
        });

        group.bench_with_input(BenchmarkId::new("batched_u64_unsafe", size), size, |b, size: &usize| {
            b.iter(|| {
                let mut acc: usize = 0;
                for i in (0..*size).step_by(8) {
                    if read_u64_unsafe_inline(buf, i) == read_u64_unsafe_inline(to, i) {
                        for j in i..i+8 {
                            if buf[j] == to[j] {
                                acc += 1;
                            }
                        }
                    }
                }
                acc
            });
        });

        group.bench_with_input(BenchmarkId::new("per_byte", size), size, |b, size: &usize| {
            b.iter(|| {
                let mut acc: usize = 0;
                for i in 0..*size {
                    if buf[i] == to[i] {
                        acc += 1;
                    }
                }
                acc
            });
        });
    }
    group.finish();
}

criterion_group!(batched, buf_benchmark_batched_buffer);
criterion_main!(batched);
