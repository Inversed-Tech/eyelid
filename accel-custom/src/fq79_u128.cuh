// ---- Fu: Field as a single 128-bit integer ----
typedef unsigned __int128 Fu;

extern "C" __constant__ Fu FU_FQ79_MODULUS = 495925933090739208380417;

__device__ void add_Fu(const Fu &a, const Fu &b, Fu &out)
{
  out = a + b;
  if (out >= FU_FQ79_MODULUS)
    out -= FU_FQ79_MODULUS;

  /* Variant using the carry flag, apparently not working with u128.
  Fu out1 = a + b;
  Fu out2 = out1 - FU_FQ79_MODULUS;
  bool underflow = out1 < FU_FQ79_MODULUS;
  //auto underflow = subc(0, 0);
  out = underflow ? out1 : out2;
  */
}

extern "C" __global__ void sum_Fu(const Fu *a, const Fu *b, Fu *out, int count)
{
  for (int i = blockIdx.x * blockDim.x + threadIdx.x; i < count; i += blockDim.x * gridDim.x)
  {
    add_Fu(a[i], b[i], out[i]);
  }
}