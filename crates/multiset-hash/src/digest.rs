use ark_ec::{AffineRepr, CurveGroup};
use ark_grumpkin::{Affine as GAffine, Projective as G};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultisetDigest(pub GAffine);

impl MultisetDigest {
    pub fn empty() -> Self {
        Self(GAffine::identity())
    }

    pub fn is_empty_digest(&self) -> bool {
        use ark_ff::Zero;
        self.0.is_zero()
    }

    pub fn combine(&self, other: &Self) -> Self {
        let sum = self.0.into_group() + other.0.into_group();
        Self(sum.into_affine())
    }

    pub fn remove(&self, other: &Self) -> Self {
        let diff = self.0.into_group() - other.0.into_group();
        Self(diff.into_affine())
    }
}
