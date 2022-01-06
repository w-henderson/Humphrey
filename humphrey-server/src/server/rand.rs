//! Provides a simple LCG implementation for generating random numbers.

use std::time::SystemTime;

/// Represents a linear congruential generator, used to generate random `u32` numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lcg {
    modulus: usize,
    multiplier: usize,
    increment: usize,
    seed: usize,
}

/// Allows random sampling on an object.
pub trait Choose {
    /// The type of the object.
    type Item;

    /// Selects a random item from the collection.
    fn choose(&self, lcg: &mut Lcg) -> Option<&Self::Item>;
}

impl Lcg {
    /// Creates a new linear congruential generator with default parameters.
    /// Default parameters are taken from [glibc](https://sourceware.org/git/?p=glibc.git;a=blob;f=stdlib/random_r.c;hb=glibc-2.26#l362).
    /// Modulus has been reduced by one so as to not be a direct power of two, reducing patterns.
    pub fn new() -> Self {
        Lcg {
            modulus: 2_usize.pow(31) - 1,
            multiplier: 1103515245,
            increment: 12345,
            seed: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize,
        }
    }

    /// Creates a new linear congruential generator with the specified parameters.
    #[allow(dead_code)]
    pub fn with_parameters(
        modulus: usize,
        multiplier: usize,
        increment: usize,
        seed: usize,
    ) -> Self {
        Lcg {
            modulus,
            multiplier,
            increment,
            seed,
        }
    }
}

impl Default for Lcg {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for Lcg {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let value = (self.multiplier * self.seed + self.increment) % self.modulus;
        self.seed = value;

        Some(value as u32)
    }
}

impl<T> Choose for [T] {
    type Item = T;

    fn choose(&self, lcg: &mut Lcg) -> Option<&Self::Item> {
        let value = lcg.next().unwrap();
        self.get((value % self.len() as u32) as usize)
    }
}
