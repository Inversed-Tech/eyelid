// ---- Fd: Field as two 64-bit limbs ----

#include "utils/int_types.cuh"
#include "utils/modifiers.cuh"
#include "utils/ptx.cuh"

const size_t FD_LIMBS = 2;

struct Fd
{
    u64 limbs[FD_LIMBS];
};

extern "C" __constant__
    Fd FD_FQ79_MODULUS = {0x32de29d2b16ce001, 0x6904};

// -MODULUS^{-1} mod 2^64
extern "C" __constant__
    u64 FD_FQ79_MODULUS_NEG_INV = 0xba55a944ed6cdfff;

// ---- Fd Addition ----

template <bool SUBTRACT, bool CARRY_OUT>
static constexpr DEVICE_INLINE u64
add_sub_u64_device(const u64 *x, const u64 *y, u64 *r, size_t n = FD_LIMBS)
{
    r[0] = SUBTRACT ? ptx::u64::sub_cc(x[0], y[0]) : ptx::u64::add_cc(x[0], y[0]);
    for (unsigned i = 1; i < n; i++)
        r[i] = SUBTRACT ? ptx::u64::subc_cc(x[i], y[i]) : ptx::u64::addc_cc(x[i], y[i]);
    if (!CARRY_OUT)
    {
        ptx::u64::addc(0, 0);
        return 0;
    }
    return SUBTRACT ? ptx::u64::subc(0, 0) : ptx::u64::addc(0, 0);
}

__device__ Fd
maybe_subtract_modulus_Fd(const Fd &a)
{
    Fd out_red = {};
    uint64_t underflow = add_sub_u64_device<true, true>(a.limbs, FD_FQ79_MODULUS.limbs, out_red.limbs);
    if (!underflow)
    {
        // printf("GPU substracting modulus.\n");
    }
    return underflow ? a : out_red;
}

__device__ void
add_Fd(const Fd &a, const Fd &b, Fd &out)
{
    add_sub_u64_device<false, false>(a.limbs, b.limbs, out.limbs);
    out = maybe_subtract_modulus_Fd(out);
}

extern "C" __global__ void
vec_add_Fd(const Fd *a, const Fd *b, Fd *out, int count)
{
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < count)
    {
        add_Fd(a[i], b[i], out[i]);
    }
}

// ---- Fd Montgomery multiplication.

/* Rust version from ark_ff:

                let mut r = [0u64; N];

                for i in 0..N {
                    let mut carry1 = 0u64;
                    r[0] = fa::mac(r[0], (a.0).0[0], (b.0).0[i], &mut carry1);

                    let k = r[0].wrapping_mul(Self::INV);

                    let mut carry2 = 0u64;
                    fa::mac_discard(r[0], k, Self::MODULUS.0[0], &mut carry2);

                    for j in 1..N {
                        r[j] = fa::mac_with_carry(r[j], (a.0).0[j], (b.0).0[i], &mut carry1);
                        r[j - 1] = fa::mac_with_carry(r[j], k, Self::MODULUS.0[j], &mut carry2);
                    }
                    r[N - 1] = carry1 + carry2;
                }
*/

static DEVICE_INLINE void
mont_mul_Fd(const u64 *a, const u64 *b, u64 *r)
{
    for (int i = 0; i < FD_LIMBS; i++)
    {
        // printf("\nGPU i=%d\n", i);
        // printf("GPU a[%d]=%llu b[%d]=%llu r[0]=%llu\n", 0, a[0], i, b[i], r[0]);

        u128 tmp = (u128)a[0] * (u128)b[i] + (u128)r[0];
        r[0] = (u64)tmp;
        u64 carry1 = (u64)(tmp >> 64);
        // printf("GPU r[0]=%llu, carry1=%llu\n", r[0], carry1);

        u64 k = r[0] * FD_FQ79_MODULUS_NEG_INV; // wrapping.
        // printf("GPU k=%llu\n", k);

        tmp = (u128)k * (u128)FD_FQ79_MODULUS.limbs[0] + (u128)r[0];
        u64 carry2 = (u64)(tmp >> 64);
        // printf("GPU carry2=%llu\n", carry2);

        for (int j = 1; j < FD_LIMBS; j++)
        {
            // printf("GPU   j=%d\n", j);

            tmp = (u128)r[j] + (u128)a[j] * (u128)b[i] + (u128)carry1;
            r[j] = (u64)tmp;
            carry1 = (u64)(tmp >> 64);
            // printf("GPU   r[%d]=%llu, carry1=%llu\n", j, r[j], carry1);

            tmp = (u128)r[j] + (u128)k * (u128)FD_FQ79_MODULUS.limbs[j] + (u128)carry2;
            r[j - 1] = (u64)tmp;
            carry2 = (u64)(tmp >> 64);
            // printf("GPU   r[%d]=%llu, carry2=%llu\n", j - 1, r[j - 1], carry2);
        }

        r[FD_LIMBS - 1] = carry1 + carry2;
        // printf("GPU r[0]=%llu r[1]=%llu\n", r[0], r[1]);
    }
}

__device__ void
mul_Fd(const Fd &a, const Fd &b, Fd &out)
{
    out = {};

    mont_mul_Fd(a.limbs, b.limbs, out.limbs);

    out = maybe_subtract_modulus_Fd(out);
    // printf("GPU out[0]=%llu out[1]=%llu\n", out.limbs[0], out.limbs[1]);
}

extern "C" __global__ void
vec_mul_Fd(const Fd *a, const Fd *b, Fd *out, int count)
{
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < count)
    {
        mul_Fd(a[i], b[i], out[i]);
    }
}
