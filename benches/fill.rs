use criterion::{black_box, criterion_group, criterion_main, Criterion};
use xwords::fill_crossword;
use xwords::Crossword;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
        b.iter(|| {
            let c = Crossword::new(String::from(
                "**   ***     *                     *     ***   **",
            ))
            .unwrap();
            fill_crossword(&c);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
