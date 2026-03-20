use ark_ec::CurveGroup;
use ark_ff::Field;
use ark_grumpkin::{Affine as GAffine, Projective as G};

use crate::{subgroup, MapToCurveRelation};
use atlas_core::AtlasError;

type Fq = <G as CurveGroup>::BaseField;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapToCurveWitness {
    pub tweak: u64,
    pub sqrt_witness: Fq,
    pub x: Fq,
}

#[derive(Debug, Clone)]
pub struct IncrementAndCheck {
    pub tweak_bound: u64,
}

impl IncrementAndCheck {
    pub fn new(tweak_bound: u64) -> Self {
        Self { tweak_bound }
    }

    fn t_fq(&self) -> Fq {
        Fq::from(self.tweak_bound)
    }
}

impl MapToCurveRelation for IncrementAndCheck {
    fn map(&self, message: Fq) -> Result<(GAffine, MapToCurveWitness), AtlasError> {
        let big_t = self.t_fq();

        for little_t in 0..self.tweak_bound {
            let field_t = Fq::from(little_t);
            let x = field_t + message * big_t;

            if let Some(pt) = subgroup::try_make_point(x) {
                // Try to get sqrt of y — skip if canonical y is not a QR
                if let Some(z) = pt.y.sqrt() {
                    let witness = MapToCurveWitness {
                        tweak: little_t,
                        sqrt_witness: z,
                        x,
                    };
                    return Ok((pt, witness));
                }
                // y not a QR — try next tweak
            }
        }

        Err(AtlasError::MapToCurveFailed {
            attempts: self.tweak_bound as u32,
            tweak_bound: self.tweak_bound as u32,
        })
    }

    fn verify(
        &self,
        message: Fq,
        point: GAffine,
        witness: &MapToCurveWitness,
    ) -> Result<(), AtlasError> {
        // cond_1: x == t + m * T
        let expected_x = Fq::from(witness.tweak) + message * self.t_fq();
        if witness.x != expected_x || point.x != expected_x {
            return Err(AtlasError::InvalidWitness {
                reason: "x != t + m*T",
            });
        }

        // cond_2: y == z * z
        if witness.sqrt_witness * witness.sqrt_witness != point.y {
            return Err(AtlasError::InvalidWitness { reason: "z^2 != y" });
        }

        // cond_3: y^2 == x^3 - 17
        let y_sq = point.y * point.y;
        let x_cu = point.x * point.x * point.x;
        let f_x = x_cu + Fq::from(-17i64);
        if y_sq != f_x {
            return Err(AtlasError::InvalidWitness {
                reason: "y^2 != x^3 - 17",
            });
        }

        // point on curve
        if !subgroup::is_in_subgroup(&point) {
            return Err(AtlasError::InvalidWitness {
                reason: "point not on curve",
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MapToCurveRelation;

    fn mapper() -> IncrementAndCheck {
        IncrementAndCheck::new(256)
    }

    #[test]
    fn map_succeeds() {
        for i in 0u64..20 {
            assert!(mapper().map(Fq::from(i)).is_ok(), "failed at m={i}");
        }
    }

    #[test]
    fn verify_roundtrip() {
        let m = mapper();
        for i in 0u64..20 {
            let msg = Fq::from(i);
            let (pt, w) = m.map(msg).unwrap();
            assert!(m.verify(msg, pt, &w).is_ok(), "verify failed at m={i}");
        }
    }

    #[test]
    fn matches_noir_test_vector() {
        // From Noir test: main(7, 1792, y, z, 0)
        // m=7, T=256, t=0 → x = 0 + 7*256 = 1792
        let m = mapper();
        let msg = Fq::from(7u64);
        let (pt, w) = m.map(msg).unwrap();
        assert_eq!(pt.x, Fq::from(1792u64));
        assert_eq!(w.tweak, 0);
        assert!(m.verify(msg, pt, &w).is_ok());
    }

    #[test]
    fn injectivity() {
        let m = mapper();
        let mut seen = std::collections::HashSet::new();
        for i in 0u64..100 {
            let (pt, _) = m.map(Fq::from(i)).unwrap();
            let key = format!("{:?}{:?}", pt.x, pt.y);
            assert!(seen.insert(key), "collision at {i}");
        }
    }

    #[test]
    fn verify_rejects_bad_tweak() {
        let m = mapper();
        let msg = Fq::from(7u64);
        let (pt, mut w) = m.map(msg).unwrap();
        w.tweak += 1;
        assert!(m.verify(msg, pt, &w).is_err());
    }
}
