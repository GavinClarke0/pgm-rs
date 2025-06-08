use criterion::Criterion;
use rand::SeedableRng;

pub fn fast_criterion() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_millis(200))
        .warm_up_time(std::time::Duration::from_millis(100))
}

pub fn generate_uniform(n: usize, step: u64) -> Vec<u64> {
    (0..n).map(|i| i as u64 * step).collect()
}

pub fn generate_exponential(n: usize, lambda: f64) -> Vec<u64> {
    use rand_distr::{Distribution, Exp};
    let exp = Exp::new(lambda).unwrap();
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut out = Vec::with_capacity(n);
    let mut sum = 0.0;
    for _ in 0..n {
        sum += exp.sample(&mut rng);
        out.push(sum as u64);
    }
    out
}

pub fn generate_clustered(n: usize, cluster_size: usize, gap: u64) -> Vec<u64> {
    let mut out = Vec::with_capacity(n);
    let mut base = 0;
    while out.len() < n {
        for j in 0..cluster_size.min(n - out.len()) {
            out.push(base + j as u64);
        }
        base += gap;
    }
    out
}

pub fn epsilon(target_error_rate: f64, keys_len: usize) -> usize {
    (keys_len as f64 * target_error_rate / 2.0).ceil() as usize
}
