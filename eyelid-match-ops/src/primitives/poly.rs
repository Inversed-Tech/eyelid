use ark_ff::MontConfig;
use ark_ff::Fp128;
use ark_ff::MontBackend;
use ark_ff::BigInt;
use ark_poly::polynomial::univariate::DensePolynomial;
use rand::thread_rng;
use ark_std::rand::Rng;
use ark_poly::DenseUVPolynomial;
use std::ops::Mul;
use std::ops::Add;


//use crate::{
//    univariate::{DenseOrSparsePolynomial, SparsePolynomial},
//    DenseUVPolynomial, EvaluationDomain, Evaluations, GeneralEvaluationDomain, Polynomial,
//};
use ark_ff::{FftField, Field, Zero};
use ark_std::{
    fmt,
    ops::{AddAssign, Deref, DerefMut, Div, Neg, Sub, SubAssign},
    vec::*,
};

const N: usize = 2048;

// Next we define 2 Finite Field using pre-computed primes and generators.
// We could also consider generating primes dynamically.

/// Params for full resolution (according to the report)
// t = 2ˆ15, q = 2ˆ79, N = 2048
// Sage commands:
// random_prime(2**79)
// 93309596432438992665667
// ff = GF(93309596432438992665667)
// ff.multiplicative_generator()
// 5
#[derive(MontConfig)]
#[modulus = "93309596432438992665667"]
#[generator = "5"]
pub struct Fq79Config;
pub type Fq79 = Fp128<MontBackend<Fq79Config, 2>>;

/// Params for full resolution (according to the report)
// t = 2ˆ12, q = 2ˆ66, N = 2048
// Sage commands:
// random_prime(2**79)
// 33253620802622737871
// ff = GF(33253620802622737871)
// ff.multiplicative_generator()
// 14
#[derive(MontConfig)]
#[modulus = "33253620802622737871"]
#[generator = "14"]
pub struct Fq66Config;
pub type Fq66 = Fp128<MontBackend<Fq66Config, 2>>;

#[test]
fn test_mul(){
    let mut rng = thread_rng();
    let mut p1 = DensePolynomial::<Fq79>::rand(N, &mut rng);
    let mut p2 = DensePolynomial::<Fq79>::rand(N, &mut rng);
    dbg!(reduce_mul(p1, p2));
    // TODO: implement some test cases
}

pub fn reduce_mul(a: DensePolynomial::<Fq79>, b: DensePolynomial::<Fq79>) -> DensePolynomial::<Fq79>{
    let mut res = a.naive_mul(&b);
    assert_eq!(res.coeffs.len(), 2*N + 1);
    for i in 0..N {
        // In the cyclotomic ring we have that XˆN = -1, therefore all elements from N to 2N are negated
        res[i] = res[i] - res[i + N];
        res.coeffs[i + N] = Fq79::zero();
    }
    res
}
