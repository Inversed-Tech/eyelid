//! Full-resolution parameters in 2^123.

use ark_ff::{Fp128, MontBackend, MontConfig};

/// The configuration of the modular field used for polynomial coefficients.
/* Generated with the following Sage commands:

```sage
def find_q(size):
    for i in range(1000):
        q = random_prime(maxi)
        if (q - 1) % 2048 == 0:
            if q > 2**122:
                return q
    return -1

q = find_q(size)
q
5825476135918962761812038067936663553
assert 2**122 < q < 2**123
generator = GF(q).multiplicative_generator()
generator
3
```

assert q - 1 == 2^11 * 6109107769 * 465611489770345388064971

omega = pow(generator, 6109107769 * 465611489770345388064971, q)
assert generator == 3
assert omega == 824405581759706812948635940988472411
assert pow(omega, 2**11, q) == 1
assert pow(omega, 2**10, q) != 1
```
*/
#[derive(MontConfig)]
#[modulus = "5825476135918962761812038067936663553"]
#[generator = "3"]
pub struct Fq123Config;

/// The modular field used for polynomial coefficients, with precomputed primes and generators.
pub type Fq123 = Fp128<MontBackend<Fq123Config, 2>>;
