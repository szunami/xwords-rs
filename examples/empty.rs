use xwords::fill::single_threaded::SingleThreadedFiller;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;
use xwords::crossword::Crossword;
use xwords::default_indexes;
use xwords::fill::parallel::ParallelFiller;
use xwords::fill::Filler;

fn main() {
    let now = Instant::now();

    let guard = pprof::ProfilerGuard::new(1000).unwrap();
    std::thread::spawn(move || loop {
        match guard.report().build() {
            Ok(report) => {
                let file = File::create("flamegraph.svg").unwrap();
                report.flamegraph(file).unwrap();
            }
            Err(_) => {}
        };
        std::thread::sleep(std::time::Duration::from_secs(5))
    });

    let real_puz = Crossword::new(String::from(
        "
    *    *     
    *    *     
         *     
   *   *   *   
**    *        
      *     ***
     *    *    
   *       *   
    *    *     
***     *      
        *    **
   *   *   *   
     *         
     *    *    
     *    *    
",
    ))
    .unwrap();

    println!("{}", real_puz);

    let (bigrams, trie) = default_indexes();
    println!("Loaded indices in {}ms", now.elapsed().as_millis());

    let filler = SingleThreadedFiller::new(&trie, &bigrams);

    let filled_puz = filler.fill(&real_puz).unwrap();
    println!("Filled in {} seconds.", now.elapsed().as_secs());
    println!("{}", filled_puz);
}
