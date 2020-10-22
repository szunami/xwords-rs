use std::sync::Arc;
use xwords::crossword::Crossword;
use xwords::default_words;
use xwords::fill::fill_crossword;

use criterion::Benchmark;
use criterion::{criterion_group, criterion_main, Criterion};
use xwords::index_words;

pub fn criterion_benchmark(c: &mut Criterion) {
    let (bigrams, trie) = index_words(default_words());

    let bigrams = Arc::new(bigrams);
    let trie = Arc::new(trie);

    let tmp_bigrams = bigrams.clone();
    let tmp_trie = trie.clone();

    let input = Crossword::new(String::from("         ")).unwrap();
    c.bench(
        "fill_crosswords",
        Benchmark::new("fill_3x3_crossword", move |b| {
            b.iter(|| fill_crossword(&input, tmp_trie.clone(), tmp_bigrams.clone()));
        }),
    );

    let input = Crossword::new(String::from("                ")).unwrap();
    let tmp_bigrams = bigrams.clone();
    let tmp_trie = trie.clone();

    c.bench(
        "fill_crosswords",
        Benchmark::new("fill_4x4_crossword", move |b| {
            b.iter(|| fill_crossword(&input, tmp_trie.clone(), tmp_bigrams.clone()));
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
