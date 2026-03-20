use crate::digest::MultisetDigest;
use crate::hasher::MultisetHash;
use ark_ec::CurveGroup;
use ark_grumpkin::Projective as G;
use atlas_core::AtlasError;

type Fq = <G as CurveGroup>::BaseField;

pub struct RollingMemoryDigest {
    hasher: MultisetHash,
    pub write_d: MultisetDigest,
    pub read_d: MultisetDigest,
}

impl RollingMemoryDigest {
    pub fn new() -> Self {
        Self {
            hasher: MultisetHash::new(256),
            write_d: MultisetDigest::empty(),
            read_d: MultisetDigest::empty(),
        }
    }

    pub fn record_write(&mut self, message: Fq) -> Result<(), AtlasError> {
        let (new_d, _) = self.hasher.update(&self.write_d, message)?;
        self.write_d = new_d;
        Ok(())
    }

    pub fn record_read(&mut self, message: Fq) -> Result<(), AtlasError> {
        let (new_d, _) = self.hasher.update(&self.read_d, message)?;
        self.read_d = new_d;
        Ok(())
    }

    pub fn assert_consistent(&self) -> Result<(), AtlasError> {
        if self.write_d == self.read_d {
            Ok(())
        } else {
            Err(AtlasError::MemoryInconsistent)
        }
    }
}

impl Default for RollingMemoryDigest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_core::MemoryRecord;

    fn to_fq(r: &MemoryRecord) -> Fq {
        use atlas_core::field::u128_to_fq;
        // Grumpkin base field is bn254 scalar field
        // so we need to convert via u128
        let packed = r.pack();
        let lo = packed as u64;
        let hi = (packed >> 64) as u64;
        let two64 = Fq::from(u64::MAX) + Fq::from(1u64);
        Fq::from(lo) + Fq::from(hi) * two64
    }

    #[test]
    fn consistent_logs_pass() {
        let mut r = RollingMemoryDigest::new();
        let msgs: Vec<Fq> = (0u64..10).map(|i| Fq::from(i)).collect();

        for &m in &msgs {
            r.record_write(m).unwrap();
        }
        for &m in &msgs {
            r.record_read(m).unwrap();
        }
        assert!(r.assert_consistent().is_ok());
    }

    #[test]
    fn inconsistent_logs_fail() {
        let mut r = RollingMemoryDigest::new();
        r.record_write(Fq::from(1u64)).unwrap();
        r.record_read(Fq::from(2u64)).unwrap();
        assert!(r.assert_consistent().is_err());
    }

    #[test]
    fn order_does_not_matter() {
        let mut r = RollingMemoryDigest::new();
        let a = Fq::from(10u64);
        let b = Fq::from(20u64);

        r.record_write(a).unwrap();
        r.record_write(b).unwrap();

        r.record_read(b).unwrap();
        r.record_read(a).unwrap();

        assert!(r.assert_consistent().is_ok());
    }
}
