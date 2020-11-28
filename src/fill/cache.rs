use std::hash::{Hash, Hasher};

use rustc_hash::{FxHashMap, FxHasher};

use crate::trie::Trie;

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

    pub fn words<T: Iterator<Item = char> + Clone>(
        &mut self,
        iter: T,
        trie: &Trie,
    ) -> &Vec<String> {
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
pub struct CachedIsViable {
    is_viable_cache: FxHashMap<u64, bool>,
}

impl CachedIsViable {
    pub fn default() -> CachedIsViable {
        CachedIsViable {
            is_viable_cache: FxHashMap::default(),
        }
    }

    pub fn is_viable<T: Iterator<Item = char> + Clone>(&mut self, iter: T, trie: &Trie) -> bool {
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
