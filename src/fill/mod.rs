use crate::crossword::{CrosswordWordIterator, Direction};
use crate::parse::WordBoundary;
use crate::Crossword;
use crate::Trie;
use cached::SizedCache;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;

pub mod mpmc;
pub mod parallel;
pub mod single_threaded;

pub trait Filler {
    fn fill(self, crossword: &Crossword) -> Result<Crossword, String>;
}

pub fn fill_one_word(
    candidate: &Crossword,
    iter: &CrosswordWordIterator,
    word: String,
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
    unsafe { result_contents = String::from_utf8_unchecked(bytes) }

    Crossword {
        contents: result_contents,
        ..*candidate
    }
}

cached_key! {
    IS_WORD: SizedCache<u64, bool> = SizedCache::with_size(10_000);
    Key = {
        use std::hash::Hash;
        let mut hasher = DefaultHasher::new();
        for c in iter.clone() {
            c.hash(&mut hasher)
        }

        hasher.finish()
    };
    fn is_word(iter: CrosswordWordIterator, trie: &Trie) -> bool = {
        trie.is_word(iter)
    }
}

cached_key! {
    WORDS: SizedCache<String, Vec<String>> = SizedCache::with_size(10_000);
    Key = { pattern.clone() };
    fn words(pattern: String, trie: &Trie) -> Vec<String> = {
        trie.words(pattern)
    }
}

pub fn is_viable(candidate: &Crossword, word_boundaries: &[WordBoundary], trie: &Trie) -> bool {
    let mut already_used = HashSet::with_capacity(word_boundaries.len());

    for word_boundary in word_boundaries {
        let iter = CrosswordWordIterator::new(candidate, word_boundary);
        if iter.clone().any(|c| c == ' ') {
            continue;
        }

        if already_used.contains(&iter) {
            return false;
        }
        already_used.insert(iter.clone());

        if !is_word(iter, trie) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {

    use crate::default_indexes;
    use crate::fill::fill_one_word;
    use crate::fill::is_viable;
    use crate::fill::is_word;
    use crate::fill::CrosswordWordIterator;
    use crate::parse::parse_word_boundaries;
    use crate::parse::WordBoundary;
    use crate::Crossword;
    use crate::{crossword::Direction, Trie};

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
                String::from("cat")
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
                String::from("cat"),
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

    #[test]
    fn is_viable_works() {
        let (_, trie) = default_indexes();

        let crossword = Crossword::new(String::from(
            "
   
   
   
",
        ))
        .unwrap();

        let word_boundaries = parse_word_boundaries(&crossword);

        assert!(is_viable(&crossword, &word_boundaries, &trie));

        assert!(!is_viable(
            &Crossword::new(String::from("ABCDEFGH ")).unwrap(),
            &word_boundaries,
            &trie
        ));

        assert!(!is_viable(
            &Crossword::new(String::from("ABCB  C  ")).unwrap(),
            &word_boundaries,
            &trie
        ));
    }

    #[test]
    fn cache_works() {
        let trie = Trie::build(vec![
            String::from("bass"),
            String::from("bats"),
            String::from("bess"),
            String::from("be"),
        ]);

        let crossword = Crossword::new(String::from(
            "
bass
a   
s   
s   
",
        ))
        .unwrap();
        let word_boundary = WordBoundary {
            start_row: 0,
            start_col: 0,
            direction: Direction::Across,
            length: 4,
        };
        let iter = CrosswordWordIterator::new(&crossword, &word_boundary);

        assert!(is_word(iter, &trie));

        let word_boundary = WordBoundary {
            start_row: 0,
            start_col: 0,
            direction: Direction::Down,
            length: 4,
        };
        let iter = CrosswordWordIterator::new(&crossword, &word_boundary);

        assert!(is_word(iter, &trie));
    }
}
