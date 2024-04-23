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
    primitives::{
        poly::{
            self, modular_poly::inv::inverse, test::gen::rand_poly, Poly, FULL_RES_POLY_DEGREE,
        },
        yashe::{Yashe, YasheParams},
    },
    IRIS_BIT_LENGTH,
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
    config = Criterion::default().sample_size(10);
    // List cyclotomic multiplication implementations here.
    targets = bench_naive_cyclotomic_mul, bench_rec_karatsuba_mul, bench_flat_karatsuba_mul
}

criterion_group! {
    name = bench_poly_split_karatsuba;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(50);
    // List polynomial split implementations here.
    targets = bench_poly_split_half, bench_poly_split_2
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
    config = Criterion::default().sample_size(20);
    // List polynomial inverse implementations here.
    targets = bench_inv
}

// Iris-length polynomial benchmarks.
// These benchmarks provide an upper bound for the performance of iris operations.
// They also help us decide if we need smaller or larger polynomial sizes.
criterion_group! {
    name = bench_cyclotomic_multiplication_iris;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(30);
    // List iris-length polynomial multiplication implementations here.
    // Currently we only benchmark the most efficient implementation.
    targets = bench_rec_karatsuba_mul_iris
}

criterion_group! {
    name = bench_inverse_iris;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10);
    // List iris-length polynomial inverse implementations here.
    targets = bench_inv_iris
}

// List groups here.
criterion_main!(
    bench_full_match,
    bench_cyclotomic_multiplication,
    bench_poly_split_karatsuba,
    bench_polynomial_modulus,
    bench_inverse,
    bench_cyclotomic_multiplication_iris,
    bench_inverse_iris
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
    let p1: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE);
    let p2: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE);

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
    let p1: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE);
    let p2: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new(
            "Recursive Karatsuba multiplication: polynomial",
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

/// Run [`poly::rec_karatsuba_mul()`] as a Criterion benchmark with random data on the full number of iris bits.
pub fn bench_rec_karatsuba_mul_iris(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1: Poly<IRIS_BIT_LENGTH> = rand_poly(IRIS_BIT_LENGTH);
    let p2: Poly<IRIS_BIT_LENGTH> = rand_poly(IRIS_BIT_LENGTH);

    settings.bench_with_input(
        BenchmarkId::new(
            "Recursive Karatsuba multiplication: polynomial",
            "2 random polys of degree IRIS_BIT_LENGTH",
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
    let p1: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE);
    let p2: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE);

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
    let p: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Karatsuba: poly split half", "random poly of degree N"),
        &(p),
        |benchmark, p| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                poly::poly_split_half(p, FULL_RES_POLY_DEGREE)
            })
        },
    );
}

/// Run [`poly::poly_split(_, 2)`] as a Criterion benchmark with random data.
pub fn bench_poly_split_2(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Karatsuba: poly split 2", "random poly of degree N"),
        &(p),
        |benchmark, p| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                poly::poly_split(p, 2)
            })
        },
    );
}

/// Run [`poly::mod_poly_manual_mut()`] as a Criterion benchmark with random data.
pub fn bench_mod_poly_manual(settings: &mut Criterion) {
    // Setup: generate a random cyclotomic polynomial the size of a typical multiplication.
    let dividend: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE * 2);

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

/// Run [`poly::mod_poly_ark_ref_slow()`] as a Criterion benchmark with random data.
pub fn bench_mod_poly_ark(settings: &mut Criterion) {
    // Setup: generate a random cyclotomic polynomial the size of a typical multiplication.
    let dividend: Poly<FULL_RES_POLY_DEGREE> = rand_poly(FULL_RES_POLY_DEGREE * 2);

    settings.bench_with_input(
        BenchmarkId::new("ark-poly polynomial modulus", "A random poly of degree 2N"),
        &dividend,
        |benchmark, dividend| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                poly::mod_poly_ark_ref_slow(dividend)
            })
        },
    );
}

/// Run [`poly::inverse()`] as a Criterion benchmark with random data.
pub fn bench_inv(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials

    let rng = rand::thread_rng();

    let params = YasheParams {
        t: 1024,
        delta: 3.2,
    };
    let ctx: Yashe<FULL_RES_POLY_DEGREE> = Yashe::new(params);

    let p = ctx.sample_gaussian(rng);

    settings.bench_with_input(
        BenchmarkId::new(
            "Cyclotomic inverse: polynomial",
            "1 relatively small random poly of degree N",
        ),
        &(p),
        |benchmark, p| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                inverse(p)
            })
        },
    );
}

/// Run [`poly::inverse()`] as a Criterion benchmark with random data on the full number of iris bits.
pub fn bench_inv_iris(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials

    let rng = rand::thread_rng();

    let params = YasheParams {
        t: 1024,
        delta: 3.2,
    };
    let ctx: Yashe<IRIS_BIT_LENGTH> = Yashe::new(params);

    let p = ctx.sample_gaussian(rng);

    settings.bench_with_input(
        BenchmarkId::new(
            "Cyclotomic inverse: polynomial",
            "1 relatively small random poly of degree IRIS_BIT_LENGTH",
        ),
        &(p),
        |benchmark, p| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                inverse(p)
            })
        },
    );
}
