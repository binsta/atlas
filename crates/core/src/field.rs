use ark_bn254::Fq;
use ark_ff::{BigInteger, PrimeField};
use sha2::{Digest, Sha256};

pub fn bytes_to_fq(data: &[u8]) -> Fq {
    let hash = Sha256::digest(data);
    Fq::from_le_bytes_mod_order(&hash)
}

pub fn u64_to_fq(v: u64) -> Fq {
    Fq::from(v)
}

pub fn u128_to_fq(v: u128) -> Fq {
    let lo = v as u64;
    let hi = (v >> 64) as u64;
    let two64 = Fq::from(u64::MAX) + Fq::from(1u64);
    Fq::from(lo) + Fq::from(hi) * two64
}

pub fn fq_to_bytes(f: &Fq) -> Vec<u8> {
    f.into_bigint().to_bytes_le()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_to_fq_is_deterministic() {
        assert_eq!(bytes_to_fq(b"hello"), bytes_to_fq(b"hello"));
    }

    #[test]
    fn distinct_inputs_give_distinct_elements() {
        assert_ne!(bytes_to_fq(b"a"), bytes_to_fq(b"b"));
    }
}
