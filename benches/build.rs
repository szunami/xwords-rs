use std::fs::File;

use criterion::BenchmarkId;
use criterion::{criterion_group, criterion_main, Criterion};
use xwords::trie::Trie;

pub fn criterion_benchmark(c: &mut Criterion) {
    let file = File::open("wordlist.json").unwrap();

    let json: serde_json::Value =
        serde_json::from_reader(file).expect("JSON was not well-formatted");

    let mut words: Vec<String> = match json.as_object() {
        Some(obj) => obj.keys().into_iter().cloned().collect(),
        None => panic!("Failed to load words"),
    };

    let mut sub_vec: Vec<String> = vec![];

    for _ in 0..10_000 {
        sub_vec.push(words.pop().unwrap());
    }

    c.bench_with_input(
        BenchmarkId::new("build_trie", 123),
        &sub_vec.clone(),
        |b, s| {
            b.iter(|| Trie::build(s.to_owned()));
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
