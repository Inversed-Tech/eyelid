#include <cstdint>
#include "utils/modifiers.cuh"
#include "utils/ptx.cuh"

// ---- Ft: Field as three u32. ----

const size_t FT_LIMBS = 3;
struct Ft
{
  u32 limbs[FT_LIMBS];
  u32 _pad; // Match the layout of two u64.
};

extern "C" __constant__
    Ft FT_FQ79_MODULUS = {0xb16ce001, 0x32de29d2, 0x6904, 0};

// -MODULUS^{-1} mod 2^32
extern "C" __constant__
    u32 FT_FQ79_MODULUS_NEG_INV = 0xed6cdfff;

template <bool SUBTRACT, bool CARRY_OUT>
static constexpr DEVICE_INLINE uint32_t
add_sub_u32_device(const uint32_t *x, const uint32_t *y, uint32_t *r, size_t n = FT_LIMBS)
{
  r[0] = SUBTRACT ? ptx::sub_cc(x[0], y[0]) : ptx::add_cc(x[0], y[0]);
  for (unsigned i = 1; i < n; i++)
    r[i] = SUBTRACT ? ptx::subc_cc(x[i], y[i]) : ptx::addc_cc(x[i], y[i]);
  if (!CARRY_OUT)
  {
    ptx::addc(0, 0);
    return 0;
  }
  return SUBTRACT ? ptx::subc(0, 0) : ptx::addc(0, 0);
}

__device__ void
add_Ft(const Ft &a, const Ft &b, Ft &out)
{
  add_sub_u32_device<false, false>(a.limbs, b.limbs, out.limbs);
  // Maybe reduce.
  Ft out_red = {};
  uint32_t underflow = add_sub_u32_device<true, true>(out.limbs, FT_FQ79_MODULUS.limbs, out_red.limbs);
  if (!underflow)
    out = out_red;
  out._pad = 0;
}

extern "C" __global__ void
vec_add_Ft(const Ft *a, const Ft *b, Ft *out, int count)
{
  //for (int i = blockIdx.x * blockDim.x + threadIdx.x; i < count; i += blockDim.x * gridDim.x)
  int i = blockIdx.x * blockDim.x + threadIdx.x;
  if (i < count)
  {
    add_Ft(a[i], b[i], out[i]);
  }
}

// ---- Ft wide multiplication.

struct FtWide
{
  u32 limbs[FT_LIMBS * 2];
};

static DEVICE_INLINE void
multiply_raw_device(const Ft &as, const Ft &bs, FtWide &rs)
{
  const uint32_t *a = as.limbs;
  const uint32_t *b = bs.limbs;
  uint32_t *r = rs.limbs;

  __align__(8) uint32_t odd[4];
  __align__(8) uint32_t even[2];

  r[0] = ptx::mul_lo(a[0], b[0]);
  r[1] = ptx::mul_hi(a[0], b[0]);
  r[2] = ptx::mul_lo(a[1], b[1]);
  r[3] = ptx::mul_hi(a[1], b[1]);
  r[4] = ptx::mul_lo(a[2], b[2]);
  r[5] = ptx::mul_hi(a[2], b[2]);

  odd[0] = ptx::mul_lo(a[0], b[1]);
  odd[0] = ptx::mad_lo(a[1], b[0], odd[0]);

  odd[1] = ptx::mul_hi(a[0], b[1]);
  odd[1] = ptx::mad_hi(a[1], b[0], odd[1]);

  odd[2] = ptx::mul_lo(a[1], b[2]);
  odd[2] = ptx::mad_lo(a[2], b[1], odd[2]);

  odd[3] = ptx::mul_hi(a[1], b[2]);
  odd[3] = ptx::mad_hi(a[2], b[1], odd[3]);

  r[1] = ptx::add_cc(r[1], odd[0]);
  r[2] = ptx::addc_cc(r[2], odd[1]);
  r[3] = ptx::addc_cc(r[3], odd[2]);
  r[4] = ptx::addc_cc(r[4], odd[3]);
  r[5] = ptx::addc(r[5], 0);

  even[0] = ptx::mul_lo(a[0], b[2]);
  even[0] = ptx::mad_lo(a[2], b[0], even[0]);

  even[1] = ptx::mul_hi(a[0], b[2]);
  even[1] = ptx::mad_hi(a[2], b[0], even[1]);

  r[2] = ptx::add_cc(r[2], even[0]);
  r[3] = ptx::addc_cc(r[3], even[1]);
  r[4] = ptx::addc_cc(r[4], 0);
  r[5] = ptx::addc(r[5], 0);
}

__device__ void
mul_wide_Ft(const Ft &a, const Ft &b, Ft &out)
{
  FtWide product;
  multiply_raw_device(a, b, product);

  // TODO: Montgomery reduction.

  for (int i = 0; i < FT_LIMBS; i++)
  {
    out.limbs[i] = product.limbs[i];
  }
}

// ---- Ft Montgomery multiplication.

static DEVICE_INLINE void
mont_mul_Ft(const u32 *a, const u32 *b, u32 *r)
{
  for (int i = 0; i < FT_LIMBS; i++)
  {
    //printf("GPU a[%d]=%u b[%d]=%u r[0]=%u\n", 0, a[0], i, b[i], r[0]);
    u64 tmp = (u64)a[0] * (u64)b[i] + (u64)r[0];
    r[0] = (u32)tmp;
    u32 carry1 = (u32)(tmp >> 32);
    //printf("GPU tmp=%llu, r[0]=%u, carry1=%u\n", tmp, r[0], carry1);

    u32 k = r[0] * FT_FQ79_MODULUS_NEG_INV; // wrapping.
    //printf("GPU k=%u\n", k);

    tmp = (u64)k * (u64)FT_FQ79_MODULUS.limbs[0] + (u64)r[0];
    u32 carry2 = (u32)(tmp >> 32);

    for (int j = 1; j < FT_LIMBS; j++)
    {
      tmp = (u64)r[j] + (u64)a[j] * (u64)b[i] + (u64)carry1;
      r[j] = (u32)tmp;
      carry1 = (u32)(tmp >> 32);

      tmp = (u64)r[j] + (u64)k * (u64)FT_FQ79_MODULUS.limbs[j] + (u64)carry2;
      r[j - 1] = (u32)tmp;
      carry2 = (u32)(tmp >> 32);
    }
    r[FT_LIMBS - 1] = carry1 + carry2;
  }
}

__device__ void
mul_Ft(const Ft &a, const Ft &b, Ft &out)
{
  out = {};

  mont_mul_Ft(a.limbs, b.limbs, out.limbs);

  // Maybe reduce.
  Ft out_red = {};
  uint32_t underflow = add_sub_u32_device<true, true>(out.limbs, FT_FQ79_MODULUS.limbs, out_red.limbs);
  if (!underflow)
    out = out_red;
}

extern "C" __global__ void
vec_mul_Ft(const Ft *a, const Ft *b, Ft *out, int count)
{
  //for (int i = blockIdx.x * blockDim.x + threadIdx.x; i < count; i += blockDim.x * gridDim.x)
  int i = blockIdx.x * blockDim.x + threadIdx.x;
  if (i < count)
  {
    mul_Ft(a[i], b[i], out[i]);
  }
}
