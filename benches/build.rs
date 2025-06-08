mod common;

use common::fast_criterion;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use pgm_rs::pgm::PGMIndex;

fn bench_build(c: &mut Criterion) {
    let sizes = [10_000, 1_000_000, 10_000_000, 100_000_000];
    let epsilon = 64;

    let mut group = c.benchmark_group("pgm_build");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));

    for &size in &sizes {
        let uniform = common::generate_uniform(size, 7);
        group.bench_with_input(BenchmarkId::new("uniform", size), &uniform, |b, data| {
            b.iter(|| PGMIndex::build(data, epsilon).unwrap())
        });

        let exp = common::generate_exponential(size, 0.01);
        group.bench_with_input(BenchmarkId::new("exponential", size), &exp, |b, data| {
            b.iter(|| PGMIndex::build(data, epsilon).unwrap())
        });

        let clustered = common::generate_clustered(size, 32, 1000);
        group.bench_with_input(
            BenchmarkId::new("clustered", size),
            &clustered,
            |b, data| b.iter(|| PGMIndex::build(data, epsilon).unwrap()),
        );
    }

    group.finish();
}

criterion_group!(name = benches; config = fast_criterion(); targets =  bench_build);
criterion_main!(benches);
