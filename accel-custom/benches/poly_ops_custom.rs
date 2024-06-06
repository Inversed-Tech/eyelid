//! Experimental GPU acceleration.
#![allow(unsafe_code, unused_mut)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use cust::device::Device;
use cust::error::CudaResult;
use cust::launch;
use cust::memory::*;
use cust::module::Module;
use cust::stream::*;
use cust::CudaApiVersion;

use ark_ff::{BigInteger, Field, MontConfig, PrimeField, Zero};
use eyelid_match_ops::primitives::poly::fq::{Fq79 as F, Fq79Config};
use num_bigint::BigUint;
use rand::random;

const R_INV: u128 = 242210205320934764651731;

#[derive(PartialEq, Eq, Debug, Copy, Clone, DeviceCopy)]
#[repr(C, align(16))]
struct XY {
    x: u64,
    y: u64,
}

fn check_endianness() -> Result<(), Box<dyn std::error::Error>> {
    let ptx = include_str!("../kernels.ptx");
    let module = Module::from_ptx(&ptx, &[])?;
    let stream = Stream::new(StreamFlags::NON_BLOCKING, None)?;

    let f = F::from(1);
    let f_limbs = UnifiedBox::new(f.0 .0)?;

    let lo = UnifiedBox::new(0u64)?;
    let hi = UnifiedBox::new(0u64)?;
    let one = UnifiedBox::new(0u64)?;
    unsafe {
        launch!(module.endianness_check<<<1, 1, 0, stream>>>(
            f_limbs.as_device_ptr(),
            lo.as_device_ptr(),
            hi.as_device_ptr(),
            one.as_device_ptr()
        ))?;
    }
    stream.synchronize()?;

    assert_eq!(*one, 1, "Incompatible u64 endianness!");
    assert_eq!([*lo, *hi], f.0 .0, "Incompatible Field / u128 endianness!");
    Ok(())
}

pub fn bench_vec_ops(c: &mut Criterion) -> Result<(), Box<dyn std::error::Error>> {
    assert!(
        Fq79Config::CAN_USE_NO_CARRY_MUL_OPT,
        "CAN_USE_NO_CARRY_MUL_OPT is required."
    );

    let target_count: usize = 10_000_000;
    let vector_len = 2048;
    let vector_count = target_count.div_ceil(vector_len);
    let elem_count = vector_len * vector_count;
    let block_dim: u32 = 256;
    let grid_dim = (elem_count as u32).div_ceil(block_dim);

    let mut benches = c.benchmark_group(format!("custom_vector_10M"));
    benches.throughput(Throughput::Elements(elem_count as u64));

    let _ctx = cust::quick_init()?;
    let cu_version = CudaApiVersion::get()?;
    let devices = Device::devices()?.collect::<CudaResult<Vec<_>>>()?;
    let device = Device::get_device(0)?;
    println!("Hello, world! {:?}. {} devices.", cu_version, devices.len());

    check_endianness()?;

    // Set up the context, load the module, and create a stream to run kernels in.
    // Compile with:
    //     nvcc src/kernels.cu --ptx
    let ptx = include_str!("../kernels.ptx");
    let module = Module::from_ptx(&ptx, &[])?;
    let stream = Stream::new(StreamFlags::NON_BLOCKING, None)?;

    println!("Generating data…");
    let a = F::from(random::<u128>());
    let b = F::from(random::<u128>());
    let a_p_b = to_limbs(&(a + b));
    let a_t_b = to_limbs(&(a * b));

    let mut a_gpu = DeviceBuffer::from_slice(&vec![to_limbs(&a); elem_count])?;
    let mut b_gpu = DeviceBuffer::from_slice(&vec![to_limbs(&b); elem_count])?;
    println!("Data generated.");

    println!("\n---- Addition on GPU ----");
    let mut sum_result = vec![[0u64; 2]; elem_count];
    {
        let mut sum_gpu = DeviceBuffer::from_slice(&vec![[0u64; 2]; elem_count])?;

        benches.bench_function("add", |b| {
            b.iter(|| {
                unsafe {
                    launch!(module.vec_add_Fd<<<grid_dim, block_dim, 0, stream>>>(
                        a_gpu.as_device_ptr(),
                        b_gpu.as_device_ptr(),
                        sum_gpu.as_device_ptr(),
                        sum_gpu.len()
                    ))
                    .expect("Failed to launch kernel");
                }
                stream.synchronize().expect("Failed to synchronize stream");
            });
        });

        sum_gpu.copy_to(&mut sum_result)?;
        println!("Addition on GPU done.");
    }

    // Check results.
    println!("x: {:?}", to_limbs(&a));
    println!("y: {:?}", to_limbs(&b));
    println!("x+y: {:?}", a_p_b);
    println!("gpu:   {:?}", sum_result[0]);
    for got in &sum_result {
        assert_eq!(got, &a_p_b);
    }
    println!("Addition works!");

    println!("\n---- Multiplication on GPU ----");
    let mut prod_result = vec![[0u64; 2]; elem_count];
    {
        let mut prod_gpu = DeviceBuffer::from_slice(&vec![[0u64; 2]; elem_count])?;

        benches.bench_function("mul", |b| {
            b.iter(|| {
                unsafe {
                    launch!(module.vec_mul_Fd<<<grid_dim, block_dim, 0, stream>>>(
                        a_gpu.as_device_ptr(),
                        b_gpu.as_device_ptr(),
                        prod_gpu.as_device_ptr(),
                        prod_gpu.len()
                    ))
                    .expect("Failed to launch kernel");
                }
                stream.synchronize().expect("Failed to synchronize stream");
            });
        });

        prod_gpu.copy_to(&mut prod_result)?;
        println!("Multiplication on GPU done.");
    }

    println!("\n---- Multiplication on CPU (debug) ----");
    let prod_raw = mul_limbs_u64(to_limbs(&a), to_limbs(&b));
    assert_eq!(prod_raw, a_t_b);

    println!();
    println!("x*y (ref):    {:?}", a_t_b);
    println!("gpu:          {:?}", prod_result[0]);
    println!();
    for got in &prod_result {
        assert_eq!(got, &a_t_b);
    }
    println!("Multiplication works!");
    
    Ok(())
}

fn to_limbs(f: &F) -> [u64; 2] {
    f.0 .0
}

fn from_limbs(limbs: [u64; 2]) -> F {
    let mut f = F::zero();
    f.0 .0 = limbs;
    f
}

fn to_limbs_32(f: &F) -> [u32; 4] {
    let limbs = f.0 .0;
    [
        limbs[0] as u32,
        (limbs[0] >> 32) as u32,
        limbs[1] as u32,
        (limbs[1] >> 32) as u32,
    ]
}

fn from_limbs_32(limbs: [u32; 4]) -> F {
    let mut f = F::zero();
    f.0 .0 = [
        limbs[0] as u64 | ((limbs[1] as u64) << 32),
        limbs[2] as u64 | ((limbs[3] as u64) << 32),
    ];
    f
}

fn _redc_limbs(limbs: [u64; 2]) -> [u64; 2] {
    to_limbs(&(&from_limbs(limbs) * &F::from(R_INV)))
}

fn mul_limbs_u64(a: [u64; 2], b: [u64; 2]) -> [u64; 2] {
    use ark_ff::biginteger::arithmetic as fa;
    const N: usize = 2;

    let mut r = [0u64; N];

    for i in 0..N {
        println!("\nCPU i: {}", i);
        println!("CPU a[{}]={} b[{}]={} r[0]={}", 0, a[0], i, b[i], r[0]);

        let mut carry1 = 0u64;
        r[0] = fa::mac(r[0], a[0], b[i], &mut carry1);
        println!("CPU r[0]={} carry1={}", r[0], carry1);

        let k = r[0].wrapping_mul(Fq79Config::INV);
        println!("CPU k={}", k);

        let mut carry2 = 0u64;
        fa::mac_discard(r[0], k, Fq79Config::MODULUS.0[0], &mut carry2);
        println!("CPU carry2={}", carry2);

        for j in 1..N {
            println!("CPU   j={}", j);

            r[j] = fa::mac_with_carry(r[j], a[j], b[i], &mut carry1);
            println!("CPU   r[{}]={} carry1={}", j, r[j], carry1);

            r[j - 1] = fa::mac_with_carry(r[j], k, Fq79Config::MODULUS.0[j], &mut carry2);
            println!("CPU   r[{}]={} carry2={}", j - 1, r[j - 1], carry2);
        }

        r[N - 1] = carry1 + carry2;
        println!("CPU r[0]={} r[1]={}", r[0], r[1]);
    }

    let mut rf = from_limbs(r);
    subtract_modulus(&mut rf);
    let out = to_limbs(&rf);
    println!("CPU out[0]={} out[1]={}", out[0], out[1]);

    out
}

fn subtract_modulus(f: &mut F) {
    if f.is_geq_modulus() {
        println!("CPU subtracting modulus!");
        f.0.sub_with_borrow(&F::MODULUS);
    }
}

criterion_group!(benches, bench_vec_ops);
criterion_main!(benches);
