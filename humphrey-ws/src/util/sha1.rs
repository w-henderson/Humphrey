#![allow(clippy::many_single_char_names)]

use std::convert::TryInto;

/// A trait which represents the ability to be hashed using SHA-1.
/// This is implemented for all types which implement `AsRef<[u8]>`.
pub trait SHA1Hash {
    /// Hashes self using SHA-1.
    fn hash(&self) -> [u8; 20];
}

impl<T> SHA1Hash for T
where
    T: AsRef<[u8]>,
{
    fn hash(&self) -> [u8; 20] {
        // Calculate padded message length then perform padding
        let message_len = ((self.as_ref().len() * 8 + 583) / 512) * 64;
        let mut message: Vec<u8> = vec![0; message_len];
        message[0..self.as_ref().len()].copy_from_slice(self.as_ref());
        message[self.as_ref().len()] = 0x80;
        message[message_len - 8..].copy_from_slice(&(self.as_ref().len() * 8).to_be_bytes());

        // Initialize hash values
        let mut h0: u32 = 0x67452301;
        let mut h1: u32 = 0xEFCDAB89;
        let mut h2: u32 = 0x98BADCFE;
        let mut h3: u32 = 0x10325476;
        let mut h4: u32 = 0xC3D2E1F0;

        // Process message in 16-byte chunks
        let mut chunk: [u32; 80] = [0; 80];
        for chunk_id in 0..message_len / 64 {
            // Break chunk into sixteen 32-bit integers
            for i in 0..16 {
                chunk[i] = u32::from_be_bytes(
                    message[(chunk_id * 64) + (i * 4)..(chunk_id * 64) + (i * 4) + 4]
                        .try_into()
                        .unwrap(),
                );
            }

            // Extend the chunk into 80 words
            for i in 16..80 {
                chunk[i] = chunk[i - 3] ^ chunk[i - 8] ^ chunk[i - 14] ^ chunk[i - 16];
                chunk[i] = chunk[i].rotate_left(1);
            }

            // Initialize hash value for this chunk
            let mut a: u32 = h0;
            let mut b: u32 = h1;
            let mut c: u32 = h2;
            let mut d: u32 = h3;
            let mut e: u32 = h4;

            // Main loop
            for (i, tem) in chunk.iter().enumerate() {
                let (f, k) = match i {
                    0..=19 => ((b & c) | (!b & d), 0x5A827999),
                    20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                    40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                    60..=79 => (b ^ c ^ d, 0xCA62C1D6),
                    _ => panic!("Invalid chunk index"),
                };

                let temp = a
                    .rotate_left(5)
                    .wrapping_add(f)
                    .wrapping_add(e)
                    .wrapping_add(k)
                    .wrapping_add(*tem);

                e = d;
                d = c;
                c = b.rotate_left(30);
                b = a;
                a = temp;
            }

            // Add this chunk's result to the hash
            h0 = h0.wrapping_add(a);
            h1 = h1.wrapping_add(b);
            h2 = h2.wrapping_add(c);
            h3 = h3.wrapping_add(d);
            h4 = h4.wrapping_add(e);
        }

        // Return the final hash
        let mut result: [u8; 20] = [0; 20];
        let h_values = [h0, h1, h2, h3, h4];
        let h_iter = h_values.iter().flat_map(|x| x.to_be_bytes());

        for (ret, src) in result.iter_mut().zip(h_iter) {
            *ret = src;
        }

        result
    }
}
