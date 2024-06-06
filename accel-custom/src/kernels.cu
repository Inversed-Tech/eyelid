#include "fq79_2x64.cuh"

// Endianness checker in case we reinterpret Rust data with different integer types.
extern "C" __global__ void endianness_check(const u128 *x, u64 *out_low, u64 *out_high, u64 *out_one)
{
  *out_low = (u64)*x;
  *out_high = (u64)(*x >> 64);
  *out_one = 1;
}
