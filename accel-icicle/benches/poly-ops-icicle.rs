use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use icicle_core::ntt::{self, NTT};
use icicle_core::ntt::{get_root_of_unity, initialize_domain, release_domain, NTTDomain};
use icicle_core::polynomials::UnivariatePolynomial;
use icicle_core::traits::ArkConvertible;
use icicle_core::traits::GenerateRandom;
use icicle_core::traits::{FieldConfig, FieldImpl, MontgomeryConvertible};
use icicle_core::vec_ops::{add_scalars, mul_scalars, sub_scalars, VecOps, VecOpsConfig};
use icicle_cuda_runtime::device_context::DeviceContext;
use icicle_cuda_runtime::memory::{DeviceSlice, DeviceVec, HostOrDeviceSlice, HostSlice};
use icicle_cuda_runtime::stream::CudaStream;
use icicle_inv_fhe79::{
    field::Fq79, field::ScalarCfg as FieldCfgGPU, field::ScalarField as FieldGPU,
    polynomials::DensePolynomial as PolyGPU,
};
use rand::random;

use accel_icicle::{alloc_elements_gpu, from_gpu, random_elements, setup, to_gpu, with_gpu};

pub fn bench_vec_ops(c: &mut Criterion) {
    let target_count: usize = 10_000_000;
    let vector_len = 2048;
    let vector_count = target_count.div_ceil(vector_len);
    let elem_count = vector_len * vector_count;

    let (vector_config, mut ntt_config) = setup();
    ntt_config.batch_size = vector_count as i32;

    let mut benches = c.benchmark_group(format!("icicle_vector_10M"));
    benches.throughput(Throughput::Elements(elem_count as u64));

    let (a_ark, a_gpu) = with_gpu(random_elements(elem_count));
    let (b_ark, b_gpu) = with_gpu(vec![Fq79::from(1); elem_count]);
    // TODO: Multiplication is not actually working, so we multiply by 1.
    //       Test random values after setting Barrett parameters the right way.

    // Addition.
    {
        let mut sum_gpu = alloc_elements_gpu(elem_count);

        benches.bench_function("add", |b| {
            b.iter(|| {
                add_scalars(
                    &a_gpu as &DeviceSlice<_>,
                    &b_gpu as &DeviceSlice<_>,
                    &mut sum_gpu as &mut DeviceSlice<_>,
                    &vector_config,
                )
                .unwrap();
            })
        });

        // Check results.
        let sum_ark = from_gpu(&sum_gpu);
        for i in 0..elem_count {
            assert_eq!(sum_ark[i], a_ark[i] + b_ark[i]);
        }
    }

    // Multiplication.
    {
        let mut prod_gpu = alloc_elements_gpu(elem_count);

        benches.bench_function("mul", |b| {
            b.iter(|| {
                mul_scalars(
                    &a_gpu as &DeviceSlice<_>,
                    &b_gpu as &DeviceSlice<_>,
                    &mut prod_gpu as &mut DeviceSlice<_>,
                    &vector_config,
                )
                .unwrap();
            })
        });

        // Check results.
        let prod_ark = from_gpu(&prod_gpu);
        for i in 0..elem_count {
            assert_eq!(prod_ark[i], a_ark[i] * b_ark[i]);
        }
    }

    // NTT.
    {
        // Allocate memory on CUDA device for NTT results
        let mut a_ntt_gpu = alloc_elements_gpu(elem_count);
        let mut a_ntt_roundtrip_gpu = alloc_elements_gpu(elem_count);

        benches.bench_function(format!("ntt_{}", vector_len), |b| {
            b.iter(|| {
                ntt::ntt(
                    &a_gpu as &DeviceSlice<_>,
                    ntt::NTTDir::kForward,
                    &ntt_config,
                    &mut a_ntt_gpu as &mut DeviceSlice<_>,
                )
                .expect("Failed to execute NTT");
            })
        });

        benches.bench_function(format!("ntt_inv_{}", vector_len), |b| {
            b.iter(|| {
                ntt::ntt(
                    &a_ntt_gpu as &DeviceSlice<_>,
                    ntt::NTTDir::kInverse,
                    &ntt_config,
                    &mut a_ntt_roundtrip_gpu as &mut DeviceSlice<_>,
                )
                .expect("Failed to execute NTT inversed");
            })
        });

        // TODO: enable after setting the roots of unity the right way.
        //assert_eq!(f_ark, from_gpu(&a_ntt_roundtrip_gpu));
    };

    benches.finish();
}

criterion_group!(benches, bench_vec_ops);
criterion_main!(benches);
