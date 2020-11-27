use crate::{
    fill::{cache::CachedIsWord, fill_one_word, CrosswordFillState},
    order::FrequencyOrderableCrossword,
    parse::WordBoundary,
    Filler,
};
use std::{collections::HashSet, hash::BuildHasherDefault};

use rustc_hash::{FxHashMap, FxHasher};

use crate::{crossword::CrosswordWordIterator, order::score_iter, parse::parse_word_boundaries};
use std::time::Instant;

use crate::{trie::Trie, Crossword};

use super::{cache::CachedWords, is_viable_reuse};

#[derive(Clone)]
pub struct SingleThreadedFiller<'s> {
    word_cache: CachedWords,
    is_word_cache: CachedIsWord,

    trie: &'s Trie,
    bigrams: &'s FxHashMap<(char, char), usize>,
}

impl<'s> SingleThreadedFiller<'s> {
    pub fn new(
        trie: &'s Trie,
        bigrams: &'s FxHashMap<(char, char), usize>,
    ) -> SingleThreadedFiller<'s> {
        SingleThreadedFiller {
            word_cache: CachedWords::new(),
            is_word_cache: CachedIsWord::new(),
            trie,
            bigrams,
        }
    }
}

impl<'s> Filler for SingleThreadedFiller<'s> {
    fn fill(&mut self, crossword: &Crossword) -> std::result::Result<Crossword, String> {
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
        let mut already_used = HashSet::with_capacity_and_hasher(
            word_boundaries.len(),
            BuildHasherDefault::<FxHasher>::default(),
        );
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
                .min_by_key(|iter| self.word_cache.words(iter.clone(), self.trie).len())
                .unwrap();
            // find valid fills for word;
            // for each fill:
            //   are all complete words legit?
            //     if so, push

            // let potential_fills = words(to_fill.clone(), self.trie);

            let potential_fills = self.word_cache.words(to_fill.clone(), self.trie);

            for potential_fill in potential_fills {
                let new_candidate = fill_one_word(&candidate, &to_fill.clone(), potential_fill);

                // if is_viable_tmp(&new_candidate, &word_boundaries, self.trie, &mut self.is_word_cache) {
                let (viable, tmp) = is_viable_reuse(
                    &new_candidate,
                    &word_boundaries,
                    self.trie,
                    already_used,
                    &mut self.is_word_cache,
                );
                already_used = tmp;
                already_used.clear();

                if viable {
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

pub fn is_viable_tmp(
    candidate: &Crossword,
    word_boundaries: &[WordBoundary],
    trie: &Trie,
    is_word_cache: &mut CachedIsWord,
) -> bool {
    let mut already_used = HashSet::with_capacity_and_hasher(
        word_boundaries.len(),
        BuildHasherDefault::<FxHasher>::default(),
    );

    for word_boundary in word_boundaries {
        let iter = CrosswordWordIterator::new(candidate, word_boundary);
        if iter.clone().any(|c| c == ' ') {
            continue;
        }

        if already_used.contains(&iter) {
            return false;
        }
        already_used.insert(iter.clone());

        if !is_word_cache.is_word(iter, trie) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {

    use crate::{default_indexes, fill::Filler};

    use crate::Crossword;

    use std::time::Instant;

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
        let mut filler = SingleThreadedFiller::new(&trie, &bigrams);
        let filled_puz = filler.fill(&grid).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }
}
