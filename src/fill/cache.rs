use std::hash::{Hash, Hasher};

use rustc_hash::{FxHashMap, FxHasher};

use crate::{crossword::CrosswordWordIterator, trie::Trie};

#[derive(Clone)]
pub struct CachedWords {
    words_cache: FxHashMap<u64, Vec<String>>,
}

impl CachedWords {
    
    pub fn new() -> CachedWords {
        CachedWords{
            words_cache: FxHashMap::default(),
        }
    }
    
    pub fn words(&mut self, iter: CrosswordWordIterator, trie: &Trie) -> &Vec<String> {
        let mut hasher = FxHasher::default();
        for c in iter.clone() {
            c.hash(&mut hasher);
        }
        let key = hasher.finish();
        
        self.words_cache.entry(key).or_insert_with(|| trie.words(iter))
    }
}

#[derive(Clone)]
pub struct CachedIsViable {
    is_word_cache: FxHashMap<u64, bool>,
}

impl CachedIsViable {
    
    pub fn new() -> CachedIsViable {
        CachedIsViable{
            is_word_cache: FxHashMap::default(),
        }
    }
    
    #[inline(always)]
    pub fn is_viable(&mut self, iter: CrosswordWordIterator, key: u64, trie: &Trie) -> bool {
        *self.is_word_cache.entry(key).or_insert_with(|| trie.is_viable(iter))
    }
}