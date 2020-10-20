use xwords::{crossword::Direction, Word};
use xwords::fill::find_fills;
use xwords::index_words;
use xwords::default_words;
use criterion::BenchmarkId;
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let input = Word::new(String::from("R  "), 0, 0, 3, Direction::Across);

    let (_, trie) = index_words(default_words());

    c.bench_with_input(
        BenchmarkId::new("find_fills", &input),
        &input.clone(),
        |b, s| {
            b.iter(|| find_fills(s.clone(), &trie));
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
