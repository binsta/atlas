use ark_ec::CurveGroup;
use ark_grumpkin::Projective as G;
use atlas_map_to_curve::{IncrementAndCheck, MapToCurveRelation};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

type Fq = <G as CurveGroup>::BaseField;

fn bench_single_map(c: &mut Criterion) {
    let mapper = IncrementAndCheck::new(256);
    let msg = Fq::from(12345u64);

    c.bench_function("map_single", |b| {
        b.iter(|| mapper.map(black_box(msg)).unwrap())
    });
}

fn bench_single_verify(c: &mut Criterion) {
    let mapper = IncrementAndCheck::new(256);
    let msg = Fq::from(42u64);
    let (pt, wit) = mapper.map(msg).unwrap();

    c.bench_function("verify_single", |b| {
        b.iter(|| {
            mapper
                .verify(black_box(msg), black_box(pt), black_box(&wit))
                .unwrap()
        })
    });
}

fn bench_map_batch(c: &mut Criterion) {
    let mapper = IncrementAndCheck::new(256);
    let mut group = c.benchmark_group("map_batch");

    for log2_n in [8u32, 10, 12, 14] {
        let n = 1usize << log2_n;
        let messages: Vec<Fq> = (0u64..n as u64).map(Fq::from).collect();

        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("2^{log2_n}")),
            &messages,
            |b, msgs| {
                b.iter(|| {
                    for &m in msgs {
                        let _ = mapper.map(black_box(m));
                    }
                })
            },
        );
    }
    group.finish();
}

fn bench_tweak_bounds(c: &mut Criterion) {
    let mut group = c.benchmark_group("tweak_bound");
    let msg = Fq::from(999u64);

    for t in [64u64, 128, 256, 512] {
        let mapper = IncrementAndCheck::new(t);
        group.bench_with_input(BenchmarkId::from_parameter(format!("T={t}")), &t, |b, _| {
            b.iter(|| mapper.map(black_box(msg)).unwrap())
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_single_map,
    bench_single_verify,
    bench_map_batch,
    bench_tweak_bounds,
);
criterion_main!(benches);
