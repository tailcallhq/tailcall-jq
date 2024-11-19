use criterion::Criterion;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "benches/json.pest"] // relative path to the grammar file
struct JSONParser;

pub fn bench_pest(c: &mut Criterion) {
    let input = r#"
    {
        "name": "John",
        "age": 30,
        "is_student": false,
        "scores": [85.5, 90, 78],
        "address": null
    }
    "#;

    c.bench_function("bench_pest", |b| {
        b.iter(|| {
            JSONParser::parse(Rule::json, input)
                .unwrap()
                .next()
                .unwrap()
        })
    });
}
