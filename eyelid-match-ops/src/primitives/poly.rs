use ark_ff::MontConfig;
use ark_ff::Fp128;
use ark_ff::MontBackend;
use ark_poly::polynomial::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;
use ark_ff::Zero;
use ark_ff::One;

/// The maximum polynomial degree
const N: usize = 2048;

// Next we define 2 Finite Field using pre-computed primes and generators.
// We could also consider generating primes dynamically.

#[derive(MontConfig)]
#[modulus = "93309596432438992665667"]
#[generator = "5"]
/// Fq79Config
pub struct Fq79Config;
/// Params for full resolution (according to the report)
// t = 2ˆ15, q = 2ˆ79, N = 2048
// Sage commands:
// random_prime(2**79)
// 93309596432438992665667
// ff = GF(93309596432438992665667)
// ff.multiplicative_generator()
// 5
pub type Fq79 = Fp128<MontBackend<Fq79Config, 2>>;

#[derive(MontConfig)]
#[modulus = "33253620802622737871"]
#[generator = "14"]
/// Fq66Config
pub struct Fq66Config;
/// Params for full resolution (according to the report)
// t = 2ˆ12, q = 2ˆ66, N = 2048
// Sage commands:
// random_prime(2**79)
// 33253620802622737871
// ff = GF(33253620802622737871)
// ff.multiplicative_generator()
// 14
pub type Fq66 = Fp128<MontBackend<Fq66Config, 2>>;

#[test]
fn test_cyclotomic_mul(){
    let p1 = rand_pol();
    let mut xnm1 = DensePolynomial::<Fq79>::zero();
    xnm1.coeffs = [Fq79::zero(); N].to_vec();
    xnm1.coeffs[N-1] = Fq79::one(); // Xˆ{N-1}, multiplying but it will rotate by N-1 and negate (except the first)
    let res = cyclotomic_mul(p1.clone(), xnm1);
    for i in 0..N-1 {
        assert_eq!(res[i], - p1[i+1]);
    }
}

/// Generates a cyclotomic polynomial with random coefficients in Fq79
pub fn rand_pol() -> DensePolynomial::<Fq79>{
    let mut rng = ark_std::test_rng();
    DensePolynomial::<Fq79>::rand(N-1, &mut rng)
}

/// Use naive_mul followed by reduction mod XˆN - 1
pub fn cyclotomic_mul(a: DensePolynomial::<Fq79>, b: DensePolynomial::<Fq79>) -> DensePolynomial::<Fq79>{
    let mut res = a.naive_mul(&b);
    assert!(a.coeffs.len() <= N);
    assert!(b.coeffs.len() <= N);
    for i in 0..N {
        // In the cyclotomic ring we have that XˆN = -1, therefore all elements from N to 2N are negated
        if i + N < res.coeffs.len() {
            res[i] = res[i] - res[i + N];
            res[i + N] = Fq79::zero();
        };
    }
    res
}
