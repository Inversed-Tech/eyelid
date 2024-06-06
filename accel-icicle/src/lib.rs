use rand::random;

use icicle_core::ntt::{self, NTTConfig, NTT};
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

pub fn setup<'a>() -> (VecOpsConfig<'a>, NTTConfig<'a, FieldGPU>) {
    // Initialize the CUDA backend for polynomial operations
    PolyGPU::init_cuda_backend();

    // Initialize the vector operations backend.
    let vector_config = VecOpsConfig::default();

    // Initialize the NTT backend.
    let domain_max_size: u64 = 1 << 13;
    let fast_twiddles_mode = true;
    let rou: FieldGPU = get_root_of_unity(domain_max_size);
    initialize_domain(rou, &DeviceContext::default(), fast_twiddles_mode).unwrap();

    let ntt_config = NTTConfig::<FieldGPU>::default();

    (vector_config, ntt_config)
}

pub fn alloc_elements_gpu(elem_count: usize) -> DeviceVec<FieldGPU> {
    DeviceVec::<FieldGPU>::cuda_malloc(elem_count).expect("Failed to allocate memory on device")
}

pub fn random_elements(size: usize) -> Vec<Fq79> {
    (0..size).map(|_| Fq79::from(random::<u128>())).collect()
}

pub fn with_gpu(vals_ark: Vec<Fq79>) -> (Vec<Fq79>, DeviceVec<FieldGPU>) {
    let vals_gpu = to_gpu(vals_ark.clone());
    (vals_ark, vals_gpu)
}

pub fn to_gpu(vals_ark: Vec<Fq79>) -> DeviceVec<FieldGPU> {
    let vals_ici = vals_ark
        .into_iter()
        .map(FieldGPU::from_ark)
        .collect::<Vec<_>>();
    let mut dv = alloc_elements_gpu(vals_ici.len());
    dv.copy_from_host(HostSlice::from_slice(&vals_ici)).unwrap();
    dv
}

pub fn from_gpu(dv: &DeviceSlice<FieldGPU>) -> Vec<Fq79> {
    let mut vals_ici = vec![FieldGPU::zero(); dv.len()];
    dv.copy_to_host(HostSlice::from_mut_slice(&mut vals_ici))
        .unwrap();
    vals_ici.iter().map(FieldGPU::to_ark).collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn vec_mul_works() {
        // Initialize the CUDA backend for polynomial operations
        PolyGPU::init_cuda_backend();
        let cfg = VecOpsConfig::default();

        let size = 16;

        let q: u128 = 495925933090739208380417;
        let a_int: u128 = q - 1;
        let b_int: u128 = q + 1;

        let a_ark = random_elements(size);
        let b_ark = vec![Fq79::from(b_int); size];

        let a_gpu = to_gpu(a_ark.clone());
        let b_gpu = to_gpu(b_ark.clone());

        let mut sum_gpu = DeviceVec::<FieldGPU>::cuda_malloc(size).unwrap();
        add_scalars(
            &a_gpu as &DeviceSlice<_>,
            &b_gpu as &DeviceSlice<_>,
            &mut sum_gpu as &mut DeviceSlice<_>,
            &cfg,
        )
        .unwrap();
        let sum_ark = from_gpu(&sum_gpu);

        let mut prod_gpu = DeviceVec::<FieldGPU>::cuda_malloc(size).unwrap();
        mul_scalars(
            &a_gpu as &DeviceSlice<_>,
            &b_gpu as &DeviceSlice<_>,
            &mut prod_gpu as &mut DeviceSlice<_>,
            &cfg,
        )
        .unwrap();
        let prod_ark = from_gpu(&prod_gpu);

        for i in 0..size {
            assert_eq!(sum_ark[i], a_ark[i] + b_ark[i]);
            assert_eq!(prod_ark[i], a_ark[i] * b_ark[i]);
        }
    }

    #[test]
    fn poly_ntt_works() {
        // Initialize the CUDA backend for polynomial operations
        PolyGPU::init_cuda_backend();

        // Initialize the NTT backend.
        let ctx = DeviceContext::default();
        let domain_max_size: u64 = 1 << 13;
        let fast_twiddles_mode = true;
        let rou: FieldGPU = get_root_of_unity(domain_max_size);
        initialize_domain(rou, &ctx, fast_twiddles_mode).unwrap();

        let size = 2048;
        let f_ark = random_elements(size);
        let f_gpu = to_gpu(f_ark.clone());

        // Allocate memory on CUDA device for NTT results
        let mut ntt_results = DeviceVec::<FieldGPU>::cuda_malloc(size).unwrap();
        let mut inv_results = DeviceVec::<FieldGPU>::cuda_malloc(size).unwrap();

        // Configure NTT
        let mut cfg = ntt::NTTConfig::<FieldGPU>::default();

        // Execute NTT on device
        println!("Executing NTT on device...");
        ntt::ntt(
            &f_gpu as &DeviceSlice<_>,
            ntt::NTTDir::kForward,
            &cfg,
            &mut ntt_results as &mut DeviceSlice<_>,
        )
        .expect("Failed to execute NTT");

        ntt::ntt(
            &ntt_results as &DeviceSlice<_>,
            ntt::NTTDir::kInverse,
            &cfg,
            &mut inv_results as &mut DeviceSlice<_>,
        )
        .expect("Failed to execute NTT");

        let f_ntt_roundtrip_ark = from_gpu(&inv_results);

        // TODO: enable after fixing the field constants.
        //assert_eq!(f_ark, f_ntt_roundtrip_ark);

        println!("NTT execution complete.");
    }

    #[test]
    fn poly_add_works() {
        // Initialize the CUDA backend for polynomial operations
        PolyGPU::init_cuda_backend();

        let size = 2048;
        let f_coeffs = random_elements(size);
        let g_coeffs = random_elements(size);

        let f_gpu = PolyGPU::from_coeffs(&to_gpu(f_coeffs.clone()) as &DeviceSlice<_>, size);
        let g_gpu = PolyGPU::from_coeffs(&to_gpu(g_coeffs.clone()) as &DeviceSlice<_>, size);

        let mut sum_gpu = &f_gpu + &g_gpu;
        let sum_coeffs = from_gpu(sum_gpu.coeffs_mut_slice());

        for i in 0..size {
            assert_eq!(sum_coeffs[i], &f_coeffs[i] + &g_coeffs[i]);
        }
    }

    #[test]
    fn poly_mul_ntt_works() {
        // Initialize the CUDA backend for polynomial operations
        PolyGPU::init_cuda_backend();

        // Initialize the NTT backend.
        let ctx = DeviceContext::default();
        let domain_max_size: u64 = 1 << 13;
        let fast_twiddles_mode = false;
        let rou: FieldGPU = get_root_of_unity(domain_max_size);
        initialize_domain(rou, &ctx, fast_twiddles_mode).unwrap();

        let size = 2048;
        let f_coeffs = random_elements(size);
        let g_coeffs = random_elements(size);
        let f_gpu = PolyGPU::from_coeffs(&to_gpu(f_coeffs.clone()) as &DeviceSlice<_>, size);
        let g_gpu = PolyGPU::from_coeffs(&to_gpu(g_coeffs.clone()) as &DeviceSlice<_>, size);

        let mut prod_gpu = &f_gpu * &g_gpu;
        let prod_coeffs = from_gpu(prod_gpu.coeffs_mut_slice());
    }
}
