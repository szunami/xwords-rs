use crate::fill::Filler;
use crate::order::score_iter;
use crate::order::FrequencyOrderableCrossword;
use crate::parse::parse_word_boundaries;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use crate::Direction;
use crate::Instant;
use crate::{crossword::CrosswordWordIterator, parse::WordBoundary};
use cached::SizedCache;
use std::{
    collections::BinaryHeap,
    collections::{HashMap, HashSet},
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

pub fn fill_one_word(
    candidate: &Crossword,
    iter: &CrosswordWordIterator,
    word: String,
) -> Crossword {
    let mut result_contents = candidate.contents.clone();
    let mut bytes = result_contents.into_bytes();

    let word_boundary = iter.word_boundary;

    match word_boundary.direction {
        Direction::Across => {
            for (char_index, c) in word.chars().enumerate() {
                let col = word_boundary.start_col + char_index;
                bytes[word_boundary.start_row * candidate.width + col] = c as u8;
            }
        }
        Direction::Down => {
            for (char_index, c) in word.chars().enumerate() {
                let row = word_boundary.start_row + char_index;
                bytes[row * candidate.width + word_boundary.start_col] = c as u8;
            }
        }
    }
    unsafe { result_contents = String::from_utf8_unchecked(bytes) }

    Crossword {
        contents: result_contents,
        ..*candidate
    }
}

cached_key! {
    IS_WORD: SizedCache<u64, bool> = SizedCache::with_size(10_000);
    Key = {
        use std::hash::Hash;
        let mut hasher = DefaultHasher::new();
        for c in iter.clone() {
            c.hash(&mut hasher)
        }

        hasher.finish()
    };
    fn is_word(iter: CrosswordWordIterator, trie: &Trie) -> bool = {
        trie.is_word(iter)
    }
}

cached_key! {
    WORDS: SizedCache<String, Vec<String>> = SizedCache::with_size(10_000);
    Key = { pattern.clone() };
    fn words(pattern: String, trie: &Trie) -> Vec<String> = {
        trie.words(pattern)
    }
}

pub fn is_viable(candidate: &Crossword, word_boundaries: &[WordBoundary], trie: &Trie) -> bool {
    let mut already_used = HashSet::with_capacity(word_boundaries.len());

    for word_boundary in word_boundaries {
        let iter = CrosswordWordIterator::new(candidate, word_boundary);
        if iter.clone().any(|c| c == ' ') {
            continue;
        }

        if already_used.contains(&iter) {
            return false;
        }
        already_used.insert(iter.clone());

        if !is_word(iter, trie) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::fill::Filler;
    use crate::{default_indexes, fill::parallel::is_word};

    use crate::Trie;
    use crate::{crossword::CrosswordWordIterator, parse::WordBoundary};
    use crate::{default_words, parse::parse_word_boundaries};
    use crate::{index_words, Crossword, Direction};
    use std::fs::File;
    use std::{sync::Arc, time::Instant};

    use super::{fill_one_word, is_viable, ParallelFiller};

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
    fn fill_one_word_works() {
        let c = Crossword::new(String::from(
            "
abc
def
ghi
",
        ))
        .unwrap();

        assert_eq!(
            fill_one_word(
                &c,
                &CrosswordWordIterator::new(
                    &c,
                    &WordBoundary {
                        start_col: 0,
                        start_row: 0,
                        length: 3,
                        direction: Direction::Across,
                    },
                ),
                String::from("cat")
            ),
            Crossword::new(String::from(
                "
cat
def
ghi
",
            ))
            .unwrap()
        );

        assert_eq!(
            fill_one_word(
                &c,
                &CrosswordWordIterator::new(
                    &c,
                    &WordBoundary {
                        start_col: 0,
                        start_row: 0,
                        length: 3,
                        direction: Direction::Down,
                    }
                ),
                String::from("cat"),
            ),
            Crossword::new(String::from(
                "
cbc
aef
thi
",
            ))
            .unwrap()
        );
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

    #[test]
    fn is_viable_works() {
        let (_, trie) = default_indexes();

        let crossword = Crossword::new(String::from(
            "
   
   
   
",
        ))
        .unwrap();

        let word_boundaries = parse_word_boundaries(&crossword);

        assert!(is_viable(&crossword, &word_boundaries, &trie));

        assert!(!is_viable(
            &Crossword::new(String::from("ABCDEFGH ")).unwrap(),
            &word_boundaries,
            &trie
        ));

        assert!(!is_viable(
            &Crossword::new(String::from("ABCB  C  ")).unwrap(),
            &word_boundaries,
            &trie
        ));
    }

    #[test]
    fn cache_works() {
        let trie = Trie::build(vec![
            String::from("bass"),
            String::from("bats"),
            String::from("bess"),
            String::from("be"),
        ]);

        let crossword = Crossword::new(String::from(
            "
bass
a   
s   
s   
",
        ))
        .unwrap();
        let word_boundary = WordBoundary {
            start_row: 0,
            start_col: 0,
            direction: Direction::Across,
            length: 4,
        };
        let iter = CrosswordWordIterator::new(&crossword, &word_boundary);

        assert!(is_word(iter, &trie));

        let word_boundary = WordBoundary {
            start_row: 0,
            start_col: 0,
            direction: Direction::Down,
            length: 4,
        };
        let iter = CrosswordWordIterator::new(&crossword, &word_boundary);

        assert!(is_word(iter, &trie));
    }
}
