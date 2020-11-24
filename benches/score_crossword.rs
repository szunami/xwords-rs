use criterion::{black_box, criterion_group, criterion_main, Benchmark, Criterion};
use std::sync::Arc;
use xwords::{crossword::Crossword, default_indexes, order::score_crossword};

pub fn criterion_benchmark(c: &mut Criterion) {
    let (bigrams, _) = default_indexes();
    let bigrams = Arc::new(bigrams);
    let tmp_bigrams = bigrams.clone();

    c.bench(
        "score_crossword",
        Benchmark::new("20201012_partial", move |b| {
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

            b.iter(|| {
                assert!(
                    score_crossword(black_box(&tmp_bigrams.as_ref()), black_box(&crossword)) > 0
                );
            });
        }),
    );
    let tmp_bigrams = bigrams.clone();

    c.bench(
        "score_crossword",
        Benchmark::new("20201012_invalid", move |b| {
            let crossword = Crossword::new(String::from(
                "
  S *F  N*B    
  E *L  O*A    
YARDGOODS*N    
  V*OW *E D*   
**E WE*GREENING
RATEDR*LI D ***
Q I E*BAN * C  
X M*NOODGES*R  
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

            b.iter(|| {
                assert_eq!(
                    score_crossword(black_box(&tmp_bigrams.as_ref()), black_box(&crossword)),
                    0
                );
            });
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
