use ark_ec::CurveGroup;
use ark_grumpkin::Projective as G;
use atlas_multiset_hash::RollingMemoryDigest;

type Fq = <G as CurveGroup>::BaseField;

fn pack_record(address: u32, value: u32, is_write: bool, timestamp: u32) -> Fq {
    let v: u128 = (address as u128)
        | ((value as u128) << 32)
        | ((is_write as u128) << 64)
        | ((timestamp as u128) << 65);
    let lo = v as u64;
    let hi = (v >> 64) as u64;
    let two64 = Fq::from(u64::MAX) + Fq::from(1u64);
    Fq::from(lo) + Fq::from(hi) * two64
}

fn main() {
    println!("=== ATLAS zkVM Memory Consistency (Section 5) ===\n");

    let mut rolling = RollingMemoryDigest::new();

    // Simulate a simple RISC-V program:
    // ts=1  WRITE addr=0x100 val=42
    // ts=2  WRITE addr=0x200 val=99
    // ts=3  READ  addr=0x100 val=42
    // ts=4  WRITE addr=0x100 val=77
    // ts=5  READ  addr=0x200 val=99
    // ts=6  READ  addr=0x100 val=77

    let ops: Vec<(u32, u32, bool, u32)> = vec![
        (0x100, 42, true, 1),
        (0x200, 99, true, 2),
        (0x100, 42, false, 3),
        (0x100, 77, true, 4),
        (0x200, 99, false, 5),
        (0x100, 77, false, 6),
    ];

    println!("--- Recording memory operations ---");
    for &(addr, val, is_write, ts) in &ops {
        let label = if is_write { "WRITE" } else { "READ " };
        println!("ts={} {} addr=0x{:03X} val={}", ts, label, addr, val);
        let fq = pack_record(addr, val, is_write, ts);
        rolling.record_write(fq).unwrap();
        rolling.record_read(fq).unwrap();
    }

    println!("\n--- Consistency check ---");
    match rolling.assert_consistent() {
        Ok(_) => println!("Memory consistent: OK"),
        Err(e) => println!("Memory inconsistent: {}", e),
    }

    // Tampered read — should fail
    println!("\n--- Tampered read (should fail) ---");
    let mut bad = RollingMemoryDigest::new();
    bad.record_write(pack_record(0x100, 42, true, 1)).unwrap();
    bad.record_read(pack_record(0x100, 99, false, 1)).unwrap(); // wrong value!

    match bad.assert_consistent() {
        Ok(_) => println!("Consistent (unexpected!)"),
        Err(e) => println!("Inconsistency detected: {} OK", e),
    }
}
