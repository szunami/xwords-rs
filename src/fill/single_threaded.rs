use crate::{
    fill::{fill_one_word, is_viable, words, CrosswordFillState},
    order::FrequencyOrderableCrossword,
    Filler,
};

use rayon::prelude::*;

use rustc_hash::FxHashMap;

use crate::{crossword::CrosswordWordIterator, order::score_iter, parse::parse_word_boundaries};
use std::time::Instant;

use crate::{trie::Trie, Crossword};

#[derive(Clone)]
pub struct SingleThreadedFiller<'s> {
    trie: &'s Trie,
    bigrams: &'s FxHashMap<(char, char), usize>,
}

impl<'s> SingleThreadedFiller<'s> {
    pub fn new(
        trie: &'s Trie,
        bigrams: &'s FxHashMap<(char, char), usize>,
    ) -> SingleThreadedFiller<'s> {
        SingleThreadedFiller { trie, bigrams }
    }
}

const BATCH_SIZE: usize = 4;

impl<'s> Filler for SingleThreadedFiller<'s> {
    fn fill(&self, crossword: &Crossword) -> std::result::Result<Crossword, String> {
        // parse crossword into partially filled words
        // fill a word

        let thread_start = Instant::now();

        let mut crossword_fill_state = {
            let mut temp_state = CrosswordFillState::default();
            let orderable = FrequencyOrderableCrossword::new(crossword.clone(), self.bigrams);
            temp_state.add_candidate(orderable);
            temp_state
        };

        let word_boundaries = parse_word_boundaries(&crossword);
        let mut candidate_count = 0;

        loop {
            let candidates = {
                let mut candidates = vec![];

                for _ in 0..BATCH_SIZE {
                    match crossword_fill_state.take_candidate() {
                        Some(candidate) => candidates.push(candidate),
                        None => continue,
                    }
                }

                candidates
            };

            if candidates.is_empty() {
                return Err(String::from("Yikes"));
            }

            candidate_count += candidates.len();

            if candidate_count % 1_000 <= candidates.len() {
                // println!("{}", candidate);
                println!(
                    "Throughput: {}",
                    candidate_count as f32 / thread_start.elapsed().as_millis() as f32
                );
            }

            let viables: Vec<Crossword> = candidates.par_iter().flat_map(|candidate| {
                let to_fill = word_boundaries
                    .iter()
                    .map(|word_boundary| CrosswordWordIterator::new(&candidate, word_boundary))
                    .filter(|iter| iter.clone().any(|c| c == ' '))
                    .min_by_key(|iter| score_iter(iter, self.bigrams))
                    .unwrap();
                // find valid fills for word;
                // for each fill:
                //   are all complete words legit?
                //     if so, push

                let potential_fills = words(to_fill.clone().to_string(), self.trie);

                let mut viables = vec![];

                for potential_fill in potential_fills {
                    let new_candidate = fill_one_word(&candidate, &to_fill.clone(), potential_fill);

                    if is_viable(&new_candidate, &word_boundaries, self.trie) {
                        viables.push(new_candidate);
                    }
                }
                viables
            }).collect();

            for viable in viables {
                if !viable.contents.contains(' ') {
                    return Ok(viable);
                }
                let orderable = FrequencyOrderableCrossword::new(viable, self.bigrams);
                if orderable.fillability_score > 0 {
                    crossword_fill_state.add_candidate(orderable);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{default_indexes, fill::Filler};

    use crate::Crossword;

    use std::{sync::Arc, time::Instant};

    use super::SingleThreadedFiller;


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
        let filler = SingleThreadedFiller::new(&trie, &bigrams);
        let filled_puz = filler.fill(&grid).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }
}
