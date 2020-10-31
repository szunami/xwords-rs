use xwords::default_indexes;
use std::sync::Arc;
use xwords::crossword::Crossword;
use xwords::default_words;
use xwords::fill::fill_crossword;

use criterion::Benchmark;
use criterion::{criterion_group, criterion_main, Criterion};
use xwords::index_words;

pub fn criterion_benchmark(c: &mut Criterion) {
    let (bigrams, trie) = default_indexes();

    let bigrams = Arc::new(bigrams);
    let trie = Arc::new(trie);

        let real_puz = Crossword::new(String::from(
            "
  S *F  N*B    
  E *L  O*A    
         *N    
  V* W *E D*   
**E  E*BROILERS
RATEDR*     ***
  I  *B N * C  
  M*       *R  
  E * L S*     
***ACIDY*GRATES
ENDZONES*A  I**
KIA*  A* R *C  
EVILS*         
B    *L  E* S  
YAYAS*E  N* M  
",
        ))
        .unwrap();

        c.bench(
            "fill_crosswords",
            Benchmark::new("fill_real_crossword", move |b| {
                b.iter(|| fill_crossword(&real_puz, trie.clone(), bigrams.clone()));
            }),
        );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
