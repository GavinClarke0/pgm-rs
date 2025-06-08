mod common;

use std::fmt;

use common::fast_criterion;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use pgm_rs::pgm::PGMIndex;
use rand::{RngCore, SeedableRng};

fn run_search_benchmarks(c: &mut Criterion) {
    let error_rates = [0.0, 0.00001, 0.0001, 0.001];

    for &error_rate in &error_rates {
        run_search_benchmark_with_error(c, error_rate)
    }
}

fn run_search_benchmark_with_error(c: &mut Criterion, error_rate: f64) {
    let sizes = [10_000, 1_000_000, 10_000_000, 100_000_000];

    let mut group = c.benchmark_group(format!("pgm_search_error_rate_{:6}", error_rate));
    for &size in &sizes {
        let epsilon = common::epsilon(error_rate, size);
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let uniform = common::generate_uniform(size, 7);
        let index = PGMIndex::build(&uniform, epsilon).unwrap();
        group.bench_with_input(BenchmarkId::new("uniform", size), &index, |b, idx| {
            b.iter(|| {
                let _ = idx.search(uniform[rng.next_u64() as usize % size]);
            });
        });

        let exp = common::generate_exponential(size, 0.01);
        let index = PGMIndex::build(&exp, epsilon).unwrap();
        group.bench_with_input(BenchmarkId::new("exponential", size), &index, |b, idx| {
            b.iter(|| {
                let _ = idx.search(exp[rng.next_u64() as usize % size]);
            });
        });

        let clustered = common::generate_clustered(size, 32, 1000);
        let index = PGMIndex::build(&clustered, epsilon).unwrap();
        group.bench_with_input(BenchmarkId::new("clustered", size), &index, |b, idx| {
            b.iter(|| {
                let _ = idx.search(clustered[rng.next_u64() as usize % size]);
            });
        });
    }

    group.finish();
}

fn binary_search_benchmark(c: &mut Criterion) {
    let sizes = [10_000, 1_000_000, 10_000_000, 100_000_000];

    let mut group = c.benchmark_group("binary_search");

    for &size in &sizes {
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let uniform = common::generate_uniform(size, 7);
        group.bench_with_input(
            BenchmarkId::new("uniform_binary_search", size),
            &uniform,
            |b, _| {
                b.iter(|| {
                    let _ = uniform.binary_search(&uniform[rng.next_u64() as usize % size]);
                });
            },
        );

        let exp = common::generate_exponential(size, 0.01);
        group.bench_with_input(
            BenchmarkId::new("exponential_binary_search", size),
            &exp,
            |b, _| {
                b.iter(|| {
                    let _ = exp.binary_search(&uniform[rng.next_u64() as usize % size]);
                });
            },
        );

        let clustered = common::generate_clustered(size, 32, 1000);
        group.bench_with_input(
            BenchmarkId::new("clustered_binary_search", size),
            &clustered,
            |b, _| {
                b.iter(|| {
                    let _ = clustered.binary_search(&uniform[rng.next_u64() as usize % size]);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(name = benches; config = fast_criterion(); targets = run_search_benchmarks, binary_search_benchmark);
criterion_main!(benches);
