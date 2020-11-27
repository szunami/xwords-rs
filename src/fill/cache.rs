use std::hash::{Hash, Hasher};

use rustc_hash::{FxHashMap, FxHasher};

use crate::{crossword::CrosswordWordIterator, trie::Trie};

// #[derive(Clone)]
// pub struct CachedWordSet {
//     words_cache: FxHashMap<u64, Vec<String>>,
//     is_word_cache: FxHashMap<u64, bool>,
// }

// impl CachedWordSet {

//     pub fn new() -> CachedWordSet {
//         CachedWordSet{
//             words_cache: FxHashMap::default(),
//             is_word_cache: FxHashMap::default(),
//         }
//     }

//     pub fn words(&mut self, iter: CrosswordWordIterator, trie: &Trie) -> &Vec<String> {
//         let mut hasher = FxHasher::default();
//         for c in iter.clone() {
//             c.hash(&mut hasher);
//         }
//         let key = hasher.finish();

//         self.words_cache.entry(key).or_insert_with(|| trie.words(iter))
//     }

//     pub fn is_word(&mut self, iter: CrosswordWordIterator, trie: &Trie) -> bool {
//         let mut hasher = FxHasher::default();
//         for c in iter.clone() {
//             c.hash(&mut hasher);
//         }
//         let key = hasher.finish();

//         *self.is_word_cache.entry(key).or_insert_with(|| trie.is_word(iter))
//     }
// }

#[derive(Clone)]
pub struct CachedWords {
    words_cache: FxHashMap<u64, Vec<String>>,
}

impl CachedWords {
    pub fn default() -> CachedWords {
        CachedWords {
            words_cache: FxHashMap::default(),
        }
    }

    pub fn words(&mut self, iter: CrosswordWordIterator, trie: &Trie) -> &Vec<String> {
        let mut hasher = FxHasher::default();
        for c in iter.clone() {
            c.hash(&mut hasher);
        }
        let key = hasher.finish();

        self.words_cache
            .entry(key)
            .or_insert_with(|| trie.words(iter))
    }
}

#[derive(Clone)]
pub struct CachedIsWord {
    is_word_cache: FxHashMap<u64, bool>,
}

impl CachedIsWord {
    pub fn new() -> CachedIsWord {
        CachedIsWord {
            is_word_cache: FxHashMap::default(),
        }
    }

    pub fn is_word(&mut self, iter: CrosswordWordIterator, trie: &Trie) -> bool {
        let mut hasher = FxHasher::default();
        for c in iter.clone() {
            c.hash(&mut hasher);
        }
        let key = hasher.finish();

        *self
            .is_word_cache
            .entry(key)
            .or_insert_with(|| trie.is_word(iter))
    }
}
