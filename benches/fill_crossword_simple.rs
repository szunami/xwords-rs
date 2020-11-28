use criterion::black_box;
use std::sync::Arc;
use xwords::{
    crossword::Crossword,
    fill::{simple::SimpleFiller, Filler},
    trie::Trie,
};

use criterion::{criterion_group, criterion_main, Benchmark, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let group_id = "simple_filler";

    let trie = Trie::load_default().expect("Failed to load trie");

    let trie = Arc::new(trie);

    let tmp_trie = trie.clone();

    c.bench(
        group_id,
        Benchmark::new("empty_20201005_crossword", move |b| {
            let mut filler = SimpleFiller::new(tmp_trie.as_ref());

            let input = std::fs::read_to_string("./grids/20201012_empty.txt")
                .expect("failed to read input");
            let input = Crossword::square(input).expect("failed to parse input");

            b.iter(|| {
                assert!(filler.fill(black_box(&input)).is_ok());
            });
        }),
    );

    let tmp_trie = trie.clone();
    c.bench(
        group_id,
        Benchmark::new("empty_20201012_crossword", move |b| {
            let input = std::fs::read_to_string("./grids/20201012_empty.txt")
                .expect("failed to read input");
            let input = Crossword::square(input).expect("failed to parse input");
            let mut filler = SimpleFiller::new(tmp_trie.as_ref());
            b.iter(|| {
                assert!(filler.fill(black_box(&input)).is_ok());
            });
        }),
    );

    let tmp_trie = trie.clone();

    c.bench(
        group_id,
        Benchmark::new("empty_20201107_crossword", move |b| {
            let mut filler = SimpleFiller::new(tmp_trie.as_ref());
            let input = std::fs::read_to_string("./grids/20201107_empty.txt")
                .expect("failed to read input");
            let input = Crossword::square(input).expect("failed to parse input");

            b.iter(|| {
                assert!(filler.fill(black_box(&input)).is_ok());
            });
        }),
    );

    let tmp_trie = trie.clone();

    c.bench(
        group_id,
        Benchmark::new("empty_20201128_crossword", move |b| {
            let mut filler = SimpleFiller::new(tmp_trie.as_ref());
            let input = std::fs::read_to_string("./grids/20201128_empty.txt")
                .expect("failed to read input");
            let input = Crossword::square(input).expect("failed to parse input");

            b.iter(|| {
                assert!(filler.fill(black_box(&input)).is_ok());
            });
        }),
    );
    
    let tmp_trie = trie.clone();

    c.bench(
        group_id,
        Benchmark::new("empty_20201124_crossword", move |b| {
            let mut filler = SimpleFiller::new(tmp_trie.as_ref());
            let input = std::fs::read_to_string("./grids/20201124_empty.txt")
                .expect("failed to read input");
            let input = Crossword::rectangle(input, 16, 15).expect("failed to parse input");

            b.iter(|| {
                assert!(filler.fill(black_box(&input)).is_ok());
            });
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
