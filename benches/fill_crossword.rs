use std::sync::Arc;

use criterion::{Benchmark};
use criterion::{criterion_group, criterion_main, Criterion};
use xwords::{Crossword, default_word_list, fill_crossword};

pub fn criterion_benchmark(c: &mut Criterion) {

    let trie = Arc::new(default_word_list());

    let tmp = trie.clone();


    let input = Crossword::new(String::from("         ")).unwrap();
    c.bench(
        "fill_crosswords",
        Benchmark::new("fill_3x3_crossword",
        move |b| {
            b.iter(|| fill_crossword(&input, tmp.clone()));
        })
    );

    let input = Crossword::new(String::from("                ")).unwrap();
    let tmp = trie.clone();

    c.bench(
        "fill_crosswords",
        Benchmark::new("fill_4x4_crossword",
        move |b| {
            b.iter(|| fill_crossword(&input, tmp.clone()));
        })
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
