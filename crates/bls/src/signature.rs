use ark_grumpkin::{Affine as GAffine, Fr as GrumpkinFr};
use atlas_map_to_curve::MapToCurveWitness;

/// BLS signature using Grumpkin-BN254 cycle.
///
/// hm  = MapToGroup(m) ∈ Grumpkin  (map-to-curve witness)
/// σ   = sk · hm ∈ Grumpkin        (signature is a Grumpkin point)
/// vk  = sk · g2 ∈ BN254 G2        (verification key)
///
/// Verification (using cycle relationship):
///   σ and hm are Grumpkin points
///   vk is a BN254 G2 point
///   We verify sk consistency via scalar multiplication check
#[derive(Debug, Clone)]
pub struct BlsSignature {
    /// sigma = sk · hm ∈ Grumpkin
    pub sigma: GAffine,
    /// hm = MapToGroup(m) ∈ Grumpkin
    pub hash_point: GAffine,
    /// sk scalar used for signing (for verification)
    pub sk_scalar: GrumpkinFr,
    /// witness for map-to-curve step
    pub map_witness: MapToCurveWitness,
}
