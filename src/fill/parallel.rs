use crate::fill::{fill_one_word, is_viable, words};

use crate::fill::{CrosswordFillState, Filler};

use crate::{
    order::{score_iter, FrequencyOrderableCrossword},
    parse::parse_word_boundaries,
};

use crate::{crossword::CrosswordWordIterator, Instant};

use rustc_hash::FxHashMap;
use std::sync::{mpsc, Arc, Mutex};

use crate::{trie::Trie, Crossword};

#[derive(Clone)]
pub struct ParallelFiller {
    trie: Arc<Trie>,
    bigrams: Arc<FxHashMap<(char, char), usize>>,
}

impl ParallelFiller {
    pub fn new(trie: Arc<Trie>, bigrams: Arc<FxHashMap<(char, char), usize>>) -> ParallelFiller {
        ParallelFiller { trie, bigrams }
    }
}

impl Filler for ParallelFiller {
    fn fill(&self, crossword: &Crossword) -> Result<Crossword, String> {
        let crossword_fill_state = {
            let mut temp_state = CrosswordFillState::default();
            let orderable = FrequencyOrderableCrossword::new(crossword.clone(), &self.bigrams);
            temp_state.add_candidate(orderable);
            temp_state
        };

        let candidates = Arc::new(Mutex::new(crossword_fill_state));

        let trie = self.trie.clone();
        let bigrams = self.bigrams.clone();

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

                        let potential_fills = words(to_fill.clone(), trie.as_ref());
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

    use crate::{default_indexes, fill::Filler};

    use crate::Crossword;

    use std::{sync::Arc, time::Instant};

    use super::ParallelFiller;

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
