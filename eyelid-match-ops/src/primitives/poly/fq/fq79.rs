//! Full-resolution parameters in 2^79.
//!
//! These are the parameters for full resolution, according to the Inversed Tech report.
//! t = 2ˆ15, q = 2ˆ79

use ark_ff::{Fp128, MontBackend, MontConfig};

/// The configuration of the modular field used for polynomial coefficients.
/* Generated with the following Sage commands:

```sage
maxi = 2**79
for i in range(1000):
    q = random_prime(maxi)
    if (q - 1) % 2048 == 0:
        print("OK", q)
```

```sage
q = 495925933090739208380417
assert 2**78 < q < 2**79
assert q - 1 == 2**13 * 23 * 271 * 9712471302621631

generator = GF(q).multiplicative_generator()
omega = pow(generator, 23 * 271 * 9712471302621631, q)
assert generator == 3
assert omega == 460543614695341080498621
assert pow(omega, 2**13, q) == 1
assert pow(omega, 2**12, q) != 1
```
*/
#[derive(MontConfig)]
#[modulus = "495925933090739208380417"]
#[generator = "3"]
pub struct Fq79Config;

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
pub type Fq79 = Fp128<MontBackend<Fq79Config, 2>>;
