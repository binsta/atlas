pub mod relation;
pub mod subgroup;

pub use relation::{IncrementAndCheck, MapToCurveWitness};
pub use subgroup::try_make_point;

use ark_ec::CurveGroup;
use ark_grumpkin::{Affine as GAffine, Projective as G};

type Fq = <G as CurveGroup>::BaseField;

pub trait MapToCurveRelation {
    fn map(&self, message: Fq) -> Result<(GAffine, MapToCurveWitness), atlas_core::AtlasError>;

    fn verify(
        &self,
        message: Fq,
        point: GAffine,
        witness: &MapToCurveWitness,
    ) -> Result<(), atlas_core::AtlasError>;
}
