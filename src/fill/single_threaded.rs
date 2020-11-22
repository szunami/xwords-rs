use crate::fill::parallel::MyCache;
use crate::fill::parallel::fill_one_word;
use crate::fill::parallel::is_viable;
use crate::fill::parallel::CrosswordFillState;
use crate::order::FrequencyOrderableCrossword;
use crate::Filler;
use cached::SizedCache;
use fxhash::FxHashMap;

use crate::crossword::CrosswordWordIterator;
use crate::order::score_iter;
use crate::parse::parse_word_boundaries;
use std::{collections::HashMap, time::Instant};

use crate::{trie::Trie, Crossword};

cached_key! {
    WORDS: MyCache<String, Vec<String>> = MyCache::default();
    Key = { pattern.clone() };
    fn words(pattern: String, trie: &Trie) -> Vec<String> = {
        trie.words(pattern)
    }
}

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

impl<'s> Filler for SingleThreadedFiller<'s> {
    fn fill(self, crossword: &Crossword) -> std::result::Result<Crossword, String> {
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
            let candidate = match crossword_fill_state.take_candidate() {
                Some(c) => c,
                None => return Err(String::from("Ran out of candidates. Yikes.")),
            };

            candidate_count += 1;

            if candidate_count % 10_000 == 0 {
                println!("{}", candidate);
                println!(
                    "Throughput: {}",
                    candidate_count as f32 / thread_start.elapsed().as_millis() as f32
                );
            }

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

            for potential_fill in potential_fills {
                let new_candidate = fill_one_word(&candidate, &to_fill.clone(), potential_fill);

                if is_viable(&new_candidate, &word_boundaries, self.trie) {
                    if !new_candidate.contents.contains(' ') {
                        return Ok(new_candidate);
                    }
                    let orderable = FrequencyOrderableCrossword::new(new_candidate, self.bigrams);
                    if orderable.fillability_score > 0 {
                        crossword_fill_state.add_candidate(orderable);
                    }
                }
            }
        }
    }
}
