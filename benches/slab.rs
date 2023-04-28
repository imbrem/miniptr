use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use either::Either;
use miniptr::{
    pool::slab::KeySlabPool,
    pool::{slab::SlabPool, *},
    slot::DefaultSlot,
};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;

fn pool_benchmark(c: &mut Criterion) {
    const REMOVAL_FRACTION: f64 = 0.3;

    let mut group = c.benchmark_group("insert-remove-trace");
    let mut rng = Xoshiro256StarStar::from_seed([0xAB; 32]);

    for size in [10, 100, 1000, 10000, 1000000] {
        let mut trace: Vec<isize> = Vec::new();
        let mut pool: KeySlabPool<Either<usize, usize>> = KeySlabPool::new();
        let mut inserted = Vec::new();
        for _ in 0..size {
            if !inserted.is_empty() && rng.gen_bool(REMOVAL_FRACTION) {
                let remove = inserted.swap_remove(rng.gen_range(0..inserted.len()));
                let _ = pool.remove(remove);
                trace.push(-(remove as isize) - 1)
            } else {
                let key = pool.insert(0);
                inserted.push(key);
                trace.push(key as isize)
            }
        }

        group.throughput(criterion::Throughput::Elements(size));
        group.bench_with_input(BenchmarkId::new("either", size), &size, |b, _| {
            b.iter(|| {
                let mut pool: KeySlabPool<Either<usize, usize>> = KeySlabPool::new();
                for &event in trace.iter() {
                    if event >= 0 {
                        black_box(pool.insert(event as usize));
                    } else {
                        black_box(pool.remove(-(event + 1) as usize));
                    }
                    black_box(&mut pool);
                }
            })
        });

        group.bench_with_input(BenchmarkId::new("default-key", size), &size, |b, _| {
            b.iter(|| {
                let mut pool: KeySlabPool<DefaultSlot<usize>> = KeySlabPool::new();
                for &event in trace.iter() {
                    if event >= 0 {
                        black_box(pool.insert(event as usize));
                    } else {
                        black_box(pool.remove(-(event + 1) as usize));
                    }
                    black_box(&mut pool);
                }
            })
        });

        group.bench_with_input(BenchmarkId::new("default", size), &size, |b, _| {
            b.iter(|| {
                let mut pool: SlabPool<DefaultSlot<usize>> = SlabPool::<DefaultSlot<usize>>::new();
                for &event in trace.iter() {
                    if event >= 0 {
                        black_box(pool.insert(event as usize));
                    } else {
                        black_box(pool.remove(-(event + 1) as usize));
                    }
                    black_box(&mut pool);
                }
            })
        });

        group.bench_with_input(
            BenchmarkId::new("vec-push-overwrite", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut pool = Vec::new();
                    for &event in trace.iter() {
                        if event >= 0 {
                            if (event as usize) < pool.len() {
                                pool[event as usize] = event;
                            } else {
                                pool.push(event)
                            }
                        }
                        black_box(&mut pool);
                    }
                })
            },
        );

        group.bench_with_input(BenchmarkId::new("vec-push-only", size), &size, |b, _| {
            b.iter(|| {
                let mut pool = Vec::new();
                for &event in trace.iter() {
                    if event >= 0 {
                        pool.push(event)
                    }
                    black_box(&mut pool);
                }
            })
        });
    }
    group.finish();
}

criterion_group!(benches, pool_benchmark);
criterion_main!(benches);
