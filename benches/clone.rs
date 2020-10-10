use std::fs::File;

use criterion::{Benchmark, BenchmarkId};
use criterion::{criterion_group, criterion_main, Criterion};
use xwords::{find_fills, Direction, Word, trie::Trie};

pub fn criterion_benchmark(c: &mut Criterion) {
 
    let file = File::open("wordlist.json").unwrap();

    let json: serde_json::Value =
        serde_json::from_reader(file).expect("JSON was not well-formatted");

    let mut words: Vec<String> = match json.as_object() {
        Some(obj) => {
            obj.keys().map(|key| key.to_owned()).collect()
        }
        None => panic!("Failed to load words"),
    };

    let mut subVec: Vec<String> = vec![];

    for i in 0..10_000 {
        subVec.push(words.pop().unwrap());
    }

    let node = Trie::build(subVec).root;


    c.bench(
        "clone_trie",
        Benchmark::new("routine_1", move |b| {
            b.iter(|| node.clone());
        })
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
