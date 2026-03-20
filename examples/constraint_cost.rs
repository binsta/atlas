//! Example: Constraint Cost Comparison (Section 7 of the paper)
//!
//! Reproduces Table 2 from the paper showing ATLAS achieves
//! >23x fewer constraints than the best hash-based alternative.

use atlas_circuits::ConstraintEstimate;

fn main() {
    println!("=== ATLAS Constraint Cost Comparison (Section 7) ===\n");

    // Single invocation
    println!("--- Single invocation ---");
    print_row(ConstraintEstimate::atlas(1));
    print_row(ConstraintEstimate::mimc(1));
    print_row(ConstraintEstimate::poseidon(1));
    print_row(ConstraintEstimate::sha256(1));

    // Scale: 2^10 invocations (zkVM with 1024 memory accesses)
    println!("\n--- 2^10 = 1024 invocations (zkVM) ---");
    print_row(ConstraintEstimate::atlas(1024));
    print_row(ConstraintEstimate::mimc(1024));
    print_row(ConstraintEstimate::poseidon(1024));
    print_row(ConstraintEstimate::sha256(1024));

    // Scale: 2^15 invocations (large zkVM execution)
    println!("\n--- 2^15 = 32768 invocations (large zkVM) ---");
    print_row(ConstraintEstimate::atlas(32768));
    print_row(ConstraintEstimate::mimc(32768));
    print_row(ConstraintEstimate::poseidon(32768));
    print_row(ConstraintEstimate::sha256(32768));

    // Proving time estimate (from paper Figure 7)
    println!("\n--- Proving time estimate at 2^15 iterations ---");
    println!("{:<12} {:>20}", "Construction", "Estimated time");
    println!("{}", "-".repeat(35));
    println!("{:<12} {:>20}", "ATLAS", "~7.6 seconds");
    println!("{:<12} {:>20}", "MiMC", ">400 seconds");
    println!("{:<12} {:>20}", "Poseidon", ">400 seconds");
    println!("{:<12} {:>20}", "SHA-256", "not feasible");

    println!("\n--- Security parameters (Section 5.4) ---");
    println!("Message space : |M| <= 2^100 (97-bit zkVM records)");
    println!("Tweak bound   : T = 256");
    println!("Group order   : p ~ 2^254 (BN254)");
    println!("Security      : >120 bits (Theorem 3)");
}

fn print_row(e: ConstraintEstimate) {
    let vs = if e.label == "ATLAS" {
        "baseline".to_string()
    } else {
        format!("{:.1}x more constraints", e.speedup_over_atlas())
    };
    println!("{:<12} {:>18} constraints   {}", e.label, e.total(), vs);
}
