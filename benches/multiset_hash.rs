use ark_ec::CurveGroup;
use ark_grumpkin::Projective as G;
use atlas_multiset_hash::{MultisetHash, RollingMemoryDigest};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

type Fq = <G as CurveGroup>::BaseField;

fn make_messages(n: usize) -> Vec<Fq> {
    (0u64..n as u64).map(Fq::from).collect()
}

fn bench_hash_single(c: &mut Criterion) {
    let hasher = MultisetHash::new(256);
    let msg = Fq::from(42u64);

    c.bench_function("multiset_hash_single", |b| {
        b.iter(|| hasher.hash_one(black_box(msg)).unwrap())
    });
}

fn bench_hash_batch(c: &mut Criterion) {
    let hasher = MultisetHash::new(256);
    let mut group = c.benchmark_group("multiset_hash_batch");

    for log2_n in [6u32, 8, 10, 12] {
        let n = 1usize << log2_n;
        let messages = make_messages(n);

        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("2^{log2_n}")),
            &messages,
            |b, msgs| b.iter(|| hasher.hash_multiset(black_box(msgs)).unwrap()),
        );
    }
    group.finish();
}

fn bench_rolling_digest(c: &mut Criterion) {
    let mut group = c.benchmark_group("rolling_digest");

    for log2_n in [6u32, 8, 10, 12] {
        let n = 1usize << log2_n;
        let messages = make_messages(n);

        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("2^{log2_n}")),
            &messages,
            |b, msgs| {
                b.iter(|| {
                    let mut r = RollingMemoryDigest::new();
                    for &m in msgs {
                        r.record_write(black_box(m)).unwrap();
                    }
                    for &m in msgs {
                        r.record_read(black_box(m)).unwrap();
                    }
                    r.assert_consistent().unwrap();
                })
            },
        );
    }
    group.finish();
}

fn bench_verify_multiset(c: &mut Criterion) {
    let hasher = MultisetHash::new(256);
    let mut group = c.benchmark_group("verify_multiset");

    for log2_n in [6u32, 8, 10] {
        let n = 1usize << log2_n;
        let messages = make_messages(n);
        let (digest, witness) = hasher.hash_multiset(&messages).unwrap();

        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("2^{log2_n}")),
            &(&messages, &digest, &witness),
            |b, (msgs, d, w)| {
                b.iter(|| {
                    hasher
                        .verify_multiset(black_box(msgs), black_box(d), black_box(w))
                        .unwrap()
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_hash_single,
    bench_hash_batch,
    bench_rolling_digest,
    bench_verify_multiset,
);
criterion_main!(benches);
