use criterion::black_box;
use std::sync::Arc;
use xwords::trie::Trie;

use criterion::{criterion_group, criterion_main, Benchmark, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let group_id = "trie";

    let trie = Trie::load_default().expect("Failed to load trie");

    let trie = Arc::new(trie);

    let tmp_trie = trie.clone();

    c.bench(
        group_id,
        Benchmark::new("empty_word", move |b| {
            b.iter(|| {
                let input = "     ".chars();
                assert!(tmp_trie.words(black_box(input)).len() > 0);
            });
        }),
    );

    let tmp_trie = trie.clone();

    c.bench(
        group_id,
        Benchmark::new("partial_word", move |b| {
            b.iter(|| {
                let input = " E R ".chars();
                assert!(tmp_trie.words(black_box(input)).len() > 0);
            });
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
