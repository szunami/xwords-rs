use crate::{
    fill::{cache::CachedIsViable, fill_one_word, CrosswordFillState},
    order::FrequencyOrderableCrossword,
    Filler,
};
use std::{collections::HashSet, hash::BuildHasherDefault};

use rustc_hash::{FxHashMap, FxHasher};

use crate::{crossword::CrosswordWordIterator, parse::parse_word_boundaries};
use std::time::Instant;

use crate::{trie::Trie, Crossword};

use super::{build_lookup, cache::CachedWords, is_viable_reuse, orthogonals};

#[derive(Clone)]
pub struct SingleThreadedFiller<'s> {
    word_cache: CachedWords,
    is_viable_cache: CachedIsViable,

    trie: &'s Trie,
    bigrams: &'s FxHashMap<(char, char), usize>,
}

impl<'s> SingleThreadedFiller<'s> {
    pub fn new(
        trie: &'s Trie,
        bigrams: &'s FxHashMap<(char, char), usize>,
    ) -> SingleThreadedFiller<'s> {
        SingleThreadedFiller {
            word_cache: CachedWords::default(),
            is_viable_cache: CachedIsViable::new(),
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
        let word_boundary_lookup = build_lookup(&word_boundaries);

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

            let orthogonals = orthogonals(&to_fill.word_boundary, &word_boundary_lookup);

            let potential_fills = self.word_cache.words(to_fill.clone(), self.trie);

            for potential_fill in potential_fills {
                let new_candidate = fill_one_word(&candidate, &to_fill.clone(), potential_fill);

                // if is_viable_tmp(&new_candidate, &word_boundaries, self.trie, &mut self.is_viable_cache) {
                let (viable, tmp) = is_viable_reuse(
                    &new_candidate,
                    &orthogonals,
                    self.trie,
                    already_used,
                    &mut self.is_viable_cache,
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
