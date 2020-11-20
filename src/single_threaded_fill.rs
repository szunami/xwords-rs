use crate::order::FrequencyOrderableCrossword;
use crate::fill::fill_one_word;
use cached::SizedCache;

use crate::crossword::CrosswordWordIterator;
use crate::fill::{is_viable, CrosswordFillState};
use crate::order::score_iter;
use crate::parse::parse_word_boundaries;
use crate::Arc;
use std::{collections::HashMap, time::Instant};

use crate::{trie::Trie, Crossword};

cached_key! {
    WORDS: SizedCache<String, Vec<String>> = SizedCache::with_size(10_000);
    Key = { pattern.clone() };
    fn words(pattern: String, trie: &Trie) -> Vec<String> = {
        trie.words(pattern)
    }
}

pub fn fill_crossword_single_threaded(
    crossword: &Crossword,
    trie: Arc<Trie>,
    bigrams: Arc<HashMap<(char, char), usize>>,
) -> Result<Crossword, String> {
    // parse crossword into partially filled words
    // fill a word

    let thread_start = Instant::now();

    let mut crossword_fill_state = {
        let mut temp_state = CrosswordFillState::new();
        let orderable = FrequencyOrderableCrossword::new(crossword.clone(), bigrams.as_ref());
        temp_state
    };

    let word_boundaries = parse_word_boundaries(&crossword);
    let mut candidate_count = 0;

    loop {
        let candidate = match crossword_fill_state.take_candidate() {
            Some(c) => c,
            None => return Err(String::from("Ran out of candidates. Yikes.")),
        };

        candidate_count += 1;

        if candidate_count % 1_000 == 0 {
            println!(
                "Throughput: {}",
                candidate_count as f32 / thread_start.elapsed().as_millis() as f32
            );
        }

        let to_fill = word_boundaries
            .iter()
            .map(|word_boundary| CrosswordWordIterator::new(&candidate, word_boundary))
            .filter(|iter| iter.clone().any(|c| c == ' '))
            .min_by_key(|iter| score_iter(iter, bigrams.as_ref()))
            .unwrap();
        // find valid fills for word;
        // for each fill:
        //   are all complete words legit?
        //     if so, push

        let potential_fills = words(to_fill.clone().to_string(), trie.as_ref());

        for potential_fill in potential_fills {
            let new_candidate = fill_one_word(&candidate, &to_fill.clone(), potential_fill);

            if is_viable(&new_candidate, &word_boundaries, trie.as_ref()) {
                if !new_candidate.contents.contains(" ") {
                    return Ok(new_candidate);
                }
                let orderable = FrequencyOrderableCrossword::new(new_candidate, bigrams.as_ref());
                crossword_fill_state.add_candidate(orderable);
            }
        }
    }
}
