use crate::fill::fill_one_word;
use crate::fill::is_viable;
use crate::fill::words;
use crate::fill::Filler;
use crate::order::score_iter;
use crate::order::FrequencyOrderableCrossword;
use crate::parse::parse_word_boundaries;
use std::time::Instant;

use crate::crossword::CrosswordWordIterator;
use std::{
    collections::BinaryHeap,
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
};

use crate::{trie::Trie, Crossword};

pub struct CrosswordFillState {
    // Used to ensure we only enqueue each crossword once.
    // Contains crosswords that are queued or have already been visited
    candidate_queue: BinaryHeap<FrequencyOrderableCrossword>,
    done: bool,
}

impl CrosswordFillState {
    pub fn default() -> CrosswordFillState {
        CrosswordFillState {
            candidate_queue: BinaryHeap::new(),
            done: false,
        }
    }

    pub fn take_candidate(&mut self) -> Option<Crossword> {
        self.candidate_queue.pop().map(|x| x.crossword)
    }

    pub fn add_candidate(&mut self, candidate: FrequencyOrderableCrossword) {
        self.candidate_queue.push(candidate);
    }

    pub fn mark_done(&mut self) {
        self.done = true;
    }
}

#[derive(Clone)]
pub struct ParallelFiller {
    trie: Arc<Trie>,
    bigrams: Arc<HashMap<(char, char), usize>>,
}

impl ParallelFiller {
    pub fn new(trie: Arc<Trie>, bigrams: Arc<HashMap<(char, char), usize>>) -> ParallelFiller {
        ParallelFiller { trie, bigrams }
    }
}

impl Filler for ParallelFiller {
    fn fill(self, crossword: &Crossword) -> Result<Crossword, String> {
        let crossword_fill_state = {
            let mut temp_state = CrosswordFillState::default();
            let orderable = FrequencyOrderableCrossword::new(crossword.clone(), &self.bigrams);
            temp_state.add_candidate(orderable);
            temp_state
        };

        let candidates = Arc::new(Mutex::new(crossword_fill_state));

        let trie = Arc::new(self.trie);
        let bigrams = Arc::new(self.bigrams);

        let (tx, rx) = mpsc::channel();
        // want to spawn multiple threads, have each of them perform the below

        for thread_index in 0..2 {
            let new_arc = Arc::clone(&candidates);
            let new_tx = tx.clone();
            let word_boundaries = parse_word_boundaries(&crossword);

            let mut candidate_count = 0;

            let trie = trie.clone();
            let bigrams = bigrams.clone();

            std::thread::Builder::new()
                .name(format!("worker{}", thread_index))
                .spawn(move || {
                    println!("Hello from thread {}", thread_index);

                    let thread_start = Instant::now();

                    loop {
                        let candidate = {
                            let mut queue = new_arc.lock().unwrap();
                            if queue.done {
                                return;
                            }
                            match queue.take_candidate() {
                                Some(candidate) => candidate,
                                None => continue,
                            }
                        };

                        candidate_count += 1;

                        if candidate_count % 1_000 == 0 {
                            println!(
                                "Thread {} throughput: {}",
                                thread_index,
                                candidate_count as f32 / thread_start.elapsed().as_millis() as f32
                            );
                            // println!("{}", candidate);
                        }

                        let to_fill = word_boundaries
                            .iter()
                            .map(|word_boundary| {
                                CrosswordWordIterator::new(&candidate, word_boundary)
                            })
                            .filter(|iter| iter.clone().any(|c| c == ' '))
                            .min_by_key(|iter| score_iter(iter, bigrams.as_ref()))
                            .unwrap();
                        // find valid fills for word;
                        // for each fill:
                        //   are all complete words legit?
                        //     if so, push

                        let potential_fills = words(to_fill.clone().to_string(), trie.as_ref());
                        let mut viables: Vec<FrequencyOrderableCrossword> = vec![];

                        for potential_fill in potential_fills {
                            let new_candidate =
                                fill_one_word(&candidate, &to_fill.clone(), potential_fill);

                            if is_viable(&new_candidate, &word_boundaries, trie.as_ref()) {
                                if !new_candidate.contents.contains(' ') {
                                    let mut queue = new_arc.lock().unwrap();
                                    queue.mark_done();

                                    match new_tx.send(new_candidate) {
                                        Ok(_) => {
                                            println!("Just sent a result.");
                                            return;
                                        }
                                        Err(err) => {
                                            println!("Failed to send a result, error was {}", err);
                                            return;
                                        }
                                    }
                                }
                                let orderable = FrequencyOrderableCrossword::new(
                                    new_candidate,
                                    bigrams.as_ref(),
                                );
                                if orderable.fillability_score > 0 {
                                    viables.push(orderable);
                                }
                            }
                        }

                        if !viables.is_empty() {
                            let mut queue = new_arc.lock().unwrap();
                            for viable_crossword in viables {
                                queue.add_candidate(viable_crossword);
                            }
                        }
                    }
                })
                .unwrap();
        }

        match rx.recv() {
            Ok(result) => Ok(result),
            Err(_) => Err(String::from("Failed to receive")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::default_indexes;
    use crate::fill::Filler;
    use crate::Arc;
    use crate::Crossword;
    use std::fs::File;
    use std::time::Instant;

    use super::ParallelFiller;

    #[test]
    fn fill_crossword_works() {
        let (bigrams, trie) = default_indexes();
        let filler = ParallelFiller::new(Arc::new(trie), Arc::new(bigrams));

        let input = Crossword::new(String::from("                ")).unwrap();

        let result = filler.fill(&input);

        assert!(result.is_ok());

        println!("{}", result.unwrap());
    }

    #[test]
    #[ignore]
    fn puz_2020_10_12_works() {
        let guard = pprof::ProfilerGuard::new(100).unwrap();
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
  S *F  N*B    
  E *L  O*ALIBI
BARITONES*N    
  V* W *E D*   
**E  E*BROILERS
RATEDR*AINTI***
  I  *B N * C  
  M*AMALGAM*R  
  E * L S*AMINO
***ACIDY*GRATES
ENDZONES*A  I**
KIA*  A* R *C  
EVILS*GOODTHING
B    *L  E* S  
YAYAS*E  N* M  
",
        ))
        .unwrap();

        println!("{}", real_puz);

        let (bigrams, trie) = default_indexes();
        let now = Instant::now();

        let filler = ParallelFiller::new(Arc::new(trie), Arc::new(bigrams));
        let filled_puz = filler.fill(&real_puz).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }

    #[test]
    #[ignore]
    fn _2020_10_12_empty_works() {
        let now = Instant::now();

        let guard = pprof::ProfilerGuard::new(100).unwrap();
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
  S *F  N*B    
  E *L  O*A    
         *N    
  V* W *E D*   
**E  E*BROILERS
RATEDR*     ***
  I  *B N * C  
  M*       *R  
  E * L S*     
***ACIDY*      
ENDZONES*A  I**
KIA*  A* R *C  
EVILS*         
B    *L  E* S  
YAYAS*E  N* M  
",
        ))
        .unwrap();

        println!("{}", real_puz);

        println!("Loaded indices in {}ms", now.elapsed().as_millis());
        let (bigrams, trie) = default_indexes();
        let filler = ParallelFiller::new(Arc::new(trie), Arc::new(bigrams));
        let filled_puz = filler.fill(&real_puz).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }

    #[test]
    fn medium_grid() {
        let grid = Crossword::new(String::from(
            "
    ***
    ***
    ***
       
***    
***    
***    
",
        ))
        .unwrap();

        let now = Instant::now();
        let (bigrams, trie) = default_indexes();
        let filler = ParallelFiller::new(Arc::new(trie), Arc::new(bigrams));
        let filled_puz = filler.fill(&grid).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }
}
