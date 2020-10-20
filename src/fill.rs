use crate::is_viable;
use crate::Word;
use crate::{Direction, find_fills, score_word};
use crate::parse_words;
use crate::parse_word_boundaries;
use std::{collections::BinaryHeap, collections::{HashMap, HashSet}, sync::{Arc, Mutex, mpsc}};

use crate::{Crossword, CrosswordFillState, trie::Trie};

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

pub fn fill_crossword(
    crossword: &Crossword,
    trie: Arc<Trie>,
    bigrams: Arc<HashMap<(char, char), usize>>,
) -> Result<Crossword, String> {
    // parse crossword into partially filled words
    // fill a word

    let crossword_fill_state = {
        let mut temp_state = CrosswordFillState {
            processed_candidates: HashSet::new(),
            candidate_queue: BinaryHeap::new(),
            done: false,
        };
        temp_state.add_candidate(crossword.clone(), bigrams.as_ref());
        temp_state
    };

    let candidates = Arc::new(Mutex::new(crossword_fill_state));
    let (tx, rx) = mpsc::channel();
    // want to spawn multiple threads, have each of them perform the below

    for thread_index in 0..4 {
        let new_arc = Arc::clone(&candidates);
        let new_tx = tx.clone();
        let word_boundaries = parse_word_boundaries(&crossword);

        let trie = trie.clone();
        let bigrams = bigrams.clone();
        let mut candidate_count = 0;

        std::thread::Builder::new()
            .name(String::from("worker"))
            .spawn(move || {
                println!("Hello from thread {}", thread_index);

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

                    if candidate_count % 100 == 0 {
                        println!("{}", candidate);
                    }
                    // println!("Thread {} just got a candidate", thread_index);

                    let words = parse_words(&candidate);
                    let to_fill = words
                        .iter()
                        .filter(|word| word.contents.chars().any(|c| c == ' '))
                        .min_by_key(|word| score_word(&word.contents, bigrams.as_ref()))
                        .unwrap();
                    // find valid fills for word;
                    // for each fill:
                    //   are all complete words legit?
                    //     if so, push

                    let potential_fills = find_fills(to_fill.clone(), trie.as_ref());
                    for potential_fill in potential_fills {
                        let new_candidate = fill_one_word(&candidate, potential_fill);

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

                            let mut queue = new_arc.lock().unwrap();
                            queue.add_candidate(new_candidate, bigrams.as_ref());
                        }
                    }
                }
            })
            .unwrap();
    }

    match rx.recv() {
        Ok(result) => {
            let queue = candidates.lock().unwrap();

            println!("Processed {} candidates", queue.processed_candidates.len());
            Ok(result)
        }
        Err(_) => Err(String::from("Failed to receive")),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
use std::fs::File;
use crate::{Crossword, index_words};

    use super::fill_crossword;


    
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
BARITONES*N    
  V* W *E D*   
**E  E*BROILERS
RATEDR*     ***
  I  *B N * C  
  M*AMALGAM*R  
  E * L S*     
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
}