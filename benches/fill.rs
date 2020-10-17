use xwords::default_word_list;
use criterion::BenchmarkId;
use criterion::{criterion_group, criterion_main, Criterion};
use xwords::{find_fills, Direction, Word};

pub fn criterion_benchmark(c: &mut Criterion) {
    let input = Word::new(String::from("R  "), 0, 0, 3, Direction::Across);

    let trie = default_word_list();

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
