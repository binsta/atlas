use ark_bn254::G1Affine;
use ark_grumpkin::Affine as GAffine;
use atlas_map_to_curve::MapToCurveWitness;

#[derive(Debug, Clone)]
pub struct BlsSignature {
    /// sigma = sk · hm_bn254 ∈ BN254 G1
    pub sigma: G1Affine,
    /// hm = MapToGroup(m) ∈ Grumpkin (ZK witness)
    pub hash_point: GAffine,
    /// hm lifted to BN254 G1 for pairing
    pub hm_bn254: G1Affine,
    /// witness for map-to-curve step
    pub map_witness: MapToCurveWitness,
}
