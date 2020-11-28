use std::{collections::hash_map::Entry, hash::{Hash, Hasher}};

use rustc_hash::{FxHashMap, FxHasher};

use crate::{crossword::CrosswordWordIterator, trie::Trie};

#[derive(Clone)]
pub struct CachedWords {
    words_cache: FxHashMap<u64, Vec<String>>,
    pub hits: usize,
    pub misses: usize,
}

impl CachedWords {
    pub fn default() -> CachedWords {
        CachedWords {
            words_cache: FxHashMap::default(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn words(&mut self, iter: CrosswordWordIterator, trie: &Trie) -> &Vec<String> {
        let mut hasher = FxHasher::default();
        for c in iter.clone() {
            c.hash(&mut hasher);
        }
        let key = hasher.finish();
        
        match self.words_cache.entry(key) {
            Entry::Occupied(result) => {
                self.hits += 1;
                result.into_mut()
            }
            Entry::Vacant(vacant) => {
                self.misses += 1;
                vacant.insert(trie.words(iter))
            }
        }
    }
}

#[derive(Clone)]
pub struct CachedIsViable {
    is_viable_cache: FxHashMap<u64, bool>,
}

impl CachedIsViable {
    pub fn new() -> CachedIsViable {
        CachedIsViable {
            is_viable_cache: FxHashMap::default(),
        }
    }

    pub fn is_viable(&mut self, iter: CrosswordWordIterator, trie: &Trie) -> bool {
        let mut hasher = FxHasher::default();
        for c in iter.clone() {
            c.hash(&mut hasher);
        }
        let key = hasher.finish();
        
        
        
        

        *self
            .is_viable_cache
            .entry(key)
            .or_insert_with(|| trie.is_viable(iter))
    }
}
