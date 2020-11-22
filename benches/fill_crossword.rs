use std::sync::Arc;
use xwords::fill::Filler;
use xwords::{crossword::Crossword, default_indexes, fill::parallel::ParallelFiller};

use criterion::{black_box, Benchmark};
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let (bigrams, trie) = default_indexes();
    let trie = Arc::new(trie);
    let bigrams = Arc::new(bigrams);

    let tmp_bigrams = bigrams.clone();
    let tmp_trie = trie.clone();

    c.bench(
        "fill_crosswords",
        Benchmark::new("fill_3x3_crossword", move |b| {
            let input = Crossword::new(String::from("         ")).unwrap();

            let filler = ParallelFiller::new(tmp_trie.clone(), tmp_bigrams.clone());
            b.iter(|| assert!(filler.fill(black_box(&input)).is_ok()));
        }),
    );

    let tmp_bigrams = bigrams.clone();
    let tmp_trie = trie.clone();
    c.bench(
        "fill_crosswords",
        Benchmark::new("fill_4x4_crossword", move |b| {
            let input = Crossword::new(String::from("                ")).unwrap();
            let filler = ParallelFiller::new(tmp_trie.clone(), tmp_bigrams.clone());
            b.iter(|| filler.fill(black_box(&input)));
        }),
    );

    let tmp_bigrams = bigrams.clone();
    let tmp_trie = trie.clone();
    c.bench(
        "fill_crosswords",
        Benchmark::new("fill_20201012_crossword", move |b| {
            let input = Crossword::new(String::from(
                "
  S *FRAN*BANAL
  E *L  O*ALIBI
BARITONES*N   O
ENV* W *E D*  N
**E  E*BROILERS
RATEDR*AINTI***
AMITY*B N *ACDC
M M*AMALGAM*R  
P E * L S*AMINO
***ACIDY*GRATES
ENDZONES*A  I**
KIA*A A* R *C  
EVILS*GOODTHING
B  ET*L  E* S  
YAYAS*ETON* M  
",
            ))
            .unwrap();
            let filler = ParallelFiller::new(tmp_trie.clone(), tmp_bigrams.clone());
            b.iter(|| filler.fill(black_box(&input)));
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
