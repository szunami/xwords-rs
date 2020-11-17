use crate::order::score_iter;
use crate::Instant;
use crate::order::FrequencyOrderableCrossword;
use crate::parse::parse_word_boundaries;
use crate::parse::parse_words;
use crate::Word;
use crate::{crossword::CrosswordWordIterator, parse::WordBoundary};
use crate::{score_word, Direction};
use cached::SizedCache;
use std::{collections::BinaryHeap, collections::{hash_map::DefaultHasher, HashMap, HashSet}, hash::Hash, hash::Hasher, sync::{mpsc, Arc, Mutex}};

use crate::{trie::Trie, Crossword};

struct CrosswordFillState {
    candidate_queue: BinaryHeap<FrequencyOrderableCrossword>,
    done: bool,
}

impl CrosswordFillState {
    fn take_candidate(&mut self) -> Option<Crossword> {
        self.candidate_queue.pop().map(|x| x.crossword)
    }

    fn add_candidate(&mut self, candidate: Crossword, bigrams: &HashMap<(char, char), usize>) {
            let orderable = FrequencyOrderableCrossword::new(candidate.clone(), bigrams);
            self.candidate_queue.push(orderable);
    }

    fn mark_done(&mut self) {
        self.done = true;
    }
}

fn fill_one_word_tmp(candidate: &Crossword, iter: &CrosswordWordIterator, word: String) -> Crossword {
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

fn fill_one_word(candidate: &Crossword, potential_fill: Word) -> Crossword {
    let mut result_contents = candidate.contents.clone();

    match potential_fill.direction {
        Direction::Across => {
            let mut bytes = result_contents.into_bytes();

            for index in 0..potential_fill.contents.len() {
                let col = potential_fill.start_col + index;

                bytes[potential_fill.start_row * candidate.width + col] =
                    potential_fill.contents.as_bytes()[index];
            }
            unsafe { result_contents = String::from_utf8_unchecked(bytes) }
        }
        Direction::Down => {
            let mut bytes = result_contents.into_bytes();

            for index in 0..potential_fill.contents.len() {
                let row = potential_fill.start_row + index;

                bytes[row * candidate.width + potential_fill.start_col] =
                    potential_fill.contents.as_bytes()[index];
            }
            unsafe { result_contents = String::from_utf8_unchecked(bytes) }
        }
    }

    Crossword {
        contents: result_contents,
        ..*candidate
    }
}

cached_key! {
    IS_WORD: SizedCache<u64, bool> = SizedCache::with_size(10_000);
    Key = { iter.clone().hash() };
    fn is_word(iter: CrosswordWordIterator, trie: &Trie) -> bool = {
        trie.is_word(iter)
    }
}

cached_key! {
    WORDS: SizedCache<u64, Vec<String>> = SizedCache::with_size(10_000);
    Key = { 
        let mut hasher = DefaultHasher::new();
        for c in pattern.chars() {
            c.hash(&mut hasher)
        }
        hasher.finish()
    };
    fn words(pattern: String, trie: &Trie) -> Vec<String> = {
        trie.words(pattern)
    }
}

pub fn fill_crossword(
    crossword: &Crossword,
    trie: Arc<Trie>,
    bigrams: Arc<HashMap<(char, char), usize>>,
) -> Result<Crossword, String> {
    // parse crossword into partially filled words
    // fill a word

    let crossword_fill_state = {
        let mut temp_state = CrosswordFillState {
            candidate_queue: BinaryHeap::new(),
            done: false,
        };
        temp_state.add_candidate(crossword.clone(), bigrams.as_ref());
        temp_state
    };

    let candidates = Arc::new(Mutex::new(crossword_fill_state));
    let (tx, rx) = mpsc::channel();
    // want to spawn multiple threads, have each of them perform the below

    for thread_index in 0..1 {
        let new_arc = Arc::clone(&candidates);
        let new_tx = tx.clone();
        let word_boundaries = parse_word_boundaries(&crossword);

        let trie = trie.clone();
        let bigrams = bigrams.clone();
        let mut candidate_count = 0;

        std::thread::Builder::new()
            .name(String::from(format!("worker{}", thread_index)))
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

                    if candidate_count % 10_000 == 0 {
                        println!("Thread {} throughput: {}", thread_index, candidate_count as f32 / thread_start.elapsed().as_millis() as f32);
                        println!("{}", candidate);
                    }

                    let to_fill = word_boundaries.iter().map(|word_boundary| { 
                        CrosswordWordIterator::new(&candidate, word_boundary)
                    }).filter(|iter| iter.clone().any(|c| c == ' '))
                    .min_by_key(|iter| score_iter(iter, bigrams.as_ref()))
                    .unwrap();
                    
                    println!("FIlling {}", to_fill.clone().to_string());
                    
                    // let words = parse_words(&candidate);
                    // let to_fill = words
                    //     .iter()
                    //     .filter(|word| word.contents.chars().any(|c| c == ' '))
                    //     .min_by_key(|word| score_word(&word.contents, bigrams.as_ref()))
                    //     .unwrap();
                        
                    // choose the right word boundary
                    // find all 
                        
                        
                        
                    // find valid fills for word;
                    // for each fill:
                    //   are all complete words legit?
                    //     if so, push
                    
                    let potential_fills = words(to_fill.clone().to_string(), trie.as_ref());
                    for potential_fill in potential_fills {
                        let new_candidate = fill_one_word_tmp(&candidate, &to_fill.clone(), potential_fill);

                        let mut viables: Vec<Crossword> = vec![];

                        if is_viable(&new_candidate, &word_boundaries, trie.as_ref()) {
                            if !new_candidate.contents.contains(" ") {
                                let mut queue = new_arc.lock().unwrap();
                                queue.mark_done();

                                match new_tx.send(new_candidate.clone()) {
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

                            viables.push(new_candidate);
                        }

                        if !viables.is_empty() {
                            let mut queue = new_arc.lock().unwrap();

                            for viable_crossword in viables {
                                queue.add_candidate(viable_crossword, bigrams.as_ref());
                            }
                        }
                    }
                }
            })
            .unwrap();
    }

    match rx.recv() {
        Ok(result) => {
            let queue = candidates.lock().unwrap();

            Ok(result)
        }
        Err(_) => Err(String::from("Failed to receive")),
    }
}

fn is_viable(candidate: &Crossword, word_boundaries: &Vec<WordBoundary>, trie: &Trie) -> bool {
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
    use crate::default_indexes;
    use crate::fill::is_word;

    use crate::Trie;
    use crate::{crossword::CrosswordWordIterator, parse::WordBoundary};
    use crate::{default_words, parse::parse_word_boundaries};
    use crate::{index_words, Crossword, Direction, Word};
    use std::fs::File;
    use std::{sync::Arc, time::Instant};

    use super::{fill_crossword, fill_one_word, is_viable};

    #[test]
    fn fill_crossword_works() {
        let (bigrams, trie) = index_words(default_words());

        let input = Crossword::new(String::from("                ")).unwrap();

        let result = fill_crossword(&input, Arc::new(trie), Arc::new(bigrams));

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

        let (bigrams, trie) = index_words(default_words());
        let now = Instant::now();

        let filled_puz = fill_crossword(&real_puz, Arc::new(trie), Arc::new(bigrams)).unwrap();
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

        let (bigrams, trie) = default_indexes();
        println!("Loaded indices in {}ms", now.elapsed().as_millis());

        let filled_puz = fill_crossword(&real_puz, Arc::new(trie), Arc::new(bigrams)).unwrap();
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
                Word {
                    contents: String::from("cat"),
                    start_col: 0,
                    start_row: 0,
                    length: 3,
                    direction: Direction::Across,
                }
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
                Word {
                    contents: String::from("cat"),
                    start_col: 0,
                    start_row: 0,
                    length: 3,
                    direction: Direction::Down,
                }
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

        let (bigrams, trie) = index_words(default_words());

        let now = Instant::now();

        let filled_puz = fill_crossword(&grid, Arc::new(trie), Arc::new(bigrams)).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }

    // #[test]
    // fn find_fill_works() {
    //     let (_, trie) = index_words(default_words());

    //     let input = Word {
    //         contents: String::from("   "),
    //         length: 3,
    //         start_row: 0,
    //         start_col: 0,
    //         direction: Direction::Across,
    //     };
    //     assert!(find_fills(input.clone(), &trie).contains(&Word {
    //         contents: String::from("CAT"),
    //         ..input.clone()
    //     }));

    //     let input = Word {
    //         contents: String::from("C T"),
    //         length: 3,
    //         start_row: 0,
    //         start_col: 0,
    //         direction: Direction::Across,
    //     };
    //     assert!(find_fills(input.clone(), &trie).contains(&Word {
    //         contents: String::from("CAT"),
    //         ..input.clone()
    //     }));

    //     let input = Word {
    //         contents: String::from("  T"),
    //         length: 3,
    //         start_row: 0,
    //         start_col: 0,
    //         direction: Direction::Across,
    //     };
    //     assert!(find_fills(input.clone(), &trie).contains(&Word {
    //         contents: String::from("CAT"),
    //         ..input.clone()
    //     }));

    //     let input = Word {
    //         contents: String::from("CAT"),
    //         length: 3,
    //         start_row: 0,
    //         start_col: 0,
    //         direction: Direction::Across,
    //     };
    //     assert!(find_fills(input.clone(), &trie).contains(&Word {
    //         contents: String::from("CAT"),
    //         ..input.clone()
    //     }));
    // }

    #[test]
    fn is_viable_works() {
        let (_, trie) = index_words(default_words());

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
