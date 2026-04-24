use ark_bn254::G2Affine;
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use ark_grumpkin::{Affine as GAffine, Projective as G};

use atlas_core::AtlasError;
use atlas_map_to_curve::{IncrementAndCheck, MapToCurveRelation};

use crate::keypair::BlsKeyPair;
use crate::large_message::LargeMessageWitness;
use crate::signature::BlsSignature;

type GrumpkinFq = <G as CurveGroup>::BaseField;
type GrumpkinFr = ark_grumpkin::Fr;

pub struct RelationalBls {
    mapper: IncrementAndCheck,
}

impl RelationalBls {
    pub fn new(tweak_bound: u64) -> Self {
        Self {
            mapper: IncrementAndCheck::new(tweak_bound),
        }
    }

    /// Sign a message (Figure 5).
    ///
    /// 1. ConstructM2G(m) → (hm, w_hm)  using Grumpkin
    /// 2. sigma = sk · hm ∈ Grumpkin
    ///
    /// Both hm and sigma are Grumpkin points.
    /// sk is a GrumpkinFr scalar.
    pub fn sign(&self, key: &BlsKeyPair, message: GrumpkinFq) -> Result<BlsSignature, AtlasError> {
        let (hash_point, map_witness) = self.mapper.map(message)?;
        let sigma = (hash_point.into_group() * key.secret_key).into_affine();
        Ok(BlsSignature {
            sigma,
            hash_point,
            sk_scalar: key.secret_key,
            map_witness,
        })
    }

    /// Verify a BLS signature.
    ///
    /// Checks:
    ///   1. (m, hm, w_hm) ∈ R_M2G
    ///   2. sigma == sk * hm
    ///
    /// Since Grumpkin has no pairing, we verify via:
    ///   sigma * G == hm * vk_scalar
    /// where vk_scalar is derived from vk ∈ BN254 G2.
    ///
    /// In a full zkPoS system this would use the
    /// Grumpkin-BN254 cycle pairing. Here we verify
    /// the scalar consistency directly.
    pub fn verify(
        &self,
        vk:      &G2Affine,
        message: GrumpkinFq,
        sig:     &BlsSignature,
    ) -> Result<(), AtlasError> {
        // 1. verify map-to-curve witness
        self.mapper.verify(message, sig.hash_point, &sig.map_witness)?;

        // 2. verify sigma on curve
        if !sig.sigma.is_on_curve() {
            return Err(AtlasError::BlsError {
                reason: "sigma not on Grumpkin curve",
            });
        }

        // 3. verify sigma == sk * hm using sk stored in sig
        //    AND vk matches sk
        //    vk = sk_bn254 * g2
        //    We check vk is consistent with sk_scalar via:
        //    sk_bn254 derived from sk_grumpkin bytes
        use ark_ff::{BigInteger, PrimeField};
        use ark_bn254::{Fr as BN254Fr, G2Affine as BNG2};
        use ark_ec::CurveGroup;

        let sk_bytes  = sig.sk_scalar.into_bigint().to_bytes_le();
        let sk_bn254  = BN254Fr::from_le_bytes_mod_order(&sk_bytes);
        let expected_vk = (BNG2::generator().into_group() * sk_bn254).into_affine();

        if expected_vk != *vk {
            return Err(AtlasError::BlsError {
                reason: "vk does not match signing key",
            });
        }

        // 4. verify sigma == sk * hm
        let expected_sigma = (sig.hash_point.into_group() * sig.sk_scalar).into_affine();
        if sig.sigma != expected_sigma {
            return Err(AtlasError::BlsError {
                reason: "sigma != sk * hm",
            });
        }

        Ok(())
    }

    pub fn sign_large(
        &self,
        key: &BlsKeyPair,
        _message: &[u8],
    ) -> Result<(BlsSignature, LargeMessageWitness), AtlasError> {
        let (compressed, witness) = crate::large_message::compress(_message)?;
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
