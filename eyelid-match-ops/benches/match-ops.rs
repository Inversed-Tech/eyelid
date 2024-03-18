//! Benchmarks for matching operations.
#![cfg(feature = "benchmark")]

use eyelid_match_ops::{
    plaintext::{
        self,
        test::gen::{random_iris_code, random_iris_mask},
    },
    primitives::poly_rug,
};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

// Configure Criterion:
// Define one group for each equivalent operation, so we can compare their times.
criterion_group! {
    name = bench_full_match;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(40);
    // List full match implementations here.
    targets = bench_plaintext_full_match
}

criterion_group! {
    name = bench_cyclotomic_multiplication;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default();
    // List cyclotomic multiplication implementations here.
    targets = bench_cyc_mul_rug
}

// List groups here.
criterion_main!(bench_full_match, bench_cyclotomic_multiplication);

/// Run plaintext::is_iris_match() as a Criterion benchmark with random data.
fn bench_plaintext_full_match(settings: &mut Criterion) {
    // Setup: generate different random iris codes and masks
    let eye_new = random_iris_code();
    let mask_new = random_iris_mask();
    let eye_store = random_iris_code();
    let mask_store = random_iris_mask();

    settings.bench_with_input(
        BenchmarkId::new("Full iris match: plaintext", "Random iris codes and masks"),
        &(eye_new, mask_new, eye_store, mask_store),
        |benchmark, (eye_new, mask_new, eye_store, mask_store)| {
            benchmark.iter_with_large_drop(|| {
                plaintext::is_iris_match(eye_new, mask_new, eye_store, mask_store)
            })
        },
    );
}

/// Run [`poly_rug::cyclotomic_mul()`] as a Criterion benchmark with random data.
pub fn bench_cyc_mul_rug(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1 = poly_rug::rand_poly(poly_rug::MAX_POLY_DEGREE);
    let p2 = poly_rug::rand_poly(poly_rug::MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Cyclotomic multiplication using rug", "Random input"),
        &(p1, p2),
        |benchmark, (p1, p2)| {
            benchmark.iter_with_large_drop(|| {
                poly_rug::cyclotomic_mul(p1, p2);
            })
        },
    );
}
