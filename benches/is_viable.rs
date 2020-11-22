use criterion::black_box;
use xwords::{default_indexes, fill::is_viable};
use criterion::{Benchmark, Criterion, criterion_group, criterion_main};
use xwords::crossword::Crossword;
use xwords::parse::parse_word_boundaries;

pub fn criterion_benchmark(c: &mut Criterion) {
    
    let crossword = Crossword::new(String::from("
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
")).unwrap();
    let word_boundaries = parse_word_boundaries(&crossword);
    let (_, trie) = default_indexes();
    c.bench(
        "is_viable",
        Benchmark::new("", move |b| {
            b.iter(|| {
                assert!(
                    is_viable(
                        black_box(&crossword), 
                        black_box(&word_boundaries),
                        black_box(&trie)
                    ));
            });
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
