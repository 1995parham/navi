use std::hash::{Hash, Hasher};

/// FNV-1a 64-bit offset basis. Must stay paired with the 64-bit prime below:
/// the 32-bit basis (`0x811C_9DC5`) leaves the top half of the state at zero,
/// which weakens the avalanche for short inputs.
const MAGIC_INIT: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a 64-bit prime.
const PRIME: u64 = 0x0100_0000_01b3;

pub fn fnv<T: Hash>(x: &T) -> u64 {
    let mut hasher = FnvHasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

struct FnvHasher(u64);

impl FnvHasher {
    fn new() -> Self {
        FnvHasher(MAGIC_INIT)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter() {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(PRIME);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fnv_is_stable_and_distinguishes_inputs() {
        assert_eq!(fnv(&"navi"), fnv(&"navi"));
        assert_ne!(fnv(&"navi"), fnv(&"navy"));
        assert_ne!(fnv(&""), 0);
    }

    #[test]
    fn test_fnv_separates_tuple_fields() {
        // Hashing a tuple must not behave like hashing the concatenation, or
        // else neighbouring fields could be shifted without changing the hash.
        assert_ne!(fnv(&("ab", "", "cd")), fnv(&("a", "", "bcd")));
    }
}
