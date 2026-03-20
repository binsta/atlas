use ark_ec::AdditiveGroup;
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::Zero;
use ark_grumpkin::{Affine as GAffine, Projective as G};
use atlas_core::AtlasError;
use atlas_map_to_curve::{IncrementAndCheck, MapToCurveRelation, MapToCurveWitness};

use crate::digest::MultisetDigest;

type Fq = <G as CurveGroup>::BaseField;

#[derive(Debug, Clone)]
pub struct MultisetElement {
    pub message: Fq,
    pub point: GAffine,
    pub witness: MapToCurveWitness,
}

#[derive(Debug, Clone, Default)]
pub struct MultisetWitness {
    pub elements: Vec<MultisetElement>,
}

pub struct MultisetHash {
    mapper: IncrementAndCheck,
}

impl MultisetHash {
    pub fn new(tweak_bound: u64) -> Self {
        Self {
            mapper: IncrementAndCheck::new(tweak_bound),
        }
    }

    pub fn hash_one(&self, message: Fq) -> Result<(MultisetDigest, MultisetElement), AtlasError> {
        let (pt, w) = self.mapper.map(message)?;
        Ok((
            MultisetDigest(pt),
            MultisetElement {
                message,
                point: pt,
                witness: w,
            },
        ))
    }

    pub fn update(
        &self,
        current: &MultisetDigest,
        message: Fq,
    ) -> Result<(MultisetDigest, MultisetElement), AtlasError> {
        let (contrib, elem) = self.hash_one(message)?;
        Ok((current.combine(&contrib), elem))
    }

    pub fn hash_multiset(
        &self,
        messages: &[Fq],
    ) -> Result<(MultisetDigest, MultisetWitness), AtlasError> {
        let mut digest = MultisetDigest::empty();
        let mut witness = MultisetWitness::default();

        for &m in messages {
            let (d, elem) = self.update(&digest, m)?;
            digest = d;
            witness.elements.push(elem);
        }

        Ok((digest, witness))
    }

    pub fn verify_multiset(
        &self,
        messages: &[Fq],
        digest: &MultisetDigest,
        witness: &MultisetWitness,
    ) -> Result<(), AtlasError> {
        if witness.elements.len() != messages.len() {
            return Err(AtlasError::InvalidWitness {
                reason: "witness length != multiset size",
            });
        }

        let mut acc = G::zero();

        for (i, elem) in witness.elements.iter().enumerate() {
            if elem.message != messages[i] {
                return Err(AtlasError::InvalidWitness {
                    reason: "witness message != messages[i]",
                });
            }
            self.mapper
                .verify(elem.message, elem.point, &elem.witness)?;
            acc += elem.point.into_group();
        }

        let computed = MultisetDigest(acc.into_affine());
        if computed != *digest {
            return Err(AtlasError::InvalidWitness {
                reason: "sum of points != digest",
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h() -> MultisetHash {
        MultisetHash::new(256)
    }

    #[test]
    fn empty_is_identity() {
        let (d, _) = h().hash_multiset(&[]).unwrap();
        assert!(d.is_empty_digest());
    }

    #[test]
    fn incrementality() {
        let h = h();
        let s1 = [Fq::from(1u64), Fq::from(2u64)];
        let s2 = [Fq::from(3u64), Fq::from(4u64)];
        let all: Vec<_> = s1.iter().chain(s2.iter()).copied().collect();

        let (d1, _) = h.hash_multiset(&s1).unwrap();
        let (d2, _) = h.hash_multiset(&s2).unwrap();
        let (da, _) = h.hash_multiset(&all).unwrap();

        assert_eq!(d1.combine(&d2), da);
    }

    #[test]
    fn order_independence() {
        let h = h();
        let a = Fq::from(10u64);
        let b = Fq::from(20u64);
        let (dab, _) = h.hash_multiset(&[a, b]).unwrap();
        let (dba, _) = h.hash_multiset(&[b, a]).unwrap();
        assert_eq!(dab, dba);
    }

    #[test]
    fn verify_roundtrip() {
        let h = h();
        let msgs = [Fq::from(1u64), Fq::from(2u64), Fq::from(3u64)];
        let (d, w) = h.hash_multiset(&msgs).unwrap();
        assert!(h.verify_multiset(&msgs, &d, &w).is_ok());
    }

    #[test]
    fn verify_rejects_wrong_digest() {
        let h = h();
        let msgs = [Fq::from(1u64)];
        let (_, w) = h.hash_multiset(&msgs).unwrap();
        assert!(h
            .verify_multiset(&msgs, &MultisetDigest::empty(), &w)
            .is_err());
    }
}
