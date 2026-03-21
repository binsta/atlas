//! Witness generator for ATLAS circom circuits.
//!
//! Generates input.json for the circom circuits using ATLAS.
//!
//! Usage:
//!   cargo run --example generate_witness
//!
//! Output:
//!   circom/input.json  — feed into circom witness generation

use ark_grumpkin::Projective as G;
use ark_ec::CurveGroup;
use ark_ff::{BigInteger, PrimeField};
use atlas_map_to_curve::{IncrementAndCheck, MapToCurveRelation};
use std::fs;

type Fq = <G as CurveGroup>::BaseField;

/// Convert a field element to decimal string for circom
fn fq_to_str(f: &Fq) -> String {
    let bytes = f.into_bigint().to_bytes_le();
    // Convert little-endian bytes to big integer decimal string
    let mut n = num_bigint::BigUint::from_bytes_le(&bytes);
    n.to_string()
}

fn main() {
    println!("=== ATLAS Circom Witness Generator ===\n");

    let mapper = IncrementAndCheck::new(256);
    let n      = 10usize;

    // Generate N memory records
    // In a real zkVM these would be actual memory access logs
    let messages: Vec<Fq> = (1u64..=n as u64).map(|i| {
        // Pack a fake memory record: addr=i*4, val=i*100, write, ts=i
        let addr     = (i * 4) as u128;
        let val      = (i * 100) as u128;
        let is_write = 1u128;
        let ts       = i as u128;
        let packed   = addr
            | (val      << 32)
            | (is_write << 64)
            | (ts       << 65);
        let lo    = packed as u64;
        let hi    = (packed >> 64) as u64;
        let two64 = Fq::from(u64::MAX) + Fq::from(1u64);
        Fq::from(lo) + Fq::from(hi) * two64
    }).collect();

    // Map each message to a Grumpkin point
    let mut ms_str  = Vec::new();
    let mut xs_str  = Vec::new();
    let mut ys_str  = Vec::new();
    let mut zs_str  = Vec::new();
    let mut ks_str  = Vec::new();

    println!("Generating witnesses for {} memory records...\n", n);

    for (i, &msg) in messages.iter().enumerate() {
        let (point, witness) = mapper.map(msg).expect("map failed");

        println!(
            "Record {}: tweak k={}, x={}, y={}",
            i,
            witness.tweak,
            fq_to_str(&point.x),
            fq_to_str(&point.y),
        );

        ms_str.push(fq_to_str(&msg));
        xs_str.push(fq_to_str(&point.x));
        ys_str.push(fq_to_str(&point.y));
        zs_str.push(fq_to_str(&witness.sqrt_witness));
        ks_str.push(witness.tweak.to_string());
    }

    // Build input.json for circom
    let json = format!(
        r#"{{
    "messages": [{messages}],
    "xs":       [{xs}],
    "ys":       [{ys}],
    "zs":       [{zs}],
    "ks":       [{ks}]
}}"#,
        messages = ms_str.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "),
        xs       = xs_str.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "),
        ys       = ys_str.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "),
        zs       = zs_str.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "),
        ks       = ks_str.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "),
    );

    fs::write("circom/input.json", &json).expect("failed to write input.json");

    println!("\n✓ Written to circom/input.json");
    println!("\nNext steps:");
    println!("  cd circom");
    println!("  circom main.circom --r1cs --wasm --sym");
    println!("  node main_js/generate_witness.js main_js/main.wasm input.json witness.wtns");
    println!("  snarkjs groth16 setup main.r1cs pot12_final.ptau main_0000.zkey");
    println!("  snarkjs groth16 prove main_0000.zkey witness.wtns proof.json public.json");
    println!("  snarkjs groth16 verify verification_key.json public.json proof.json");
}