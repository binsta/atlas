use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum AtlasError {
    #[error("map-to-curve failed after {attempts} attempts (tweak bound T={tweak_bound})")]
    MapToCurveFailed { attempts: u32, tweak_bound: u32 },

    #[error("invalid witness: {reason}")]
    InvalidWitness { reason: &'static str },

    #[error("message out of range (max {max_bits} bits)")]
    MessageOutOfRange { max_bits: u32 },

    #[error("BLS error: {reason}")]
    BlsError { reason: &'static str },

    #[error("memory inconsistency: read digest != write digest")]
    MemoryInconsistent,
}
