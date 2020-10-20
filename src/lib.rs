use crate::fill::Arc;
use crate::trie::Trie;
use crate::{ngram::bigrams, order::score_word};
use std::collections::HashMap;

use std::hash::Hash;
use std::{collections::HashSet, fmt, fs::File};

mod fill;
mod ngram;
mod order;
mod parse;
pub mod trie;

pub fn fill_crossword(contents: String, words: Vec<String>) {
    let crossword = Crossword::new(contents).unwrap();
    let (bigrams, trie) = index_words(words);
    fill::fill_crossword(&crossword, Arc::new(trie), Arc::new(bigrams));
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Crossword {
    contents: String,
    width: usize,
    height: usize,
}

impl Crossword {
    pub fn new(contents: String) -> Result<Crossword, String> {
        let without_newlines: String = contents.chars().filter(|c| *c != '\n').collect();

        let width = (without_newlines.len() as f64).sqrt() as usize;
        if width * width != without_newlines.len() {
            return Err(String::from("Contents are not a square."));
        }
        Ok(Crossword {
            contents: without_newlines,
            width,
            height: width,
        })
    }
}

#[derive(Clone, Debug)]
pub struct CrosswordWordIterator<'s> {
    crossword: &'s Crossword,
    word_boundary: &'s WordBoundary,
    index: usize,
}

impl<'s> Iterator for CrosswordWordIterator<'s> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.word_boundary.length {
            return None;
        }

        match self.word_boundary.direction {
            Direction::Across => {
                let char_index = self.word_boundary.start_row * self.crossword.width
                    + self.word_boundary.start_col
                    + self.index;
                let result = self.crossword.contents.as_bytes()[char_index] as char;
                self.index += 1;
                Some(result)
            }
            Direction::Down => {
                let char_index = (self.word_boundary.start_row + self.index) * self.crossword.width
                    + self.word_boundary.start_col;
                let result = self.crossword.contents.as_bytes()[char_index] as char;
                self.index += 1;
                Some(result)
            }
        }
    }
}

impl Hash for CrosswordWordIterator<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for c in (*self).clone() {
            c.hash(state);
        }
    }
}

impl PartialEq for CrosswordWordIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.word_boundary.length != other.word_boundary.length {
            return false;
        }

        self.clone().zip(other.clone()).all(|(a, b)| a == b)
    }
}

impl Eq for CrosswordWordIterator<'_> {}

impl fmt::Display for Crossword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                write!(
                    f,
                    "{}",
                    self.contents.as_bytes()[row * self.width + col] as char
                )?;
                if col != self.width - 1 {
                    write!(f, " ")?;
                }
            }
            writeln!(f)?;

            if row != self.height - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

fn is_viable(candidate: &Crossword, word_boundaries: &Vec<WordBoundary>, trie: &Trie) -> bool {
    let mut already_used = HashSet::new();

    for word_boundary in word_boundaries {
        let iter = CrosswordWordIterator {
            crossword: candidate,
            word_boundary,
            index: 0,
        };
        if iter.clone().any(|c| c == ' ') {
            continue;
        }

        if already_used.contains(&iter) {
            return false;
        }
        already_used.insert(iter.clone());

        if !trie.is_word_iter(iter) {
            return false;
        }
    }
    true
}

// TODO: use RO behavior here
pub fn find_fills(word: Word, trie: &Trie) -> Vec<Word> {
    trie.words(word.contents.clone())
        .drain(0..)
        .map(|new_word| Word {
            contents: new_word,
            ..word.clone()
        })
        .collect()
}

#[derive(Debug, PartialEq, Clone)]
pub struct WordBoundary {
    start_row: usize,
    start_col: usize,
    length: usize,
    direction: Direction,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Direction {
    Across,
    Down,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Word {
    contents: String,
    start_row: usize,
    start_col: usize,
    length: usize,
    direction: Direction,
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Contents: {}", self.contents)
    }
}

impl Word {
    pub fn new(
        contents: String,
        start_row: usize,
        start_col: usize,
        length: usize,
        direction: Direction,
    ) -> Word {
        Word {
            contents,
            start_row,
            start_col,
            length,
            direction,
        }
    }
}

pub fn default_words() -> Vec<String> {
    let file = File::open("wordlist.json").unwrap();
    serde_json::from_reader(file).expect("JSON was not well-formatted")
}

pub fn index_words(raw_data: Vec<String>) -> (HashMap<(char, char), usize>, Trie) {
    let bigram = bigrams(&raw_data);
    let trie = Trie::build(raw_data);
    (bigram, trie)
}

#[cfg(test)]
mod tests {

    use crate::order::FrequencyOrderableCrossword;
use crate::parse::parse_word_boundaries;
    use crate::WordBoundary;
    use crate::{default_words, index_words};
    use std::{cmp::Ordering, collections::HashSet};

    use crate::{find_fills, is_viable, Crossword, CrosswordWordIterator, Direction, Word};

    #[test]
    fn it_works() {
        let result = Crossword::new(String::from(
            "
abc
def
ghi
",
        ));

        assert!(result.is_ok());

        let c = result.unwrap();
        assert_eq!(String::from("abcdefghi"), c.contents);
        assert_eq!(3, c.width);
        assert_eq!(3, c.height);
        println!("{}", c);
    }

    #[test]
    fn find_fill_works() {
        let (_, trie) = index_words(default_words());

        let input = Word {
            contents: String::from("   "),
            length: 3,
            start_row: 0,
            start_col: 0,
            direction: Direction::Across,
        };
        assert!(find_fills(input.clone(), &trie).contains(&Word {
            contents: String::from("CAT"),
            ..input.clone()
        }));

        let input = Word {
            contents: String::from("C T"),
            length: 3,
            start_row: 0,
            start_col: 0,
            direction: Direction::Across,
        };
        assert!(find_fills(input.clone(), &trie).contains(&Word {
            contents: String::from("CAT"),
            ..input.clone()
        }));

        let input = Word {
            contents: String::from("  T"),
            length: 3,
            start_row: 0,
            start_col: 0,
            direction: Direction::Across,
        };
        assert!(find_fills(input.clone(), &trie).contains(&Word {
            contents: String::from("CAT"),
            ..input.clone()
        }));

        let input = Word {
            contents: String::from("CAT"),
            length: 3,
            start_row: 0,
            start_col: 0,
            direction: Direction::Across,
        };
        assert!(find_fills(input.clone(), &trie).contains(&Word {
            contents: String::from("CAT"),
            ..input.clone()
        }));
    }

    #[test]
    fn is_viable_works() {
        let (_, trie) = index_words(default_words());

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
    fn crossword_iterator_works() {
        let input = Crossword::new(String::from("ABCDEFGHI")).unwrap();
        let word_boundary = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Across,
            length: 3,
        };

        let t = CrosswordWordIterator {
            crossword: &input,
            word_boundary: &word_boundary,
            index: 0,
        };

        let s: String = t.collect();

        assert_eq!(String::from("ABC"), s);

        let word_boundary = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Down,
            length: 3,
        };

        let t = CrosswordWordIterator {
            crossword: &input,
            word_boundary: &word_boundary,
            index: 0,
        };

        let s: String = t.collect();

        assert_eq!(String::from("ADG"), s);
    }

    #[test]
    fn crossword_iterator_eq_works() {
        let input = Crossword::new(String::from("ABCB  C  ")).unwrap();
        let a = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Across,
            length: 3,
        };
        let b = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Down,
            length: 3,
        };

        let a_iter = CrosswordWordIterator {
            crossword: &input,
            word_boundary: &a,
            index: 0,
        };

        let b_iter = CrosswordWordIterator {
            crossword: &input,
            word_boundary: &b,
            index: 0,
        };

        assert_eq!(a_iter, b_iter);
    }

    #[test]
    fn crossword_iterator_hash_works() {
        let input = Crossword::new(String::from("ABCB  C  ")).unwrap();
        let a = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Across,
            length: 3,
        };
        let b = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Down,
            length: 3,
        };

        let a_iter = CrosswordWordIterator {
            crossword: &input,
            word_boundary: &a,
            index: 0,
        };

        let b_iter = CrosswordWordIterator {
            crossword: &input,
            word_boundary: &b,
            index: 0,
        };

        let mut set = HashSet::new();

        set.insert(a_iter);

        assert!(set.contains(&b_iter));
    }

    #[test]
    fn crossword_ord_works() {
        let words = default_words();
        let (bigrams, _) = index_words(words);

        let a = FrequencyOrderableCrossword::new(
            Crossword::new(String::from("   TNERTN")).unwrap(),
            &bigrams,
        );
        println!("{:?}", a);

        let b = FrequencyOrderableCrossword::new(
            Crossword::new(String::from("   XYQQWZ")).unwrap(),
            &bigrams,
        );

        println!("{:?}", b);

        assert_eq!(a.cmp(&b), Ordering::Greater)
    }
}
