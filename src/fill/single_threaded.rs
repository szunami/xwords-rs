use crate::parse::WordBoundary;
use crate::{
    crossword::Direction, fill::cache::CachedIsViable,
};
use crate::{
    fill::{fill_one_word, CrosswordFillState},
    order::FrequencyOrderableCrossword,
    Filler,
};
use std::{collections::HashSet, hash::BuildHasherDefault};

use rustc_hash::{FxHashMap, FxHasher};

use crate::{crossword::CrosswordWordIterator, order::score_iter, parse::parse_word_boundaries};
use std::time::Instant;

use crate::{trie::Trie, Crossword};

use super::cache::CachedWords;
use super::is_viable_reuse;

#[derive(Clone)]
pub struct SingleThreadedFiller<'s> {
    word_cache: CachedWords,
    is_word_cache: CachedIsViable,

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
            is_word_cache: CachedIsViable::new(),
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
        let (down_lookup, across_lookup) = build_lookup(&word_boundaries);

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
                .min_by_key(|iter| score_iter(iter, self.bigrams))
                .unwrap();

            let orthogonals = match to_fill.word_boundary.direction {
                Across => {
                    orthogonals(to_fill.word_boundary, &down_lookup)
                }
                Down => {
                    orthogonals(to_fill.word_boundary, &across_lookup)

                }
            };
            // let orthogonals = orthogonals(to_fill.word_boundary, &word_boundary_lookup);
            // find valid fills for word;
            // for each fill:
            //   are all complete words legit?
            //     if so, push

            // let potential_fills = words(to_fill.clone(), self.trie);

            // let affected_words = orthogonals(to_fill, lookup)

            let potential_fills = self.word_cache.words(to_fill.clone(), self.trie);

            for potential_fill in potential_fills {
                let new_candidate = fill_one_word(&candidate, &to_fill.clone(), potential_fill);

                // if is_viable_tmp(&new_candidate, &word_boundaries, self.trie, &mut self.is_word_cache) {
                let (viable, tmp) = is_viable_reuse(
                    &new_candidate,
                    &orthogonals,
                    self.trie,
                    already_used,
                    &mut self.is_word_cache,
                );
                already_used = tmp;
                already_used.clear();

                if viable {
                    if !new_candidate.contents.contains(' ') {
                        println!("Evaluated {} candidates", candidate_count);
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

pub fn orthogonals<'s>(
    to_fill: &'s WordBoundary,
    word_boundary_lookup: &std::collections::HashMap<
        (usize, usize),
        &'s WordBoundary,
        BuildHasherDefault<FxHasher>,
    >,
) -> Vec<&'s WordBoundary> {
    // TODO: avoid allocating here
    let mut result = Vec::with_capacity(to_fill.length);

    match to_fill.direction {
        Across => {
            for index in 0..to_fill.length {
                let col = to_fill.start_col + index;

                result.push(*word_boundary_lookup.get(&(to_fill.start_row, col)).unwrap());
            }
        }
        Down => {
            for index in 0..to_fill.length {
                let row = to_fill.start_row + index;

                result.push(*word_boundary_lookup.get(&(row, to_fill.start_col)).unwrap());
            }
        }
    }

    result
}

pub fn build_lookup<'s>(
    word_boundaries: &'s Vec<WordBoundary>,
) -> (
    FxHashMap<(usize, usize), &'s WordBoundary>,
    FxHashMap<(usize, usize), &'s WordBoundary>,
    ) {
    let mut down_result = FxHashMap::default();
    let mut across_result = FxHashMap::default();

    for word_boundary in word_boundaries {
        match word_boundary.direction {
            Across => {
                for index in 0..word_boundary.length {
                    let col = word_boundary.start_col + index;

                    across_result.insert((word_boundary.start_row, col), word_boundary);
                }
            }
            Down => {
                for index in 0..word_boundary.length {
                    let row = word_boundary.start_row + index;

                    down_result.insert((row, word_boundary.start_col), word_boundary);
                }
            }
        }
    }

    (down_result, across_result)
}

// pub fn is_viable_tmp(candidate: &Crossword, word_boundaries: &[WordBoundary], trie: &Trie,
//     is_word_cache: &mut CachedIsViable) -> bool {
//     let mut already_used = HashSet::with_capacity_and_hasher(
//         word_boundaries.len(),
//         BuildHasherDefault::<FxHasher>::default(),
//     );

//     for word_boundary in word_boundaries {
//         let iter = CrosswordWordIterator::new(candidate, word_boundary);
//         if iter.clone().any(|c| c == ' ') {
//             if !is_word_cache.is_viable(iter, trie) {
//                 return false;
//             }
//         } else {
//             if already_used.contains(&iter) {
//                 return false;
//             }
//             already_used.insert(iter.clone());

//             if !is_word_cache.is_viable(iter, trie) {
//                 return false;
//             }
//         }
//     }
//     true
// }

#[cfg(test)]
mod tests {

    use crate::{crossword::Direction, parse::parse_word_boundaries};
use crate::parse::WordBoundary;
use crate::{default_indexes, fill::Filler};

    use crate::Crossword;

    use std::time::Instant;

    use super::{SingleThreadedFiller, build_lookup, orthogonals};

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
    
    #[test]
    fn build_lookup_works() {
        let input = std::fs::read_to_string("./grids/empty_4x4.txt").expect("failed to read input");
        let input = Crossword::new(input).expect("failed to parse input");
    
        let word_boundaries = parse_word_boundaries(&input);
        
        let (down_lookup, across_lookup) = build_lookup(&word_boundaries);
        
        assert_eq!(16, down_lookup.len());
        assert_eq!(16, across_lookup.len());
        
        let word_boundary = WordBoundary::new(
            0, 0, 4, Direction::Across
        );
        
        let orthogonals = orthogonals(
            &word_boundary, &down_lookup);
            
        assert_eq!(orthogonals.len(), 4);
            
        assert_eq!(**orthogonals.first().unwrap(),
            WordBoundary::new(
                0, 0, 4, Direction::Down
            )
        );
            
    }
}
