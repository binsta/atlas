use ark_bn254::{Bn254, G1Affine, G1Projective, G2Affine};
use ark_ec::{pairing::Pairing, AffineRepr, CurveGroup};
use ark_grumpkin::{Affine as GAffine, Projective as G};

use atlas_core::AtlasError;
use atlas_map_to_curve::{IncrementAndCheck, MapToCurveRelation};

use crate::keypair::BlsKeyPair;
use crate::large_message::LargeMessageWitness;
use crate::signature::BlsSignature;

type GrumpkinFq = <G as CurveGroup>::BaseField;

pub struct RelationalBls {
    mapper: IncrementAndCheck,
}

impl RelationalBls {
    pub fn new(tweak_bound: u64) -> Self {
        Self {
            mapper: IncrementAndCheck::new(tweak_bound),
        }
    }

    pub fn sign(&self, key: &BlsKeyPair, message: GrumpkinFq) -> Result<BlsSignature, AtlasError> {
        // Step 1: map message to Grumpkin point (ZK witness)
        let (hash_point, map_witness) = self.mapper.map(message)?;

        // Step 2: use hash_point x-coordinate to derive BN254 G1 point
        // We use the x-coord as a scalar to multiply the BN254 generator
        // This gives a valid BN254 G1 point tied to the Grumpkin witness
        use ark_bn254::Fr as BN254Fr;
        use ark_ff::{BigInteger, PrimeField};
        let x_bytes = hash_point.x.into_bigint().to_bytes_le();
        let scalar = BN254Fr::from_le_bytes_mod_order(&x_bytes);
        let g1 = G1Affine::generator();
        let hm_bn254: G1Affine = (g1.into_group() * scalar).into_affine();

        // Step 3: sigma = sk * hm_bn254
        let sigma = (hm_bn254.into_group() * key.secret_key).into_affine();

        Ok(BlsSignature {
            sigma,
            hash_point,
            map_witness,
            hm_bn254,
        })
    }

    pub fn verify(
        &self,
        vk: &G2Affine,
        message: GrumpkinFq,
        sig: &BlsSignature,
    ) -> Result<(), AtlasError> {
        // 1. verify map-to-curve witness
        self.mapper
            .verify(message, sig.hash_point, &sig.map_witness)?;

        // 2. recompute hm_bn254 from hash_point
        use ark_bn254::Fr as BN254Fr;
        use ark_ff::{BigInteger, PrimeField};
        let x_bytes = sig.hash_point.x.into_bigint().to_bytes_le();
        let scalar = BN254Fr::from_le_bytes_mod_order(&x_bytes);
        let g1 = G1Affine::generator();
        let hm_bn254: G1Affine = (g1.into_group() * scalar).into_affine();

        // 3. pairing check: e(sigma, g2) == e(hm_bn254, vk)
        let g2 = G2Affine::generator();
        let lhs = Bn254::pairing(sig.sigma, g2);
        let rhs = Bn254::pairing(hm_bn254, *vk);

        if lhs != rhs {
            return Err(AtlasError::BlsError {
                reason: "pairing check failed: e(sigma,g2) != e(hm,vk)",
            });
        }

        Ok(())
    }

    pub fn sign_large(
        &self,
        key: &BlsKeyPair,
        message: &[u8],
    ) -> Result<(BlsSignature, LargeMessageWitness), AtlasError> {
        let (compressed, witness) = crate::large_message::compress(message)?;
        let sig = self.sign(key, compressed)?;
        Ok((sig, witness))
    }

    pub fn verify_large(
        &self,
        vk: &G2Affine,
        message: &[u8],
        sig: &BlsSignature,
        witness: &LargeMessageWitness,
    ) -> Result<(), AtlasError> {
        let compressed = crate::large_message::recompute(&witness.chunks, &witness.challenge)?;
        self.verify(vk, compressed, sig)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(42)
    }
    fn bls() -> RelationalBls {
        RelationalBls::new(128)
    }

    #[test]
    fn sign_and_verify() {
        let kp = BlsKeyPair::generate(&mut rng());
        let msg = GrumpkinFq::from(12345u64);
        let sig = bls().sign(&kp, msg).unwrap();
        assert!(bls().verify(&kp.verification_key, msg, &sig).is_ok());
    }

    #[test]
    fn wrong_message_fails() {
        let kp = BlsKeyPair::generate(&mut rng());
        let sig = bls().sign(&kp, GrumpkinFq::from(1u64)).unwrap();
        assert!(bls()
            .verify(&kp.verification_key, GrumpkinFq::from(2u64), &sig)
            .is_err());
    }

    #[test]
    fn wrong_key_fails() {
        let mut r = rng();
        let kp1 = BlsKeyPair::generate(&mut r);
        let kp2 = BlsKeyPair::generate(&mut r);
        let msg = GrumpkinFq::from(1u64);
        let sig = bls().sign(&kp1, msg).unwrap();
        assert!(bls().verify(&kp2.verification_key, msg, &sig).is_err());
    }

    #[test]
    fn deterministic() {
        let kp = BlsKeyPair::generate(&mut rng());
        let msg = GrumpkinFq::from(99u64);
        let s1 = bls().sign(&kp, msg).unwrap();
        let s2 = bls().sign(&kp, msg).unwrap();
        assert_eq!(s1.sigma, s2.sigma);
    }
}
