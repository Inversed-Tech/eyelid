//! Benchmarks for matching operations.

#![cfg(feature = "benchmark")]
// Allow missing docs in macro-produced code.
// TODO: move the macros to a separate module and allow missing docs only in that module.
#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use eyelid_match_ops::{
    plaintext::{
        self,
        test::gen::{random_iris_code, random_iris_mask},
    },
    primitives::poly::{self, test::gen::rand_poly, test::inv::sample, MAX_POLY_DEGREE},
};

// Configure Criterion:
// Define one group for each equivalent operation, so we can compare their times.
criterion_group! {
    name = bench_full_match;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(50);
    // List full match implementations here.
    targets = bench_plaintext_full_match
}

criterion_group! {
    name = bench_cyclotomic_multiplication;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(50);
    // List cyclotomic multiplication implementations here.
    targets = bench_naive_cyclotomic_mul, bench_rec_karatsuba_mul, bench_flat_karatsuba_mul
}

criterion_group! {
    name = bench_poly_split_karatsuba;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(50);
    // List cyclotomic multiplication implementations here.
    targets = bench_poly_split_half
}

criterion_group! {
    name = bench_polynomial_modulus;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default();
    // List polynomial modulus implementations here.
    targets = bench_mod_poly_manual, bench_mod_poly_ark
}

criterion_group! {
    name = bench_inverse;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default();
    // List polynomial modulus implementations here.
    targets = bench_inv
}

// List groups here.
criterion_main!(
    bench_full_match,
    bench_cyclotomic_multiplication,
    bench_poly_split_karatsuba,
    bench_polynomial_modulus,
    bench_inverse
);

/// Run [`plaintext::is_iris_match()`] as a Criterion benchmark with random data.
fn bench_plaintext_full_match(settings: &mut Criterion) {
    // Setup: generate different random iris codes and masks
    let eye_new = random_iris_code();
    let mask_new = random_iris_mask();
    let eye_store = random_iris_code();
    let mask_store = random_iris_mask();

    settings.bench_with_input(
        BenchmarkId::new("Full iris match: plaintext", "Random bits"),
        &(eye_new, mask_new, eye_store, mask_store),
        |benchmark, (eye_new, mask_new, eye_store, mask_store)| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                plaintext::is_iris_match(eye_new, mask_new, eye_store, mask_store)
            })
        },
    );
}

/// Run [`poly::naive_cyclotomic_mul()`] as a Criterion benchmark with random data.
pub fn bench_naive_cyclotomic_mul(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1 = rand_poly(MAX_POLY_DEGREE);
    let p2 = rand_poly(MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new(
            "Cyclotomic multiplication: polynomial",
            "2 random polys of degree N",
        ),
        &(p1, p2),
        |benchmark, (p1, p2)| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                poly::naive_cyclotomic_mul(p1, p2)
            })
        },
    );
}

/// Run [`poly::rec_karatsuba_mul()`] as a Criterion benchmark with random data.
pub fn bench_rec_karatsuba_mul(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1 = rand_poly(MAX_POLY_DEGREE);
    let p2 = rand_poly(MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new(
            "Karatsuba multiplication: polynomial",
            "2 random polys of degree N",
        ),
        &(p1, p2),
        |benchmark, (p1, p2)| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                poly::rec_karatsuba_mul(p1, p2)
            })
        },
    );
}

/// Run [`poly::flat_karatsuba_mul()`] as a Criterion benchmark with random data.
pub fn bench_flat_karatsuba_mul(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1 = rand_poly(MAX_POLY_DEGREE);
    let p2 = rand_poly(MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new(
            "Flat Karatsuba multiplication: polynomial",
            "2 random polys of degree N",
        ),
        &(p1, p2),
        |benchmark, (p1, p2)| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                poly::flat_karatsuba_mul(p1, p2)
            })
        },
    );
}

/// Run [`poly::poly_split_half()`] as a Criterion benchmark with random data.
pub fn bench_poly_split_half(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p = rand_poly(MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Karatsuba: poly split", "random poly of degree N"),
        &(p),
        |benchmark, p| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                poly::poly_split_half(p, MAX_POLY_DEGREE)
            })
        },
    );
}

/// Run [`poly::mod_poly_manual()`] as a Criterion benchmark with random data.
pub fn bench_mod_poly_manual(settings: &mut Criterion) {
    // Setup: generate a random cyclotomic polynomial the size of a typical multiplication.
    let dividend = rand_poly(MAX_POLY_DEGREE * 2);

    settings.bench_with_input(
        BenchmarkId::new("Manual polynomial modulus", "A random poly of degree 2N"),
        &dividend,
        |benchmark, dividend| {
            benchmark.iter_with_large_drop(|| {
                // TODO: work out how to avoid timing this clone
                // (The production code already avoids cloning where possible.)
                let mut dividend = dividend.clone();

                poly::mod_poly_manual_mut(&mut dividend);

                // To avoid timing dropping dividend, we return it instead
                dividend
            })
        },
    );
}

/// Run [`poly::mod_poly_ark()`] as a Criterion benchmark with random data.
pub fn bench_mod_poly_ark(settings: &mut Criterion) {
    // Setup: generate a random cyclotomic polynomial the size of a typical multiplication.
    let dividend = rand_poly(MAX_POLY_DEGREE * 2);

    settings.bench_with_input(
        BenchmarkId::new("ark-poly polynomial modulus", "A random poly of degree 2N"),
        &dividend,
        |benchmark, dividend| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                poly::mod_poly_ark_ref(dividend)
            })
        },
    );
}

/// Run [`poly::karatsuba_mul()`] as a Criterion benchmark with random data.
pub fn bench_inv(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p = sample();

    settings.bench_with_input(
        BenchmarkId::new(
            "Cyclotomic inverse: polynomial",
            "1 relatively small random poly of degree N",
        ),
        &(p),
        |benchmark, p| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                poly::inverse(p)
            })
        },
    );
}
