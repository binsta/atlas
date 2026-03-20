use ark_ec::CurveGroup;
use ark_ff::Field;
use ark_grumpkin::{Affine as GAffine, Projective as G};

type Fq = <G as CurveGroup>::BaseField;

const CONST: i64 = -17;

pub fn try_make_point(x: Fq) -> Option<GAffine> {
    let x_cube = x.square() * x;
    let y_sq = x_cube + Fq::from(CONST);
    let y_raw = y_sq.sqrt()?;
    let y = canonical_y(y_raw);

    // Only accept if y is actually a QR (has a sqrt)
    y.sqrt()?;

    let pt = GAffine::new_unchecked(x, y);
    if pt.is_on_curve() {
        Some(pt)
    } else {
        None
    }
}

pub fn canonical_y(y: Fq) -> Fq {
    if y.sqrt().is_some() {
        y
    } else {
        -y
    }
}

pub fn is_in_subgroup(pt: &GAffine) -> bool {
    pt.is_on_curve()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_valid_points() {
        let found = (0u64..100)
            .filter(|&i| try_make_point(Fq::from(i)).is_some())
            .count();
        assert!(found > 20);
    }

    #[test]
    fn all_points_on_curve() {
        for i in 0u64..100 {
            if let Some(pt) = try_make_point(Fq::from(i)) {
                assert!(pt.is_on_curve());
            }
        }
    }
}
