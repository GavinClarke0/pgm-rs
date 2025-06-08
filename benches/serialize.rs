mod common;

use common::fast_criterion;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use pgm_rs::pgm::PGMIndex;

fn run_serialize_benchmark_10_e_error_rate(c: &mut Criterion) {
    let sizes = [10_000, 1_000_000, 10_000_000, 100_000_000];
    let error_rate = 0.0001;

    let mut group = c.benchmark_group("pgm_serialize");

    for &size in &sizes {
        let epsilon = common::epsilon(error_rate, size);

        let uniform = common::generate_uniform(size, 7);
        let index = PGMIndex::build(&uniform, epsilon).unwrap();
        group.bench_function(BenchmarkId::new("uniform", size), |b| {
            b.iter(|| {
                let _ = index.to_bytes();
            })
        });

        let exp = common::generate_exponential(size, 0.01);
        let index = PGMIndex::build(&exp, epsilon).unwrap();
        group.bench_function(BenchmarkId::new("exponential", size), |b| {
            b.iter(|| {
                let _ = index.to_bytes();
            })
        });

        let clustered = common::generate_clustered(size, 32, 1000);
        let index = PGMIndex::build(&clustered, epsilon).unwrap();
        group.bench_function(BenchmarkId::new("clustered", size), |b| {
            b.iter(|| {
                let _ = index.to_bytes();
            })
        });
    }
    group.finish();
}

fn run_serialize_benchmark_0_error_rate(c: &mut Criterion) {
    let sizes = [10_000, 1_000_000, 10_000_000, 100_000_000];
    let error_rate = 0.0;

    let mut group = c.benchmark_group("pgm_serialize");

    for &size in &sizes {
        let epsilon = common::epsilon(error_rate, size);

        let uniform = common::generate_uniform(size, 7);
        let index = PGMIndex::build(&uniform, epsilon).unwrap();
        group.bench_function(BenchmarkId::new("uniform_0_error", size), |b| {
            b.iter(|| {
                let _ = index.to_bytes();
            })
        });

        let exp = common::generate_exponential(size, 0.01);
        let index = PGMIndex::build(&exp, epsilon).unwrap();
        group.bench_function(BenchmarkId::new("exponential_0_error", size), |b| {
            b.iter(|| {
                let _ = index.to_bytes();
            })
        });

        let clustered = common::generate_clustered(size, 32, 1000);
        let index = PGMIndex::build(&clustered, epsilon).unwrap();
        group.bench_function(BenchmarkId::new("clustered_0_error", size), |b| {
            b.iter(|| {
                let _ = index.to_bytes();
            })
        });
    }
    group.finish();
}

criterion_group!(name = benches; config = fast_criterion(); targets =  run_serialize_benchmark_10_e_error_rate, run_serialize_benchmark_0_error_rate);
criterion_main!(benches);
