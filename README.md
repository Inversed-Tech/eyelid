# eyelid

Private iris matching

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
