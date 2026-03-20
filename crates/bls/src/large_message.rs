use ark_ec::CurveGroup;
use ark_ff::PrimeField;
use ark_grumpkin::Projective as G;
use atlas_core::AtlasError;
use sha2::{Digest, Sha256};

type Fq = <G as CurveGroup>::BaseField;

#[derive(Debug, Clone)]
pub struct LargeMessageWitness {
    pub chunks: Vec<Fq>,
    pub challenge: Fq,
}

pub fn compress(message: &[u8]) -> Result<(Fq, LargeMessageWitness), AtlasError> {
    if message.is_empty() {
        return Err(AtlasError::MessageOutOfRange { max_bits: 0 });
    }

    let chunks: Vec<Fq> = message
        .chunks(15)
        .map(|c| {
            let mut bytes = [0u8; 16];
            bytes[..c.len()].copy_from_slice(c);
            let val = u128::from_le_bytes(bytes);
            let lo = val as u64;
            let hi = (val >> 64) as u64;
            let two64 = Fq::from(u64::MAX) + Fq::from(1u64);
            Fq::from(lo) + Fq::from(hi) * two64
        })
        .collect();

    let challenge = derive_challenge(message);
    let compressed = recompute(&chunks, &challenge)?;

    Ok((compressed, LargeMessageWitness { chunks, challenge }))
}

pub fn recompute(chunks: &[Fq], challenge: &Fq) -> Result<Fq, AtlasError> {
    if chunks.is_empty() {
        return Err(AtlasError::MessageOutOfRange { max_bits: 0 });
    }

    let mut result = chunks[0];
    let mut r_power = *challenge;

    for chunk in &chunks[1..] {
        result += r_power * chunk;
        r_power *= challenge;
    }

    Ok(result)
}

fn derive_challenge(message: &[u8]) -> Fq {
    let hash = Sha256::digest(message);
    Fq::from_le_bytes_mod_order(&hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_recompute_matches() {
        let msg = b"hello this is a large ethereum block header";
        let (compressed, witness) = compress(msg).unwrap();
        let recomputed = recompute(&witness.chunks, &witness.challenge).unwrap();
        assert_eq!(compressed, recomputed);
    }

    #[test]
    fn different_messages_different_compressed() {
        let (c1, _) = compress(b"block1").unwrap();
        let (c2, _) = compress(b"block2").unwrap();
        assert_ne!(c1, c2);
    }

    #[test]
    fn empty_message_fails() {
        assert!(compress(b"").is_err());
    }
}
