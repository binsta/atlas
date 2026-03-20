use ark_ec::CurveGroup;
use ark_grumpkin::Projective as G;
use atlas_map_to_curve::{IncrementAndCheck, MapToCurveRelation};

type Fq = <G as CurveGroup>::BaseField;

fn main() {
    println!("=== ATLAS Map-to-Curve (Section 4) ===\n");

    let mapper = IncrementAndCheck::new(256);

    let messages = [
        ("hello world", Fq::from(1u64)),
        ("zkVM record", Fq::from(2u64)),
        ("block header", Fq::from(3u64)),
    ];

    for (label, msg) in &messages {
        let (point, witness) = mapper.map(*msg).expect("map failed");
        println!("Message : {}", label);
        println!("Tweak k : {}", witness.tweak);
        println!("Point x : {:?}", point.x);
        println!("Point y : {:?}", point.y);
        mapper.verify(*msg, point, &witness).expect("verify failed");
        println!("Witness : OK\n");
    }

    println!("--- Injectivity check ---");
    let m1 = Fq::from(100u64);
    let m2 = Fq::from(101u64);
    let (p1, _) = mapper.map(m1).unwrap();
    let (p2, _) = mapper.map(m2).unwrap();
    assert_ne!(p1, p2);
    println!("message_1 != message_2 → different points: OK");
}
