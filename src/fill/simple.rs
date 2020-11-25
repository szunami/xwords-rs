use std::time::Instant;

use crate::{
    crossword::{Crossword, CrosswordWordIterator},
    parse::parse_word_boundaries,
    trie::Trie,
};

use super::{fill_one_word, is_viable, words, Filler};

pub struct SimpleFiller<'s> {
    trie: &'s Trie,
}

impl<'s> SimpleFiller<'s> {
    pub fn new(trie: &'s Trie) -> SimpleFiller<'s> {
        SimpleFiller { trie }
    }
}

impl<'s> Filler for SimpleFiller<'s> {
    fn fill(&mut self, initial_crossword: &Crossword) -> Result<Crossword, String> {
        let thread_start = Instant::now();
        let mut candidate_count = 0;

        let word_boundaries = parse_word_boundaries(&initial_crossword);

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

            let potential_fills = words(to_fill.clone(), self.trie);

            for potential_fill in potential_fills {
                let new_candidate = fill_one_word(&candidate, &to_fill.clone(), &potential_fill);

                if is_viable(&new_candidate, &word_boundaries, self.trie) {
                    if !new_candidate.contents.contains(' ') {
                        return Ok(new_candidate);
                    }
                    candidates.push(new_candidate);
                }
            }
        }

        Err(String::from("We failed"))
    }
}

#[cfg(test)]
mod tests {

    use crate::{default_indexes, fill::Filler};

    use crate::Crossword;

    use std::{cmp::Ordering, time::Instant};

    use super::SimpleFiller;

    #[test]
    fn test() {
        assert_eq!((1, 2).cmp(&(3, 4)), Ordering::Less)
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
