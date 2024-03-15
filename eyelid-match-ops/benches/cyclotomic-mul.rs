//! Benchmarks for cyclotomic multiplication.
#![cfg(feature = "benchmark")]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use eyelid_match_ops::primitives::poly::rand_pol;
use eyelid_match_ops::primitives::poly::cyclotomic_mul;

// Configure Criterion:
// Define one group for each equivalent operation, so we can compare their times.
criterion_group! {
    name = bench_cyclotomic_multiplication;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(40);
    // List full match implementationsions here.
    targets = bench_cyclotomic_mul
}

// List groups here.
criterion_main!(bench_cyclotomic_multiplication);

/// Run cyclotomic_mul as a Criterion benchmark with random data.
pub fn bench_cyclotomic_mul(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1 = rand_pol();
    let p2 = rand_pol();

    settings.bench_with_input(
        BenchmarkId::new("Cyclotomic multiplication", "Random input"),
        &(p1, p2),
        |benchmark, (p1, p2)| {
            benchmark.iter_with_large_drop(
                || {
                    cyclotomic_mul(p1.clone(), p2.clone());
                })
        },
    );
}
