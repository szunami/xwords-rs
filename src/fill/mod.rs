use crate::{
    crossword::{CrosswordWordIterator, Direction},
    fill::cache::CachedIsViable,
    order::FrequencyOrderableCrossword,
    parse::WordBoundary,
    trie::Trie,
    Crossword, FxHashMap,
};

use core::hash::{BuildHasherDefault, Hash};
use rustc_hash::{FxHashSet, FxHasher};
use std::{collections, collections::BinaryHeap, hash::Hasher};

pub mod cache;
pub mod parallel;
pub mod simple;
pub mod single_threaded;

pub trait Filler {
    fn fill(&mut self, crossword: &Crossword) -> Result<Crossword, String>;
}

struct CrosswordFillState {
    candidate_queue: BinaryHeap<FrequencyOrderableCrossword>,
}

impl CrosswordFillState {
    pub fn default() -> CrosswordFillState {
        CrosswordFillState {
            candidate_queue: BinaryHeap::new(),
        }
    }

    pub fn take_candidate(&mut self) -> Option<Crossword> {
        self.candidate_queue.pop().map(|x| x.crossword)
    }

    pub fn add_candidate(&mut self, candidate: FrequencyOrderableCrossword) {
        self.candidate_queue.push(candidate);
    }
}

pub fn is_viable_reuse(
    candidate: &Crossword,
    word_boundaries: &Vec<&WordBoundary>,
    trie: &Trie,
    mut already_used: FxHashSet<u64>,
    is_viable_cache: &mut CachedIsViable,
) -> (bool, FxHashSet<u64>) {
    for word_boundary in word_boundaries {
        let iter = CrosswordWordIterator::new(candidate, word_boundary);

        let mut hasher = FxHasher::default();
        let mut full = true;
        for c in iter.clone() {
            c.hash(&mut hasher);
            full = full && c != ' ';
        }
        let key = hasher.finish();

        if full && already_used.contains(&key) {
            return (false, already_used);
        }
        already_used.insert(key);

        if !is_viable_cache.is_viable(iter, trie) {
            return (false, already_used);
        }
    }
    (true, already_used)
}

pub fn fill_one_word(
    candidate: &Crossword,
    iter: &CrosswordWordIterator,
    word: &String,
) -> Crossword {
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
    result_contents = String::from_utf8(bytes).expect("Non utf8 char");

    Crossword {
        contents: result_contents,
        ..*candidate
    }
}

pub fn build_lookup<'s>(
    word_boundaries: &'s Vec<WordBoundary>,
) -> FxHashMap<(Direction, usize, usize), &'s WordBoundary> {
    let mut result = FxHashMap::default();

    for word_boundary in word_boundaries {
        match word_boundary.direction {
            Direction::Across => {
                for index in 0..word_boundary.length {
                    let col = word_boundary.start_col + index;

                    result.insert(
                        (Direction::Across, word_boundary.start_row, col),
                        word_boundary,
                    );
                }
            }
            Direction::Down => {
                for index in 0..word_boundary.length {
                    let row = word_boundary.start_row + index;

                    result.insert(
                        (Direction::Down, row, word_boundary.start_col),
                        word_boundary,
                    );
                }
            }
        }
    }

    result
}

pub fn orthogonals<'s>(
    to_fill: &'s WordBoundary,
    word_boundary_lookup: &collections::HashMap<
        (Direction, usize, usize),
        &'s WordBoundary,
        BuildHasherDefault<FxHasher>,
    >,
) -> Vec<&'s WordBoundary> {
    // TODO: avoid allocating here
    let mut result = Vec::with_capacity(to_fill.length);

    match to_fill.direction {
        Direction::Across => {
            for index in 0..to_fill.length {
                let col = to_fill.start_col + index;

                result.push(
                    *word_boundary_lookup
                        .get(&(Direction::Down, to_fill.start_row, col))
                        .unwrap(),
                );
            }
        }
        Direction::Down => {
            for index in 0..to_fill.length {
                let row = to_fill.start_row + index;

                result.push(
                    *word_boundary_lookup
                        .get(&(Direction::Across, row, to_fill.start_col))
                        .unwrap(),
                );
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::{
        crossword::Direction, fill::CrosswordWordIterator, parse::WordBoundary, Crossword,
    };

    use super::fill_one_word;

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
                &CrosswordWordIterator::new(
                    &c,
                    &WordBoundary {
                        start_col: 0,
                        start_row: 0,
                        length: 3,
                        direction: Direction::Across,
                    },
                ),
                &String::from("cat")
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
                &CrosswordWordIterator::new(
                    &c,
                    &WordBoundary {
                        start_col: 0,
                        start_row: 0,
                        length: 3,
                        direction: Direction::Down,
                    }
                ),
                &String::from("cat"),
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
}
