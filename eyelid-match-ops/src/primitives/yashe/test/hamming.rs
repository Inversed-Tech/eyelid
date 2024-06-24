mod tests {

    use crate::encoded::conf::LargeRes;
    use crate::primitives::hamming::SimpleHammingEncoding;
    use crate::primitives::yashe::{Yashe, YasheConf};
    use crate::FullRes;

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
}
