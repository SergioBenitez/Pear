use pear::input::Result;
use pear::macros::parse;

use json::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[inline(always)]
fn parse_json<'a, I: Input<'a>>(mut input: I) -> Result<JsonValue<'a>, I> {
    let result = parse!(value: &mut input);
    assert!(result.is_ok());
    result
}

// #[bench]
// fn canada(b: &mut Bencher) {
//     let data = include_str!("../assets/canada.json");
//     b.iter(|| parse_json(data));
// }

// This is the benchmark from PEST. Unfortunately, our parser here is fully
// fleshed out: it actually creates the `value`, while the PEST one just checks
// if it parses. As a result, our parser will be much slower. You can immitate
// the PEST parser's behavior by changing the parser so that it doesn't build
// real values and instead returns dummy values.
pub fn simple_data(c: &mut Criterion) {
    let data = include_str!("../assets/simple.json");
    c.bench_function("simple", |b| b.iter(|| black_box(parse_json(data))));
}

criterion_group!(json, simple_data);
criterion_main!(json);
