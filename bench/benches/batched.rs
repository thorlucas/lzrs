use bench::{rand_buffer, batched::*};
use criterion::{criterion_group, criterion_main, Throughput, Criterion, BenchmarkId, black_box};

const KB: usize = 1024;
const SIZES: [usize; 1] = [4*KB];

pub fn buf_benchmark_batched_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("batched_read");
    for size in SIZES.iter() {    
        let buf = &rand_buffer(*size)[..];
        let from = &rand_buffer(*size)[..];

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("batched_u64", size), size, |b, size: &usize| {
            b.iter(|| {
                let mut acc: usize = 0;
                let from = black_box(from);
                for i in (0..*size).step_by(8) {
                    if read_u64_inline(buf, i) == read_u64_inline(from, i) {
                        acc += 1
                    }
                }
                acc
            });
        });

        /*
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
        */

        group.bench_with_input(BenchmarkId::new("per_byte", size), size, |b, size: &usize| {
            b.iter(|| {
                let mut acc: usize = 0;
                for i in 0..*size {
                    if buf[i] == from[i] {
                        acc += 1;
                    }
                }
                acc
            });
        });
    }
    group.finish();
}

pub fn buf_benchmark_batched_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("batched_write");
    for size in SIZES.iter() {    
        let buf = &mut rand_buffer(*size)[..];
        let from = &rand_buffer(*size)[..];

        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("batched_u64", size), size, |b, size: &usize| {
            b.iter(|| {
                let from = black_box(from);
                for i in (0..*size).step_by(8) {
                    let src = read_u64_inline(from, i);
                    write_u64_inline(src, buf, i);
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("per_byte", size), size, |b, size: &usize| {
            b.iter(|| {
                let from = black_box(from);
                for i in 0..*size {
                    buf[i] = from[i];
                }
            });
        });
    }
    group.finish();
}

criterion_group!(batched, buf_benchmark_batched_buffer, buf_benchmark_batched_write);
criterion_main!(batched);
