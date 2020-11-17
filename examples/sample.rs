use std::sync::Arc;
use xwords::fill::fill_crossword;
use xwords::default_indexes;
use xwords::crossword::Crossword;
use std::fs::File;
use std::time::Instant;

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

      let filled_puz = fill_crossword(&real_puz, Arc::new(trie), Arc::new(bigrams)).unwrap();
    println!("Filled in {} seconds.", now.elapsed().as_secs());
    println!("{}", filled_puz);}