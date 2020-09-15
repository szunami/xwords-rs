use criterion::{criterion_group, criterion_main, Criterion};
use xwords::{find_fills, Word, Direction};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("find_fills", |b| {
        b.iter(|| {
            find_fills(Word::new(String::from("R  "), 0, 0, 3, Direction::Across))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
