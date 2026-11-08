//! FNV-1a 64-bit hashing for state fingerprints.
//!
//! The spec names FNV-1a or Blake3. FNV-1a is used here because it needs no
//! dependency and is fast enough for regression fingerprints. Blake3 can
//! replace it later without changing callers, since they only use `Fnv1a`.

const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fnv1a {
    state: u64,
}

impl Default for Fnv1a {
    fn default() -> Self {
        Self::new()
    }
}

impl Fnv1a {
    #[inline]
    pub const fn new() -> Self {
        Self { state: OFFSET_BASIS }
    }

    #[inline]
    pub fn write_u8(&mut self, b: u8) {
        self.state ^= b as u64;
        self.state = self.state.wrapping_mul(PRIME);
    }

    #[inline]
    pub fn write_u16(&mut self, v: u16) {
        for b in v.to_le_bytes() {
            self.write_u8(b);
        }
    }

    #[inline]
    pub fn write_u32(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.write_u8(b);
        }
    }

    #[inline]
    pub fn write_u64(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.write_u8(b);
        }
    }

    #[inline]
    pub const fn finish(self) -> u64 {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_offset_basis() {
        assert_eq!(Fnv1a::new().finish(), 0xcbf2_9ce4_8422_2325);
    }

    #[test]
    fn known_vector_single_byte_a() {
        // Published FNV-1a 64 test vector for "a".
        let mut h = Fnv1a::new();
        h.write_u8(b'a');
        assert_eq!(h.finish(), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn order_sensitive() {
        let mut a = Fnv1a::new();
        a.write_u32(1);
        a.write_u32(2);
        let mut b = Fnv1a::new();
        b.write_u32(2);
        b.write_u32(1);
        assert_ne!(a.finish(), b.finish());
    }
}
