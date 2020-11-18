use std::sync::Arc;
use xwords::fill::fill_crossword;
use xwords::{crossword::Crossword, default_indexes};

use criterion::Benchmark;
use criterion::{criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let (bigrams, trie) = default_indexes();

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

    let tmp_bigrams = bigrams.clone();
    let tmp_trie = trie.clone();

    c.bench(
        "fill_crosswords",
        Benchmark::new("fill_20201012_crossword", move |b| {
            b.iter(|| fill_crossword(&input, tmp_trie.clone(), tmp_bigrams.clone()));
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
