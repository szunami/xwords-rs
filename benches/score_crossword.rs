use criterion::{black_box, criterion_group, criterion_main, Benchmark, Criterion};
use xwords::{crossword::{Crossword, CrosswordWordIterator, Direction}, default_indexes, parse::WordBoundary, order::score_crossword};

pub fn criterion_benchmark(c: &mut Criterion) {
    let (bigrams, _) = default_indexes();

    c.bench(
        "score_crossword",
        Benchmark::new("20201012_YARDGOODS", move |b| {
            let crossword = Crossword::new(String::from(
                "
  S *F  N*B    
  E *L  O*A    
YARDGOODS*N    
  V*OW *E D*   
**E WE*GREENING
RATEDR*LI D ***
  I E*BAN * C  
  M*NOODGES*R  
  E * LWS*C I  
***ACIDY*GRATES
EVILOMEN*AI I**
KIA*A A* RN*C  
EVILS*GUIDELINE
BD  T*L  E* S  
YAYAS*E  N* M  
",
            ))
            .unwrap();
            let word_boundary = WordBoundary::new(2, 0, 9, Direction::Across);
            let iter = CrosswordWordIterator::new(&crossword, &word_boundary);

            b.iter(|| {
                assert!(score_crossword(black_box(&bigrams), black_box(&crossword)) > 0);
            });
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
