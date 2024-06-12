

pub struct SimpleHammingEncoding<C> {
    m: Message<C>,
    m_rev: Message<C>,
}

/// SimpleHammingEncodingCiphertext is a struct that holds two ciphertexts, c and c_rev,
/// which are the encodings of the message m and m_rev, respectively. The encoding is
/// done by reversing the message and encoding it as a regular Yashe Ciphertext.
pub struct SimpleHammingEncodingCiphertext<C> {
    c: Ciphertext<C>,
    c_rev: Ciphertext<C>,
}

impl SimpleHammingEncoding {
    pub fn new(m: Message<C>) -> Self {
        let m_rev = m.clone().reverse();
        Self { m, m_rev }
    }

    /// Sample a random SimpleHammingEncoding, by sampling a random binary Yashe Message, which 
    /// is done by calling function sample_binary_message, and returning a new SimpleHammingEncoding,
    /// which sets m to the sampled message, and m_rev to the reverse of m.
    pub fn sample() -> SimpleHammingEncoding {
        let m = Yashe::sample_binary_message();
        SimpleHammingEncoding::new(m)
    }

    /// Compute the Hamming distance between two SimpleHammingEncoding v1 and v2. In order to do this,
    /// we subtract each component of the encoding, namely v1.m from v2.m and v1.m_rev from v2.m_rev, 
    /// and multiply the obtained Messages, returning a regular Yashe Message as output.
    pub fn hamming_distance(&self, v2: SimpleHammingEncoding) -> Message<C> {
        let m = self.m.sub(&v2.m);
        dbg!(m);
        let m_rev = self.m_rev.sub(&v2.m_rev);
        dbg!(m_rev);
        m.mul_assign(&m_rev);
        m
    }

    /// Encrypts the message m encoded as a SimpleHammingEncoding, which is done by encrypting
    /// each component of the encoding separately, and returning a SimpleHammingEncodingCiphertext.
    pub fn encrypt_simple_hamming_encoding(&self) -> SimpleHammingEncodingCiphertext<C> {
        let c = Ciphertext {
            c: self.m.encrypt(),
            c_rev: self.m_rev.encrypt(),
        };
        SimpleHammingEncodingCiphertext { c, c_rev }
    }

    /// Decrypts the SimpleHammingEncodingCiphertext c, by decrypting each component of the encoding
    /// separately, and returning the result as a SimpleHammingEncoding.
    pub fn decrypt_simple_hamming_encoding(c: SimpleHammingEncodingCiphertext<C>) -> SimpleHammingEncoding {
        let m = c.c.decrypt();
        let m_rev = c.c_rev.decrypt();
        SimpleHammingEncoding { m, m_rev }
    }

    /// In order to homomorphically compute the hamming distance between two 
    /// SimpleHammingEncodingCiphertexts, we need to subtract each 
    /// component respectivelly. Namely, given c1 and c2, we need to compute
    /// a SimpleHammingEncodingCiphertext c, such that c.c = c1.c - c2.c, 
    /// and c.c_rev = c1.c_rev - c2.c_rev. Then we multiply c.c by c.c_rev 
    /// and return the result as a regular Yashe Ciphertext.
    pub fn homomorphic_hamming_distance(c1: SimpleHammingEncodingCiphertext<C>, c2: SimpleHammingEncodingCiphertext<C>) -> Ciphertext<C> {
        let c = Ciphertext {
            c: c1.c.sub(&c2.c),
            c_rev: c1.c_rev.sub(&c2.c_rev),
        };
        c.c.mul_assign(&c.c_rev);
        c
    }
}