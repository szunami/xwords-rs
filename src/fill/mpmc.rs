use crate::Trie;
use crate::fill::is_viable;
use crate::fill::fill_one_word;
use crate::fill::words;
use std::{collections::HashMap, sync::{Arc, mpsc}};

use crate::order::score_iter;
use crate::{crossword::CrosswordWordIterator, parse::parse_word_boundaries};
use crossbeam::channel::unbounded;

use crate::{crossword::Crossword, Filler};
pub struct MPMCFiller {
    trie: Arc<Trie>,
    bigrams: Arc<HashMap<(char, char), usize>>,
}

impl Filler for MPMCFiller {
    fn fill(self, initial_crossword: &Crossword) -> std::result::Result<Crossword, String> {
        // let q = SegQueue::new();
        // q.push(initial_crossword.clone());
        
        let (s, r) = unbounded::<Crossword>();
        
        let (tx, rx) = mpsc::channel();

        
        let trie = self.trie.clone();
        let bigrams = self.bigrams.clone();
        
        for thread_index in 0..2 {
            
            let (s_i, r_i) = (s.clone(), r.clone());
            let word_boundaries = parse_word_boundaries(&initial_crossword);
            let trie = trie.clone();
            let bigrams = bigrams.clone();
            let tx = tx.clone();

            std::thread::Builder::new()
                .name(format!("worker{}", thread_index))
                .spawn(move || {
                    loop {
                        if let Ok(candidate) = r_i.recv() {
                            
                            let to_fill = word_boundaries
                            .iter()
                            .map(|word_boundary| {
                                CrosswordWordIterator::new(&candidate, word_boundary)
                            })
                            .filter(|iter| iter.clone().any(|c| c == ' '))
                            .min_by_key(|iter| score_iter(iter, bigrams.as_ref()))
                            .unwrap();
                            
                            let potential_fills = words(to_fill.clone().to_string(), trie.as_ref());
                            for potential_fill in potential_fills {
                                let new_candidate =
                                    fill_one_word(&candidate, &to_fill.clone(), potential_fill);
                                
                                if is_viable(&new_candidate, &word_boundaries, trie.as_ref()) {
                                    
                                    if !new_candidate.contents.contains(' '){
                                        tx.send(new_candidate);
                                    } else {
                                        s_i.send(new_candidate);
                                    }
                                }
                            }
                        }
                    }
                }).unwrap();
        }
        
        s.send(initial_crossword.clone());
        
        match rx.recv() {
            Ok(result) => Ok(result),
            Err(_) => Err(String::from("Failed to receive")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::MPMCFiller;
    use crate::{Crossword, fill::Filler, default_indexes};

    #[test]
    fn fill_crossword_works() {
        let (bigrams, trie) = default_indexes();
        let filler = MPMCFiller {
            trie:  Arc::new(trie),
            bigrams: Arc::new(bigrams),
        };

        let input = Crossword::new(String::from("                ")).unwrap();

        let result = filler.fill(&input);

        assert!(result.is_ok());

        println!("{}", result.unwrap());
    }
}
