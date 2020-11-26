use std::{collections::HashSet, hash::BuildHasherDefault, time::Instant};

use rustc_hash::FxHasher;

use crate::{
    crossword::{Crossword, CrosswordWordIterator},
    parse::parse_word_boundaries,
    trie::Trie,
};

use super::{Filler, cache::{CachedIsViable, CachedWords}, fill_one_word, is_viable_reuse};

pub struct SimpleFiller<'s> {
    word_cache: CachedWords,
    is_word_cache: CachedIsViable,
    
    trie: &'s Trie,
}

impl<'s> SimpleFiller<'s> {
    pub fn new(trie: &'s Trie) -> SimpleFiller<'s> {
        SimpleFiller { 
            word_cache: CachedWords::new(),
            is_word_cache: CachedIsViable::new(),
            trie }
    }
}

impl<'s> Filler for SimpleFiller<'s> {
    fn fill(&mut self, initial_crossword: &Crossword) -> Result<Crossword, String> {
        let thread_start = Instant::now();
        let mut candidate_count = 0;

        let word_boundaries = parse_word_boundaries(&initial_crossword);
        let mut already_used = HashSet::with_capacity_and_hasher(
            word_boundaries.len(),
            BuildHasherDefault::<FxHasher>::default(),
        );

        let mut candidates = vec![initial_crossword.to_owned()];

        while !candidates.is_empty() {
            let candidate = candidates.pop().unwrap();
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
                .max_by_key(|iter| {
                    // choose a long word with few spaces

                    let mut length = 0;
                    let mut space_count = 0;

                    for c in iter.clone() {
                        if c == ' ' {
                            space_count += 1;
                        }
                        length += 1;
                    }

                    (length, -space_count)
                })
                .unwrap();
                // println!("{}", candidate);
                // println!("{:?}", to_fill.clone().to_string());

                let potential_fills = self.word_cache.words(to_fill.clone(), self.trie);

            for potential_fill in potential_fills {
                let new_candidate = fill_one_word(&candidate, &to_fill.clone(), &potential_fill);

                // if is_viable_tmp(&new_candidate, &word_boundaries, self.trie, &mut self.is_word_cache) {
                let (viable, tmp) =
                    is_viable_reuse(&new_candidate, &word_boundaries, self.trie, already_used, &mut self.is_word_cache);
                already_used = tmp;
                already_used.clear();

                if viable {
                    if !new_candidate.contents.contains(' ') {
                        println!("Evaluated {} candidates", candidate_count);
                        return Ok(new_candidate);
                    }
                    // println!("Inserting:");
                    // println!("{}", new_candidate);
                    candidates.push(new_candidate);
                }
            }
        }

        Err(String::from("We failed"))
    }
}

#[cfg(test)]
mod tests {

    use cached::Cached;
    use rustc_hash::FxHashSet;

    use crate::{fill::{cache::CachedIsViable, is_viable_reuse}, parse::parse_word_boundaries};
use crate::{crossword::CrosswordWordIterator, parse::WordBoundary, default_indexes, fill::Filler};

    use crate::Crossword;

    use std::{cmp::Ordering, time::Instant};

    use super::SimpleFiller;

    #[test]
    fn test() {
        assert_eq!((1, 2).cmp(&(3, 4)), Ordering::Less)
    }

    #[test]
    fn weird() {
        let puz = Crossword::new(String::from("
   C***
   I***
   C***
CIGARET
***D LC
***A IB
***SIZY
")).unwrap();

        let (_bigrams, trie) = default_indexes();
        let word_boundaries = parse_word_boundaries(&puz);
        
        let mut is_viable = CachedIsViable::new();
        
        let (viable, _) = is_viable_reuse(&puz, &word_boundaries, &trie, FxHashSet::default(), &mut is_viable);
        
        assert!(!viable);
        
        // let word_boundary = WordBoundary::new(
        //     4, 3, 4, crate::crossword::Direction::Across
        // );

        // let iter = CrosswordWordIterator::new(
        //     &puz,
        //     &word_boundary
        // );
        
        // assert_eq!(iter.clone().to_string(), String::from("D LC"));
        // assert!(trie.is_viable(iter));
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
        let (_bigrams, trie) = default_indexes();
        let mut filler = SimpleFiller::new(&trie);
        let filled_puz =  filler.fill(&grid).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }
}
