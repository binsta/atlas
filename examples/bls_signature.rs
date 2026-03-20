use ark_ec::CurveGroup;
use ark_grumpkin::Projective as G;
use atlas_bls::{BlsKeyPair, RelationalBls};
use rand::rngs::StdRng;
use rand::SeedableRng;

type Fq = <G as CurveGroup>::BaseField;

fn main() {
    println!("=== ATLAS Relational BLS Signature (Section 6) ===\n");

    let mut rng = StdRng::seed_from_u64(42);
    let bls = RelationalBls::new(128);
    let keypair = BlsKeyPair::generate(&mut rng);

    // --- Small message ---
    println!("--- Small message signing ---");
    let msg = Fq::from(12345u64);
    let sig = bls.sign(&keypair, msg).expect("signing failed");

    println!("Tweak used : {}", sig.map_witness.tweak);
    println!("sigma      : {:?}", sig.sigma);

    match bls.verify(&keypair.verification_key, msg, &sig) {
        Ok(_) => println!("Signature valid: OK\n"),
        Err(e) => println!("Signature invalid: {}\n", e),
    }

    // --- Wrong message should fail ---
    println!("--- Wrong message (should fail) ---");
    let wrong_msg = Fq::from(99999u64);
    match bls.verify(&keypair.verification_key, wrong_msg, &sig) {
        Ok(_) => println!("Verified (unexpected!)"),
        Err(e) => println!("Rejected: {} OK\n", e),
    }

    // --- Large message (Section 6.4) ---
    println!("--- Large message signing (Section 6.4) ---");
    let block: Vec<u8> = (0u8..=255).collect();
    let (large_sig, witness) = bls
        .sign_large(&keypair, &block)
        .expect("large signing failed");

    println!("Block size : {} bytes", block.len());
    println!("Chunks     : {}", witness.chunks.len());

    match bls.verify_large(&keypair.verification_key, &block, &large_sig, &witness) {
        Ok(_) => println!("Large signature valid: OK\n"),
        Err(e) => println!("Large signature invalid: {}\n", e),
    }

    // --- Multiple signers ---
    println!("--- Multiple signers ---");
    let kp2 = BlsKeyPair::generate(&mut rng);
    let kp3 = BlsKeyPair::generate(&mut rng);
    let msg2 = Fq::from(200u64);
    let msg3 = Fq::from(300u64);

    let sig2 = bls.sign(&kp2, msg2).unwrap();
    let sig3 = bls.sign(&kp3, msg3).unwrap();

    bls.verify(&kp2.verification_key, msg2, &sig2).unwrap();
    bls.verify(&kp3.verification_key, msg3, &sig3).unwrap();
    println!("3 independent signers verified: OK");
}
