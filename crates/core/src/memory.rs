use crate::field::u128_to_fq;
use ark_bn254::Fq;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoryRecord {
    pub address: u32,
    pub value: u32,
    pub is_write: bool,
    pub timestamp: u32,
}

impl MemoryRecord {
    pub fn new(address: u32, value: u32, is_write: bool, timestamp: u32) -> Self {
        Self {
            address,
            value,
            is_write,
            timestamp,
        }
    }

    /// Pack into u128 (97 bits used)
    /// layout: [address:32 | value:32 | is_write:1 | timestamp:32]
    pub fn pack(&self) -> u128 {
        (self.address as u128)
            | ((self.value as u128) << 32)
            | ((self.is_write as u128) << 64)
            | ((self.timestamp as u128) << 65)
    }

    pub fn to_fq(&self) -> Fq {
        u128_to_fq(self.pack())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_distinguishes_rw_flag() {
        let w = MemoryRecord::new(1, 1, true, 0);
        let r = MemoryRecord::new(1, 1, false, 0);
        assert_ne!(w.pack(), r.pack());
    }

    #[test]
    fn pack_distinguishes_timestamp() {
        let a = MemoryRecord::new(0, 0, false, 10);
        let b = MemoryRecord::new(0, 0, false, 11);
        assert_ne!(a.pack(), b.pack());
    }
}
