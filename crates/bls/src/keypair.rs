use ark_bn254::{Fr, G2Affine, G2Projective};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::UniformRand;
use rand::Rng;

/// BLS key pair using BN254 G2 for verification key.
/// sk ∈ Fr,  vk = sk · g2 ∈ G2
#[derive(Debug, Clone)]
pub struct BlsKeyPair {
    pub secret_key: Fr,
    pub verification_key: G2Affine,
}

impl BlsKeyPair {
    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        let sk = Fr::rand(rng);
        let vk = (G2Affine::generator().into_group() * sk).into_affine();
        Self {
            secret_key: sk,
            verification_key: vk,
        }
    }
}
