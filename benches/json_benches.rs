use criterion::{criterion_group, criterion_main, Criterion};

mod bench_pest;
mod bench_serde_json;

mod bench_nom;

fn all_benchmarks(c: &mut Criterion) {
    bench_pest::bench_pest(c);
    bench_serde_json::bench_serde_json(c);
    bench_nom::bench_nom(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = all_benchmarks
}
criterion_main!(benches);
