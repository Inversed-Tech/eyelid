//! Benchmarks for matching operations.
//!
//! To add a benchmark to the PR comparison, change the benchmark selection regex in
//! `ci-bench-changes.yml`(https://github.com/Inversed-Tech/eyelid/blob/3668934d68780513ea61ede8f4ccfb2d6a7eaedb/.github/workflows/ci-bench-changes.yml#L55).
//!
//! Benchmarks that take longer than a minute are disabled by default.
//! Use this command to run the benchmarks that are very slow:
//! ```sh
//! RUSTFLAGS="--cfg slow_benchmarks" cargo bench --features benchmark
//! ```

#![cfg(feature = "benchmark")]
// Allow missing docs in macro-produced code.
// TODO: move the macros to a separate module and allow missing docs only in that module.
#![allow(missing_docs)]

use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use eyelid_match_ops::{
    encoded::{PolyCode, PolyQuery},
    encrypted::{convert_negative_coefficients, EncryptedPolyCode, EncryptedPolyQuery},
    plaintext::{
        self,
        test::gen::{random_iris_code, random_iris_mask},
    },
    primitives::{
        poly::{self, test::gen::rand_poly, Poly, PolyConf},
        yashe::{self, Ciphertext, Message, Yashe},
    },
    EncodeConf, IrisConf, MiddleRes, TestRes,
};

// Configure Criterion:
// Define one group for each equivalent operation, so we can compare their times.
criterion_group! {
    name = bench_full_match;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(50);
    // List full match implementations here.
    targets = bench_plaintext_full_match, bench_ciphertext_full_match
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

criterion_group! {
    name = bench_key_generation;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10);
    // List key generation implementations here.
    targets = bench_keygen
}

criterion_group! {
    name = bench_encryption;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10);
    // List encryption implementations here.
    targets = bench_enc
}

criterion_group! {
    name = bench_decryption;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10);
    // List decryption implementations here.
    targets = bench_dec
}

criterion_group! {
    name = bench_yashe_mul;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10);
    // List Yashe multiplication implementations here.
    targets = bench_yashe_msg_mul, bench_yashe_cipher_mul
}

// Middle resolution polynomial benchmarks.
criterion_group! {
    name = bench_cyclotomic_multiplication_mid;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(50));
    // List iris-length polynomial multiplication implementations here.
    targets = bench_naive_cyclotomic_mul_mid, bench_rec_karatsuba_mul_mid, bench_flat_karatsuba_mul_mid
}

criterion_group! {
    name = bench_inverse_mid;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(120));
    // List iris-length polynomial inverse implementations here.
    targets = bench_inv_mid
}

criterion_group! {
    name = bench_key_generation_mid;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(230));
    // List key generation implementations here.
    targets = bench_keygen_mid
}

// List groups here.
criterion_main!(
    bench_full_match,
    bench_cyclotomic_multiplication,
    bench_poly_split_karatsuba,
    bench_polynomial_modulus,
    bench_inverse,
    bench_key_generation,
    bench_encryption,
    bench_decryption,
    bench_yashe_mul,
    bench_cyclotomic_multiplication_mid,
    bench_inverse_mid,
    bench_key_generation_mid
);

/// The name used for slow benchmark groups.
pub const SLOW_BENCH_NAME: &str = "Slow";

/// The name used for randomly generated bits.
pub const RANDOM_BITS_NAME: &str = "random";

/// The name used for small randomly distributions.
pub const SMALL_RANDOM_NAME: &str = "small rand";

/// Run [`plaintext::is_iris_match()`] as a Criterion benchmark with random data.
fn bench_plaintext_full_match(settings: &mut Criterion) {
    use eyelid_match_ops::FullBits;

    // Setup: generate different random iris codes and masks
    let eye_new = random_iris_code();
    let mask_new = random_iris_mask();
    let eye_store = random_iris_code();
    let mask_store = random_iris_mask();

    settings.bench_with_input(
        BenchmarkId::new("Plaintext full match", RANDOM_BITS_NAME),
        &(eye_new, mask_new, eye_store, mask_store),
        |benchmark, (eye_new, mask_new, eye_store, mask_store)| {
            benchmark.iter_with_large_drop(|| {
                // To avoid timing dropping the return value, this line must not end in ';'
                plaintext::is_iris_match::<FullBits, { FullBits::STORE_ELEM_LEN }>(
                    eye_new, mask_new, eye_store, mask_store,
                )
            })
        },
    );
}

/// Run [`encrypterd_poly_query::is_match()`] as a Criterion benchmark with random data.
fn bench_ciphertext_full_match(settings: &mut Criterion) {
    use eyelid_match_ops::FullBits;

    let mut rng = rand::thread_rng();
    let ctx: Yashe<<FullBits as EncodeConf>::PlainConf> = Yashe::new();
    let (private_key, public_key) = ctx.keygen(&mut rng);

    let eye_new: bitvec::array::BitArray<[usize; FullBits::STORE_ELEM_LEN]> = random_iris_code();
    let mask_new: bitvec::array::BitArray<[usize; FullBits::STORE_ELEM_LEN]> = random_iris_mask();
    let eye_store: bitvec::array::BitArray<[usize; FullBits::STORE_ELEM_LEN]> = random_iris_code();
    let mask_store: bitvec::array::BitArray<[usize; FullBits::STORE_ELEM_LEN]> = random_iris_mask();

    let mut poly_query: PolyQuery<FullBits> = PolyQuery::from_plaintext(&eye_new, &mask_new);
    let mut poly_code = PolyCode::from_plaintext(&eye_store, &mask_store);

    convert_negative_coefficients::<FullBits>(&mut poly_query.polys);
    convert_negative_coefficients::<FullBits>(&mut poly_code.polys);

    let encrypted_poly_query =
        EncryptedPolyQuery::encrypt_query(ctx, poly_query.clone(), &public_key, &mut rng);
    let encrypted_poly_code =
        EncryptedPolyCode::encrypt_code(ctx, poly_code.clone(), &public_key, &mut rng);

    settings.bench_with_input(
        BenchmarkId::new("Ciphertext full match", RANDOM_BITS_NAME),
        &(encrypted_poly_query, private_key, encrypted_poly_code),
        |benchmark, (encrypted_poly_query, private_key, encrypted_poly_code)| {
            benchmark.iter_with_large_drop(|| {
                // There aren't any large drops here, but we use the same benchmark method for consistency
                encrypted_poly_query
                    .is_match(ctx, private_key, encrypted_poly_code)
                    .expect("encrypted matching must work")
            })
        },
    );
}

/// Run [`poly::naive_cyclotomic_mul()`] as a Criterion benchmark with random data.
pub fn bench_naive_cyclotomic_mul(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE);
    let p2: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Naive mul poly", RANDOM_BITS_NAME),
        &(p1, p2),
        |benchmark, (p1, p2)| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark
                .iter_with_large_drop(|| -> Poly<TestRes> { poly::naive_cyclotomic_mul(p1, p2) })
        },
    );
}

/// Run [`poly::naive_cyclotomic_mul()`] as a Criterion benchmark with random data on middle resolution.
pub fn bench_naive_cyclotomic_mul_mid(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1: Poly<MiddleRes> = rand_poly(MiddleRes::MAX_POLY_DEGREE);
    let p2: Poly<MiddleRes> = rand_poly(MiddleRes::MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Naive mul mid poly", RANDOM_BITS_NAME),
        &(p1, p2),
        |benchmark, (p1, p2)| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark
                .iter_with_large_drop(|| -> Poly<MiddleRes> { poly::naive_cyclotomic_mul(p1, p2) })
        },
    );
}

/// Run [`poly::rec_karatsuba_mul()`] as a Criterion benchmark with random data.
pub fn bench_rec_karatsuba_mul(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE);
    let p2: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Rec karatsuba mul poly", RANDOM_BITS_NAME),
        &(p1, p2),
        |benchmark, (p1, p2)| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark.iter_with_large_drop(|| -> Poly<TestRes> { poly::rec_karatsuba_mul(p1, p2) })
        },
    );
}

/// Run [`poly::rec_karatsuba_mul()`] as a Criterion benchmark with random data on middle resolution.
pub fn bench_rec_karatsuba_mul_mid(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1: Poly<MiddleRes> = rand_poly(MiddleRes::MAX_POLY_DEGREE);
    let p2: Poly<MiddleRes> = rand_poly(MiddleRes::MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Rec karatsuba mul mid poly", RANDOM_BITS_NAME),
        &(p1, p2),
        |benchmark, (p1, p2)| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark
                .iter_with_large_drop(|| -> Poly<MiddleRes> { poly::rec_karatsuba_mul(p1, p2) })
        },
    );
}

/// Run [`poly::flat_karatsuba_mul()`] as a Criterion benchmark with random data.
pub fn bench_flat_karatsuba_mul(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE);
    let p2: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Flat karatsuba mul poly", RANDOM_BITS_NAME),
        &(p1, p2),
        |benchmark, (p1, p2)| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark.iter_with_large_drop(|| -> Poly<TestRes> { poly::flat_karatsuba_mul(p1, p2) })
        },
    );
}

/// Run [`poly::flat_karatsuba_mul()`] as a Criterion benchmark with random data on middle resolution.
pub fn bench_flat_karatsuba_mul_mid(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p1: Poly<MiddleRes> = rand_poly(MiddleRes::MAX_POLY_DEGREE);
    let p2: Poly<MiddleRes> = rand_poly(MiddleRes::MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Flat karatsuba mul mid poly", RANDOM_BITS_NAME),
        &(p1, p2),
        |benchmark, (p1, p2)| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark
                .iter_with_large_drop(|| -> Poly<MiddleRes> { poly::flat_karatsuba_mul(p1, p2) })
        },
    );
}

/// Run [`poly::poly_split_half()`] as a Criterion benchmark with random data.
pub fn bench_poly_split_half(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Split poly half", RANDOM_BITS_NAME),
        &(p),
        |benchmark, p| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark.iter_with_large_drop(|| -> (Poly<TestRes>, Poly<TestRes>) {
                poly::poly_split_half(p, TestRes::MAX_POLY_DEGREE)
            })
        },
    );
}

/// Run [`poly::poly_split(_, 2)`] as a Criterion benchmark with random data.
pub fn bench_poly_split_2(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials
    let p: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE);

    settings.bench_with_input(
        BenchmarkId::new("Split poly 2", RANDOM_BITS_NAME),
        &(p),
        |benchmark, p| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark.iter_with_large_drop(|| -> Vec<Poly<TestRes>> { poly::poly_split(p, 2) })
        },
    );
}

/// Run [`poly::mod_poly_manual_mut()`] as a Criterion benchmark with random data.
pub fn bench_mod_poly_manual(settings: &mut Criterion) {
    // Setup: generate a random cyclotomic polynomial the size of a typical multiplication.
    let dividend: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE * 2);

    settings.bench_with_input(
        BenchmarkId::new("Manual mod reduce poly", RANDOM_BITS_NAME),
        &dividend,
        |benchmark, dividend| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark.iter_with_large_drop(|| -> Poly<TestRes> {
                // TODO: work out how to avoid timing this clone
                // (The production code already avoids cloning where possible.)
                let mut dividend = dividend.clone();

                poly::mod_poly_manual_mut(&mut dividend);

                dividend
            })
        },
    );
}

/// Run [`poly::mod_poly_ark_ref_slow()`] as a Criterion benchmark with random data.
pub fn bench_mod_poly_ark(settings: &mut Criterion) {
    // Setup: generate a random cyclotomic polynomial the size of a typical multiplication.
    let dividend: Poly<TestRes> = rand_poly(TestRes::MAX_POLY_DEGREE * 2);

    settings.bench_with_input(
        BenchmarkId::new("ark-ff mod reduce poly", RANDOM_BITS_NAME),
        &dividend,
        |benchmark, dividend| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark
                .iter_with_large_drop(|| -> Poly<TestRes> { poly::mod_poly_ark_ref_slow(dividend) })
        },
    );
}

/// Run [`poly::inverse()`] as a Criterion benchmark with gaussian random data.
///
/// TODO: consider benchmarking the inverse of a uniform random polynomial as well
pub fn bench_inv(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials

    let mut rng = rand::thread_rng();

    let ctx: Yashe<TestRes> = Yashe::new();

    let p = ctx.sample_key(&mut rng);

    settings.bench_with_input(
        BenchmarkId::new("Inverse poly", SMALL_RANDOM_NAME),
        &(p),
        |benchmark, p| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark
                .iter_with_large_drop(|| -> Result<Poly<TestRes>, &'static str> { p.inverse() })
        },
    );
}

/// Run [`poly::inverse()`] as a Criterion benchmark with gaussian random data on middle resolution.
pub fn bench_inv_mid(settings: &mut Criterion) {
    // Setup: generate random cyclotomic polynomials

    let mut rng = rand::thread_rng();

    let ctx: Yashe<MiddleRes> = Yashe::new();

    let p = ctx.sample_key(&mut rng);

    settings.bench_with_input(
        BenchmarkId::new("Inverse mid poly", SMALL_RANDOM_NAME),
        &(p),
        |benchmark, p| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark
                .iter_with_large_drop(|| -> Result<Poly<MiddleRes>, &'static str> { p.inverse() })
        },
    );
}

/// Run [`Yashe::keygen()`] as a Criterion benchmark with random data.
pub fn bench_keygen(settings: &mut Criterion) {
    // Setup parameters
    let ctx: Yashe<TestRes> = Yashe::new();

    settings.bench_with_input(
        BenchmarkId::new("YASHE keygen", SMALL_RANDOM_NAME),
        &ctx,
        |benchmark, ctx| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark.iter_with_large_drop(
                || -> (yashe::PrivateKey<TestRes>, yashe::PublicKey<TestRes>) {
                    // The thread_rng() call is efficient, because it only clones a small amount of memory,
                    // which is dedicated to the current thread.
                    ctx.keygen(&mut rand::thread_rng())
                },
            )
        },
    );
}

/// Run [`Yashe::enc()`] as a Criterion benchmark with random data.
pub fn bench_enc(settings: &mut Criterion) {
    // Setup parameters
    let mut rng = rand::thread_rng();
    let ctx: Yashe<TestRes> = Yashe::new();

    let (_private_key, public_key) = ctx.keygen(&mut rng);
    let m = ctx.sample_message(&mut rng);

    settings.bench_with_input(
        BenchmarkId::new("YASHE enc", SMALL_RANDOM_NAME),
        &ctx,
        |benchmark, ctx| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark.iter_with_large_drop(|| -> Ciphertext<TestRes> {
                ctx.encrypt(m.clone(), &public_key, &mut rng)
            })
        },
    );
}

/// Run [`Yashe::dec()`] as a Criterion benchmark with random data.
pub fn bench_dec(settings: &mut Criterion) {
    // Setup parameters
    let mut rng = rand::thread_rng();
    let ctx: Yashe<TestRes> = Yashe::new();

    let (private_key, public_key) = ctx.keygen(&mut rng);
    let m = ctx.sample_message(&mut rng);
    let c = ctx.encrypt(m, &public_key, &mut rng);

    settings.bench_with_input(
        BenchmarkId::new("YASHE dec", SMALL_RANDOM_NAME),
        &ctx,
        |benchmark, ctx| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark.iter_with_large_drop(|| -> Message<TestRes> {
                ctx.decrypt(c.clone(), &private_key)
            })
        },
    );
}

/// Run [`Yashe::plaintext_mul()`] as a Criterion benchmark with random data.
pub fn bench_yashe_msg_mul(settings: &mut Criterion) {
    // Setup parameters
    let mut rng = rand::thread_rng();
    let ctx: Yashe<TestRes> = Yashe::new();

    let m1 = ctx.sample_message(&mut rng);
    let m2 = ctx.sample_message(&mut rng);

    settings.bench_with_input(
        BenchmarkId::new("YASHE msg mul", SMALL_RANDOM_NAME),
        &ctx,
        |benchmark, ctx| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark.iter_with_large_drop(|| -> Message<TestRes> {
                ctx.plaintext_mul(m1.clone(), m2.clone())
            })
        },
    );
}

/// Run [`Yashe::ciphertext_mul()`] as a Criterion benchmark with random data.
pub fn bench_yashe_cipher_mul(settings: &mut Criterion) {
    // Setup parameters
    let mut rng = rand::thread_rng();
    let ctx: Yashe<TestRes> = Yashe::new();

    let (_private_key, public_key) = ctx.keygen(&mut rng);
    let m1 = ctx.sample_message(&mut rng);
    let m2 = ctx.sample_message(&mut rng);

    let m1 = ctx.encrypt(m1, &public_key, &mut rng);
    let m2 = ctx.encrypt(m2, &public_key, &mut rng);

    settings.bench_with_input(
        BenchmarkId::new("YASHE cipher mul", SMALL_RANDOM_NAME),
        &ctx,
        |benchmark, ctx| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark.iter_with_large_drop(|| -> Ciphertext<TestRes> {
                ctx.ciphertext_mul(m1.clone(), m2.clone())
            })
        },
    );
}

/// Run [`Yashe::keygen()`] as a Criterion benchmark with random data on middle resolution.
pub fn bench_keygen_mid(settings: &mut Criterion) {
    // Setup parameters
    let ctx: Yashe<MiddleRes> = Yashe::new();

    settings.bench_with_input(
        BenchmarkId::new("YASHE mid keygen", SMALL_RANDOM_NAME),
        &ctx,
        |benchmark, ctx| {
            // To avoid timing dropping the return value, we require it to be returned from the closure.
            benchmark.iter_with_large_drop(
                || -> (yashe::PrivateKey<MiddleRes>, yashe::PublicKey<MiddleRes>) {
                    ctx.keygen(&mut rand::thread_rng())
                },
            )
        },
    );
}
