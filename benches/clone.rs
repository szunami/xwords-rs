use std::fs::File;

use criterion::Benchmark;
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

    let node = Trie::build(sub_vec).root;

    c.bench(
        "clone_trie",
        Benchmark::new("routine_1", move |b| {
            b.iter(|| node.clone());
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
