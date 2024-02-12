# eyelid
Private iris matching

## Future Work

Benchmark comparisons with:
- `ark_poly` and `ark_ff` with the [`parallel` feature](https://github.com/search?q=repo%3Aarkworks-rs%2Falgebra+parallel&type=code),
- `ark_ff` with the [`asm` feature](https://github.com/arkworks-rs/algebra/blob/master/README.md#assembly-backend-for-field-arithmetic) on [`x86_64` only](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#platform-specific-dependencies). The README is outdated, `asm!()` is supported by the stable Rust compiler [on some platforms](https://doc.rust-lang.org/core/arch/index.html#modules), or
- compiling with [`RUSTFLAGS="--emit-asm"`](https://github.com/arkworks-rs/dpc/blob/bea4439a23fe0f3a8e857db5c0740d26e85bd726/README.md?plain=1#L83).
