/// Estimated PLONK constraint counts per invocation (Table 2 of the paper).
pub const ATLAS_CONSTRAINTS: u64 = 30;
pub const MIMC_CONSTRAINTS: u64 = 351;
pub const POSEIDON_CONSTRAINTS: u64 = 948;
pub const SHA256_CONSTRAINTS: u64 = 7_095;

#[derive(Debug, Clone)]
pub struct ConstraintEstimate {
    pub label: &'static str,
    pub per_invoke: u64,
    pub num_invokes: u64,
}

impl ConstraintEstimate {
    pub fn total(&self) -> u64 {
        self.per_invoke * self.num_invokes
    }

    pub fn speedup_over_atlas(&self) -> f64 {
        let atlas = ATLAS_CONSTRAINTS * self.num_invokes;
        self.total() as f64 / atlas as f64
    }

    pub fn atlas(n: u64) -> Self {
        Self {
            label: "ATLAS",
            per_invoke: ATLAS_CONSTRAINTS,
            num_invokes: n,
        }
    }
    pub fn mimc(n: u64) -> Self {
        Self {
            label: "MiMC",
            per_invoke: MIMC_CONSTRAINTS,
            num_invokes: n,
        }
    }
    pub fn poseidon(n: u64) -> Self {
        Self {
            label: "Poseidon",
            per_invoke: POSEIDON_CONSTRAINTS,
            num_invokes: n,
        }
    }
    pub fn sha256(n: u64) -> Self {
        Self {
            label: "SHA-256",
            per_invoke: SHA256_CONSTRAINTS,
            num_invokes: n,
        }
    }

    pub fn print_table(n: u64) {
        let rows = [
            Self::atlas(n),
            Self::mimc(n),
            Self::poseidon(n),
            Self::sha256(n),
        ];
        println!(
            "\n{:<12} {:>18} {:>15}",
            "Construction", "Total Constraints", "vs ATLAS"
        );
        println!("{}", "-".repeat(48));
        for r in &rows {
            let vs = if r.label == "ATLAS" {
                "baseline".to_string()
            } else {
                format!("{:.1}x more", r.speedup_over_atlas())
            };
            println!("{:<12} {:>18} {:>15}", r.label, r.total(), vs);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_fewer_than_mimc() {
        assert!(ATLAS_CONSTRAINTS < MIMC_CONSTRAINTS);
    }

    #[test]
    fn atlas_fewer_than_sha256() {
        assert!(ATLAS_CONSTRAINTS < SHA256_CONSTRAINTS);
    }

    #[test]
    fn speedup_over_mimc_at_least_10x() {
        assert!(ConstraintEstimate::mimc(1).speedup_over_atlas() > 10.0);
    }

    #[test]
    fn speedup_over_sha256_at_least_100x() {
        assert!(ConstraintEstimate::sha256(1).speedup_over_atlas() > 100.0);
    }
}
