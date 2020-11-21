use std::sync::Arc;
use std::time::Instant;
use xwords::crossword::Crossword;
use xwords::default_indexes;
use xwords::fill::parallel::ParallelFiller;
use xwords::fill::Filler;

fn main() {
    let now = Instant::now();

    let real_puz = Crossword::new(String::from(
        "
  S *F  N*B
  E *L  O*A
         *N
  V* W *E D*
**E  E*
RATEDR*     ***
  I  *B N * C
  M*       *R
  E * L S*
***ACIDY*GRATES
        *A  I**
KIA*  A* R *C
EVILS*
B    *L  E* S
YAYAS*E  N* M
",
    ))
    .unwrap();

    println!("{}", real_puz);

    let (bigrams, trie) = default_indexes();
    println!("Loaded indices in {}ms", now.elapsed().as_millis());

    let filler = ParallelFiller::new(Arc::new(trie), Arc::new(bigrams));
    let filled_puz = filler.fill(&real_puz).unwrap();
    println!("Filled in {} seconds.", now.elapsed().as_secs());
    println!("{}", filled_puz);
}
