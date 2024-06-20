#[cfg(test)]
mod tests {

    use crate::encoded::conf::LargeRes;
    use crate::primitives::hamming::SimpleHammingEncoding;
    use crate::FullRes;
    use crate::{
        primitives::yashe::{Yashe, YasheConf},
        //        FullRes,
    };

    #[test]
    fn test_hamming_distance() {
        hamming_distance_helper::<FullRes>();
        hamming_distance_helper::<LargeRes>();
    }

    fn hamming_distance_helper<C: YasheConf>()
    where
        C::Coeff: From<u128> + From<u64> + From<i64>,
    {
        let mut rng = rand::thread_rng();
        let ctx: Yashe<C> = Yashe::new();
        let (private_key, public_key) = ctx.keygen(&mut rng);
        // Must be smaller than or equal to MAX_POLY_DEGREE
        let size = 1000;

        let v1 = SimpleHammingEncoding::sample(ctx, size, &mut rng);
        let v2 = SimpleHammingEncoding::sample(ctx, size, &mut rng);
        let c1 = v1.encrypt_simple_hamming_encoding(ctx, &public_key, &mut rng);
        let c2 = v2.encrypt_simple_hamming_encoding(ctx, &public_key, &mut rng);
        let c = c1.homomorphic_hamming_distance(ctx, c2);
        let m = ctx.decrypt_mul(c, &private_key);

        let hd = v1.hamming_distance(v2, size);
        assert_eq!(m.m[size - 1], hd);
    }

    /*#[test]
    /// Next test is disabled because it is not working. 
    /// It is a beginning of an experimentation to homomorphically
    /// compute the bit extraction by dividing by small powers of 2.
    fn test_hamming_distance_below_threshold() {
        hamming_distance_below_threshold_helper::<FullRes>();
    }

    fn hamming_distance_below_threshold_helper<C: YasheConf>()
    where
        C::Coeff: From<u128> + From<u64> + From<i64>,
    {
        let mut rng = rand::thread_rng();
        let ctx: Yashe<C> = Yashe::new();
        let (private_key, public_key) = ctx.keygen(&mut rng);
        // Must be smaller than or equal to MAX_POLY_DEGREE
        let size = 2048;

        let v1 = SimpleHammingEncoding::sample(ctx, size, &mut rng);
        let v2 = SimpleHammingEncoding::sample(ctx, size, &mut rng);
        let c1 = v1.encrypt_simple_hamming_encoding(ctx, &public_key, &mut rng);
        let c2 = v2.encrypt_simple_hamming_encoding(ctx, &public_key, &mut rng);
        let mut c = c1.homomorphic_hamming_distance(ctx, c2);

        // just divide by 2
        #[allow(unused_mut)]
        for mut coeff in c.c.coeffs_mut() {
            let mut coeff_res = C::coeff_as_u128(*coeff);
            coeff_res /= 2;
            *coeff = coeff_res.into();
        }

        let m = ctx.decrypt_mul(c, &private_key);
        dbg!(m.m[size - 1]);

        let hd = v1.hamming_distance(v2, size);
        let mut hd_res = C::coeff_as_u128(hd);
        dbg!(hd_res);
        hd_res /= 2;
        dbg!(hd_res);
        hd_res = hd_res % u128::from(C::T);
        dbg!(hd_res);
        let m_res = C::coeff_as_u128(m.m[size - 1]) % u128::from(C::T);
        let neg = C::T as i64 - m_res as i64;
        dbg!(neg);
        assert_eq!(m_res, hd_res.into());
        //assert_eq!(m.m[size - 1], hd_res.into());
    }*/
}
