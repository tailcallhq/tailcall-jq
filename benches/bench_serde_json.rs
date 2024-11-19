use criterion::Criterion;

pub fn bench_serde_json(c: &mut Criterion) {
    let input = r#"
    {
        "name": "John",
        "age": 30,
        "is_student": false,
        "scores": [85.5, 90, 78],
        "address": null
    }
    "#;
    c.bench_function("serde_json", move |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(input).unwrap())
    });
}
