use ark_bn254::{Fr as BN254Fr, G2Affine};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::UniformRand;
use ark_grumpkin::Fr as GrumpkinFr;
use rand::Rng;

/// BLS key pair using Grumpkin-BN254 cycle.
///
/// Grumpkin scalar field == BN254 base field
/// BN254 scalar field    == Grumpkin base field
///
/// sk ∈ GrumpkinFr  (= BN254 base field)
/// vk = sk · g2 ∈ BN254 G2
#[derive(Debug, Clone)]
pub struct BlsKeyPair {
    /// sk as GrumpkinFr for signing Grumpkin points
    pub secret_key: GrumpkinFr,
    /// sk as BN254Fr for vk in G2
    pub secret_key_bn254: BN254Fr,
    /// vk = sk_bn254 · g2 ∈ BN254 G2
    pub verification_key: G2Affine,
}

impl BlsKeyPair {
    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        // Generate random scalar as bytes, interpret in both fields
        let sk_grumpkin = GrumpkinFr::rand(rng);

        // Convert to BN254Fr via bytes (cycle relationship)
        use ark_ff::{BigInteger, PrimeField};
        let sk_bytes = sk_grumpkin.into_bigint().to_bytes_le();
        let sk_bn254 = BN254Fr::from_le_bytes_mod_order(&sk_bytes);

        let vk = (G2Affine::generator().into_group() * sk_bn254).into_affine();
        Self {
            secret_key: sk_grumpkin,
            secret_key_bn254: sk_bn254,
            verification_key: vk,
        }
    }
}
