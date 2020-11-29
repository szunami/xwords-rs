/*!
Core types to represent a crossword puzzle.
*/

use crate::parse::WordBoundary;
use std::{fmt, hash::Hash};

/// The underlying representation of a crossword puzzle. All of
/// the contents are stored in a string and the dimensions of the grid
/// are stored explicitly.
///
/// In the contents, `*` represents a shaded square, and a ` ` represents
/// a blank square.
/// 
/// To parse a square grid, see [`xwords::crossword::Crossword::square`]. To parse a 
/// rectangular grid, see [`xwords::crossword::Crossword::rectangle`]

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Crossword {
    pub(crate) contents: String,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl Crossword {
    /// Parses a crossword. Assumes that grid width and height are equal and returns
    /// an Err if not. Newlines are removed.
    pub fn square(contents: String) -> Result<Crossword, String> {
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

    /// Parses a crossword. Assumes that width and height are as specified. If the length
    /// of the input does not match the input dimensions, an Err is returned. Newlines are
    /// removed.
    pub fn rectangle(contents: String, width: usize, height: usize) -> Result<Crossword, String> {
        let without_newlines: String = contents.chars().filter(|c| *c != '\n').collect();
        if without_newlines.len() != width * height {
            return Err(String::from("Contents do not match specified dimensions"));
        }
        Ok(Crossword {
            contents: without_newlines,
            width,
            height,
        })
    }
}

/// An `Iterator<char>` that correctly traversing a Crossword, accounting for direction.
/// 
/// The length of the word is stored in the `word_boundary`.
#[derive(Clone, Debug)]
pub struct WordIterator<'s> {
    crossword: &'s Crossword,
    pub word_boundary: &'s WordBoundary,
    index: usize,
}

impl<'s> WordIterator<'s> {
    pub fn new(
        crossword: &'s Crossword,
        word_boundary: &'s WordBoundary,
    ) -> WordIterator<'s> {
        WordIterator {
            crossword,
            word_boundary,
            index: 0,
        }
    }
}

impl<'s> fmt::Display for WordIterator<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.clone() {
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}

impl<'s> Iterator for WordIterator<'s> {
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

impl Hash for WordIterator<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for c in (*self).clone() {
            c.hash(state);
        }
    }
}

impl PartialEq for WordIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.word_boundary.length != other.word_boundary.length {
            return false;
        }

        self.clone().zip(other.clone()).all(|(a, b)| a == b)
    }
}

impl Eq for WordIterator<'_> {}

impl fmt::Display for Crossword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                write!(
                    f,
                    "{}",
                    self.contents.as_bytes()[row * self.width + col] as char
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// The direction of a word in a Crossword.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Direction {
    Across,
    Down,
}

#[cfg(test)]
mod tests {
    use super::Crossword;
    use crate::{crossword::WordIterator, parse::WordBoundary};
    use std::collections::HashSet;

    use super::Direction;

    #[test]

    fn it_works() {
        let result = Crossword::square(String::from(
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
    fn crossword_iterator_works() {
        let input = Crossword::square(String::from("ABCDEFGHI")).unwrap();
        let word_boundary = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Across,
            length: 3,
        };

        let t = WordIterator {
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

        let t = WordIterator {
            crossword: &input,
            word_boundary: &word_boundary,
            index: 0,
        };

        let s: String = t.collect();

        assert_eq!(String::from("ADG"), s);
    }

    #[test]
    fn crossword_iterator_eq_works() {
        let input = Crossword::square(String::from("ABCB  C  ")).unwrap();
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

        let a_iter = WordIterator {
            crossword: &input,
            word_boundary: &a,
            index: 0,
        };

        let b_iter = WordIterator {
            crossword: &input,
            word_boundary: &b,
            index: 0,
        };

        assert_eq!(a_iter, b_iter);
    }

    #[test]
    fn crossword_iterator_hash_works() {
        let input = Crossword::square(String::from("ABCB  C  ")).unwrap();
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

        let a_iter = WordIterator {
            crossword: &input,
            word_boundary: &a,
            index: 0,
        };

        let b_iter = WordIterator {
            crossword: &input,
            word_boundary: &b,
            index: 0,
        };

        let mut set = HashSet::new();

        set.insert(a_iter);

        assert!(set.contains(&b_iter));
    }
}
