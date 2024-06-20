# Eyelid

## Private iris matching using Homomorphic Encryption

In this repository, we implement the Homomorphic Encryption (HE) scheme called [YASHE](https://eprint.iacr.org/2013/075). The iris codes are encoded into polynomials, which can be encrypted using YASHE. Multiplying these polynomials gives us a result polynomial whose coefficients correspond to the Hamming distance we wish to compute.

The negacyclic ring structure of YASHE allows us to compute the Hamming distance of the expected rotations of the iris codes. By using this encoding, we can encrypt the query and the database containing the registered iris codes, ensuring that the computation of all the Hamming distances occurs privately, protected by the HE scheme.

We have tailored the YASHE construction as described in the original paper, removing the use of modulus switching and key switching (therefore no bootstrapping). To decrypt the homomorphic evaluation of a function $f$, it is necessary to apply $f$ to the private key to obtain the customized new private key needed for decryption. This is required because without key switching, the original private key only works to decrypt fresh ciphertexts.

We benchmarked the main components of the implementation, such that we can experiment with new ideas to optimize the code.

As the next steps, we plan to explore solutions to the bit extraction problem. Under certain circumstances, it is possible to optimize the computation of the post-processing of all the Hamming distances by performing it before decryption, using the HE scheme. This way, decryption of the homomorphic evaluation only reveals the bit required for matching, while hiding all the sensitive information.

Also, there is room for optimizations, such as improving the time to multiply polynomials, since this is the main building block used in the construction. Other optimizations are also possible, such as using the Chinese Remainder Theorem to drastically reduce the overhead of the HE scheme. This would allow us to construct batches of queries that can be encoded into a single ciphertext, considerably improving performance. Finally, we consider implementing it using GPUs and running it in better hardware to improve the benchmarks. 

## Testing

Temporarily switch to a tiny field to make test errors easier to debug:
```sh
RUSTFLAGS="--cfg tiny_poly" cargo test
RUSTFLAGS="--cfg tiny_poly" cargo bench --features benchmark
```

## Future Work

Benchmark Rust futures with `criterion` by enabling the [`async_tokio` feature](https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_async.html).

Benchmark using different configurations:
- `ark_poly` and `ark_ff` with the [`parallel` feature](https://github.com/search?q=repo%3Aarkworks-rs%2Falgebra+parallel&type=code),
- `ark_ff` with the [`asm` feature](https://github.com/arkworks-rs/algebra/blob/master/README.md#assembly-backend-for-field-arithmetic) on [`x86_64` only](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies). The README is outdated, `asm!()` is supported by the stable Rust compiler [on some platforms](https://doc.rust-lang.org/core/arch/index.html#modules), or
- compiling with [`RUSTFLAGS="--emit-asm"`](https://github.com/arkworks-rs/dpc/blob/bea4439a23fe0f3a8e857db5c0740d26e85bd726/README.md?plain=1#L83).

Produce different benchmark outputs from `criterion` by enabling the [`html_reports` and `plotters` features](https://bheisler.github.io/criterion.rs/book/user_guide/html_report.html).

Benchmark using different tools:
- instruction counts with [`iai`](https://bheisler.github.io/criterion.rs/book/iai/getting_started.html) or [`criterion-perf-events`](https://crates.io/crates/criterion-perf-events), or
- heaviest functions using [`flamegraph`](https://github.com/flamegraph-rs/flamegraph).

# Demo

In order to run the unit test that exercises the matching of homomorphically encrypted encodings do the following:

```sh
cargo test --release -- --nocapture encrypted::test::matching::test_matching_homomorphic_codes
```

To see that a different query indeed doesn't match, run the following:

```sh
cargo test --release -- --nocapture encrypted::test::matching::test_different_homomorphic_codes
```
