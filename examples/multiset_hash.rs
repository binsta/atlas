use ark_ec::CurveGroup;
use ark_grumpkin::Projective as G;
use atlas_multiset_hash::MultisetHash;

type Fq = <G as CurveGroup>::BaseField;

fn main() {
    println!("=== ATLAS Multiset Hash (Section 5) ===\n");

    let hasher = MultisetHash::new(256);

    // --- Basic hashing ---
    println!("--- Single element ---");
    let msg = Fq::from(42u64);
    let (digest, witness) = hasher.hash_multiset(&[msg]).unwrap();
    hasher.verify_multiset(&[msg], &digest, &witness).unwrap();
    println!("hash({{42}}) verified: OK\n");

    // --- Incrementality (Theorem 2) ---
    println!("--- Incrementality: MSH(S1 ∪ S2) = MSH(S1) + MSH(S2) ---");
    let s1 = [Fq::from(1u64), Fq::from(2u64)];
    let s2 = [Fq::from(3u64), Fq::from(4u64)];
    let all: Vec<Fq> = s1.iter().chain(s2.iter()).copied().collect();

    let (d1, _) = hasher.hash_multiset(&s1).unwrap();
    let (d2, _) = hasher.hash_multiset(&s2).unwrap();
    let (da, _) = hasher.hash_multiset(&all).unwrap();

    assert_eq!(d1.combine(&d2), da);
    println!("Incrementality holds: OK\n");

    // --- Order independence ---
    println!("--- Order independence: {{1,2,3}} == {{3,1,2}} ---");
    let (d_abc, _) = hasher
        .hash_multiset(&[Fq::from(1u64), Fq::from(2u64), Fq::from(3u64)])
        .unwrap();
    let (d_cab, _) = hasher
        .hash_multiset(&[Fq::from(3u64), Fq::from(1u64), Fq::from(2u64)])
        .unwrap();
    assert_eq!(d_abc, d_cab);
    println!("Order independence holds: OK\n");

    // --- Collision resistance (different multisets → different digests) ---
    println!("--- Collision resistance ---");
    let (d_12, _) = hasher
        .hash_multiset(&[Fq::from(1u64), Fq::from(2u64)])
        .unwrap();
    let (d_13, _) = hasher
        .hash_multiset(&[Fq::from(1u64), Fq::from(3u64)])
        .unwrap();
    assert_ne!(d_12, d_13);
    println!("{{1,2}} != {{1,3}}: OK\n");

    // --- zkVM memory records ---
    println!("--- zkVM memory records (Section 5.4) ---");
    let records: Vec<Fq> = (0u64..10)
        .map(|i| {
            let address = (i * 4) as u32;
            let value = (i * 100) as u32;
            let is_write = i % 2 == 0;
            let timestamp = i as u32;
            let v: u128 = (address as u128)
                | ((value as u128) << 32)
                | ((is_write as u128) << 64)
                | ((timestamp as u128) << 65);
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let two64 = Fq::from(u64::MAX) + Fq::from(1u64);
            Fq::from(lo) + Fq::from(hi) * two64
        })
        .collect();

    let (digest, witness) = hasher.hash_multiset(&records).unwrap();
    hasher.verify_multiset(&records, &digest, &witness).unwrap();
    println!("10 memory records hashed and verified: OK\n");

    // --- Security parameters ---
    println!("--- Security parameters (Theorem 3) ---");
    println!("|M|        : <= 2^100 (97-bit zkVM records)");
    println!("T          : 256");
    println!("p          : ~2^254 (Grumpkin order)");
    println!("Security   : >120 bits");
}
