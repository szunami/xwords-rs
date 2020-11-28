use crate::{
    crossword::{CrosswordWordIterator, Direction},
    fill::cache::CachedIsViable,
    parse::WordBoundary,
    trie::Trie,
    Crossword, FxHashMap,
};

use core::hash::{BuildHasherDefault, Hash};
use rustc_hash::{FxHashSet, FxHasher};
use std::{collections, hash::Hasher};

pub mod cache;
pub mod simple;

pub trait Filler {
    fn fill(&mut self, crossword: &Crossword) -> Result<Crossword, String>;
}

pub fn is_viable_reuse(
    candidate: &Crossword,
    word_boundaries: &[&WordBoundary],
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
    word: &str,
) -> Crossword {
    let mut result_contents = String::with_capacity(iter.word_boundary.length);
    let word_boundary = iter.word_boundary;
    let mut word_iter = word.chars();

    match word_boundary.direction {
        Direction::Across => {
            for (index, c) in candidate.contents.chars().enumerate() {
                let row = index / candidate.width;
                let col = index % candidate.width;

                if row == word_boundary.start_row
                    && col >= word_boundary.start_col
                    && col < word_boundary.start_col + word_boundary.length
                {
                    result_contents.push(word_iter.next().unwrap());
                } else {
                    result_contents.push(c);
                }
            }
        }
        Direction::Down => {
            for (index, c) in candidate.contents.chars().enumerate() {
                let row = index / candidate.width;
                let col = index % candidate.width;

                if col == word_boundary.start_col
                    && row >= word_boundary.start_row
                    && row < word_boundary.start_row + word_boundary.length
                {
                    result_contents.push(word_iter.next().unwrap());
                } else {
                    result_contents.push(c);
                }
            }
        }
    }

    Crossword {
        contents: result_contents,
        ..*candidate
    }
}

pub fn build_lookup<'s>(
    word_boundaries: &'s[WordBoundary],
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
        let c = Crossword::square(String::from(
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
            Crossword::square(String::from(
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
            Crossword::square(String::from(
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
