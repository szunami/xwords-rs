use crate::{
    crossword::{CrosswordWordIterator, Direction},
    order::FrequencyOrderableCrossword,
    parse::WordBoundary,
    trie::Trie,
    Crossword, FxHashMap,
};
use cached::Cached;
use core::hash::Hash;
use fxhash::{FxHashSet, FxHasher};
use std::{collections::BinaryHeap, hash::Hasher};

pub mod parallel;
pub mod single_threaded;

pub trait Filler {
    fn fill(&self, crossword: &Crossword) -> Result<Crossword, String>;
}

struct CrosswordFillState {
    candidate_queue: BinaryHeap<FrequencyOrderableCrossword>,
    done: bool,
}

impl CrosswordFillState {
    pub fn default() -> CrosswordFillState {
        CrosswordFillState {
            candidate_queue: BinaryHeap::new(),
            done: false,
        }
    }

    pub fn take_candidate(&mut self) -> Option<Crossword> {
        self.candidate_queue.pop().map(|x| x.crossword)
    }

    pub fn add_candidate(&mut self, candidate: FrequencyOrderableCrossword) {
        self.candidate_queue.push(candidate);
    }

    pub fn mark_done(&mut self) {
        self.done = true;
    }
}

pub fn is_viable(candidate: &Crossword, word_boundaries: &[WordBoundary], trie: &Trie) -> bool {
    let mut already_used = FxHashSet::default();

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

pub struct FxCache<K: Hash + Eq, V> {
    store: FxHashMap<K, V>,
}
impl<K: Hash + Eq, V> FxCache<K, V> {
    pub fn default() -> FxCache<K, V> {
        FxCache {
            store: FxHashMap::default(),
        }
    }
}
impl<K: Hash + Eq, V> Cached<K, V> for FxCache<K, V> {
    fn cache_get(&mut self, k: &K) -> Option<&V> {
        self.store.get(k)
    }
    fn cache_get_mut(&mut self, k: &K) -> Option<&mut V> {
        self.store.get_mut(k)
    }
    fn cache_get_or_set_with<F: FnOnce() -> V>(&mut self, k: K, f: F) -> &mut V {
        self.store.entry(k).or_insert_with(f)
    }
    fn cache_set(&mut self, k: K, v: V) -> Option<V> {
        self.store.insert(k, v)
    }
    fn cache_remove(&mut self, k: &K) -> Option<V> {
        self.store.remove(k)
    }
    fn cache_clear(&mut self) {
        self.store.clear();
    }
    fn cache_reset(&mut self) {
        self.store = FxHashMap::default();
    }
    fn cache_size(&self) -> usize {
        self.store.len()
    }
}

pub fn is_word(iter: CrosswordWordIterator, trie: &Trie) -> bool {
    is_word_internal(iter, trie)
}

cached_key! {
    IS_WORD: FxCache<u64, bool> = FxCache::default();
    Key = {
        use std::hash::Hash;
        let mut hasher = FxHasher::default();
        for c in iter.clone() {
            c.hash(&mut hasher)
        }
        hasher.finish()
    };
    fn is_word_internal(iter: CrosswordWordIterator, trie: &Trie) -> bool = {
        trie.is_word(iter)
    }
}

pub fn words(pattern: String, trie: &Trie) -> Vec<String> {
    words_internal(pattern, trie)
}

cached_key! {
    WORDS: FxCache<String, Vec<String>> = FxCache::default();
    Key = { pattern.clone() };
    fn words_internal(pattern: String, trie: &Trie) -> Vec<String> = {
        trie.words(pattern)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        crossword::Direction,
        default_indexes,
        fill::{is_word, CrosswordWordIterator},
        parse::{parse_word_boundaries, WordBoundary},
        Crossword, Trie,
    };

    use super::{fill_one_word, is_viable};

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
