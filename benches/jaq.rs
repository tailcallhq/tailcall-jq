use jaq_json::Val;
use serde_json::json;
use criterion::{criterion_group, criterion_main, Criterion};
use tailcall_template::mustache::{Mustache, Segment};
use jaq_core::{load, Ctx, Native, RcIter};
use load::{Arena, File, Loader};

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

pub fn criterion_benchmark(c: &mut Criterion) {
    // BASIC SCENARIO
    {
        let data = json!({"key": "42"});
        let expected = "Value: 42".to_string();
        {
            let mustache = Mustache::from(vec![
                Segment::Literal("Value: ".to_string()),
                Segment::Expression(vec!["key".to_string()]),
            ]);
            c.bench_function("basic_mustache", |b| b.iter(|| bench_mustache(&data, &mustache, &expected)));
        }
        {
            let program = File { code: "\"Value: \" + .key", path: () };

            // start out only from core filters,
            // which do not include filters in the standard library
            // such as `map`, `select` etc.
            let loader = Loader::new([]);
            let arena = Arena::default();

            // parse the filter
            let modules = loader.load(&arena, program).unwrap();

            // compile the filter
            let filter: jaq_core::Filter<Native<Val>> = jaq_core::Compiler::<_, Native<_>>::default()
                .compile(modules)
                .unwrap();

            c.bench_function("basic_jq", |b| b.iter(|| bench_jq(&data, &filter, &expected)));
        }
    }
    // COMPLEX SCENARIO
    {
        let data = json!({"user": "Alice", "age": 30});
        let expected = "User: Alice, Age: 30".to_string();
        {
            let mustache = Mustache::from(vec![
                Segment::Literal("User: ".to_string()),
                Segment::Expression(vec!["user".to_string()]),
                Segment::Literal(", Age: ".to_string()),
                Segment::Expression(vec!["age".to_string()]),
            ]);
            c.bench_function("complex_mustache", |b| b.iter(|| bench_mustache(&data, &mustache, &expected)));
        }
        {
            let program = File { code: "\"User: \" + .user + \", Age: \" + (.age | tostring)", path: () };
            let loader = Loader::new(jaq_std::defs());
            let arena = Arena::default();
            let modules = loader.load(&arena, program).unwrap();
            let filter = jaq_core::Compiler::<_, Native<_>>::default()
                .with_funs(jaq_std::funs())
                .compile(modules)
                .unwrap();

            c.bench_function("complex_jq", |b| b.iter(|| bench_jq(&data, &filter, &expected)));
        }
    }
    // NESTED SCENARIO
    {
        let data = json!({
            "user": {
                "name": "Alice",
                "details": {
                    "age": 30,
                    "location": {
                        "city": "Wonderland",
                        "country": "Fantasy"
                    }
                }
            }
        });
        let expected = "User: Alice, Age: 30, Location: Wonderland, Country: Fantasy".to_string();
        {
            let mustache = Mustache::from(vec![
                Segment::Literal("User: ".to_string()),
                Segment::Expression(vec!["user".to_string(), "name".to_string()]),
                Segment::Literal(", Age: ".to_string()),
                Segment::Expression(vec!["user".to_string(), "details".to_string(), "age".to_string()]),
                Segment::Literal(", Location: ".to_string()),
                Segment::Expression(vec!["user".to_string(), "details".to_string(), "location".to_string(), "city".to_string()]),
                Segment::Literal(", Country: ".to_string()),
                Segment::Expression(vec!["user".to_string(), "details".to_string(), "location".to_string(), "country".to_string()]),
            ]);
            c.bench_function("nested_mustache", |b| b.iter(|| bench_mustache(&data, &mustache, &expected)));
        }
        {
            let program = File { code: "\"User: \" + .user.name + \", Age: \" + (.user.details.age | tostring) + \", Location: \" + .user.details.location.city + \", Country: \" + .user.details.location.country", path: () };
            let loader = Loader::new(jaq_std::defs());
            let arena = Arena::default();
            let modules = loader.load(&arena, program).unwrap();
            let filter = jaq_core::Compiler::<_, Native<_>>::default()
                .with_funs(jaq_std::funs())
                .compile(modules)
                .unwrap();

            c.bench_function("nested_jq", |b| b.iter(|| bench_jq(&data, &filter, &expected)));
        }
    }
}

fn bench_mustache(data: &serde_json::Value, mustache: &Mustache, expected: &str) {
    let result = mustache.render(data);
    assert_eq!(result, expected.to_string());
}

fn bench_jq(data: &serde_json::Value, filter: &jaq_core::Filter<Native<Val>>, expected: &str) {
    let inputs = RcIter::new(core::iter::empty());

    // iterator over the output values
    let mut out = filter.run((Ctx::new([], &inputs), Val::from(data.clone())));

    assert_eq!(out.next(), Some(Ok(Val::from(expected.to_string()))));
    assert_eq!(out.next(), None);
}
