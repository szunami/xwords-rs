/*!
Utility methods to split a `Crossword` into component words.
*/
use crate::{Crossword, Direction};

/// Parses a Crossword into a `Vec<WordBoundary>`. Returns all words present in the puzzle.
///
/// Note that every square in a Crossword is present in two word boundaries; one `Down` and
/// one `Across`.
///
/// Also note that as a `Crossword` is being filled, the word boundaries do not change.
pub fn parse_word_boundaries(crossword: &Crossword) -> Vec<WordBoundary> {
    let mut result = vec![];

    let byte_array = crossword.contents.as_bytes();

    let mut start_row = None;
    let mut start_col = None;
    let mut length = 0;

    for row in 0..crossword.height {
        for col in 0..crossword.width {
            let current_char = byte_array[row * crossword.width + col] as char;
            if current_char != '*' {
                // found a char; is it our first?
                if start_row == None {
                    start_row = Some(row);
                    start_col = Some(col);
                }
                length += 1;
            } else {
                // If we don't have any data yet, just keep going
                if start_row == None {
                    continue;
                }
                let new_word = WordBoundary {
                    start_row: start_row.unwrap(),
                    start_col: start_col.unwrap(),
                    length,
                    direction: Direction::Across,
                };
                result.push(new_word);
                length = 0;
                start_row = None;
                start_col = None;
            }
        }
        // have to process end of row
        if length > 0 {
            let new_word = WordBoundary {
                start_row: start_row.unwrap(),
                start_col: start_col.unwrap(),
                length,
                direction: Direction::Across,
            };
            result.push(new_word);
            length = 0;
            start_row = None;
            start_col = None;
        }
    }

    let mut start_row = None;
    let mut start_col = None;
    let mut length = 0;

    for col in 0..crossword.width {
        for row in 0..crossword.height {
            let current_char = byte_array[row * crossword.width + col] as char;
            if current_char != '*' {
                // found a char; is it our first?
                if start_row == None {
                    start_row = Some(row);
                    start_col = Some(col);
                }
                length += 1;
            } else {
                if start_row == None {
                    continue;
                }
                let new_word = WordBoundary {
                    start_row: start_row.unwrap(),
                    start_col: start_col.unwrap(),
                    length,
                    direction: Direction::Down,
                };
                result.push(new_word);
                length = 0;
                start_row = None;
                start_col = None;
            }
        }
        // have to process end of row
        if length > 0 {
            let new_word = WordBoundary {
                start_row: start_row.unwrap(),
                start_col: start_col.unwrap(),
                length,
                direction: Direction::Down,
            };
            result.push(new_word);
            length = 0;
            start_row = None;
            start_col = None;
        }
    }

    result
}

/// A representation of a word in a `Crossword`. Note that a `WordBoundary` is not
/// attached to a specific `Crossword`, and that it is mostly used to represent
/// a location in a grid.
///
/// Note that a `WordBoundary` can be combined with a `&Crossword` to create a `WordIterator`,
/// which will produce the `char`s present in that specific `Crossword`.
#[derive(Debug, PartialEq, Clone)]
pub struct WordBoundary {
    pub start_row: usize,
    pub start_col: usize,
    pub length: usize,
    pub direction: Direction,
}

impl WordBoundary {
    pub fn new(
        start_row: usize,
        start_col: usize,
        length: usize,
        direction: Direction,
    ) -> WordBoundary {
        WordBoundary {
            start_row,
            start_col,
            length,
            direction,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::parse::parse_word_boundaries;

    use crate::{Crossword, Direction};

    use super::WordBoundary;

    #[test]
    fn parse_word_boundaries_works() {
        let c = Crossword::square(String::from(
            "
abc
def
ghi
",
        ))
        .unwrap();
        let result = parse_word_boundaries(&c);

        assert_eq!(result.len(), 6);
        assert_eq!(
            result[0],
            WordBoundary {
                start_col: 0,
                start_row: 0,
                length: 3,
                direction: Direction::Across
            }
        );
        assert_eq!(
            result[1],
            WordBoundary {
                start_col: 0,
                start_row: 1,
                length: 3,
                direction: Direction::Across
            }
        );
        assert_eq!(
            result[2],
            WordBoundary {
                start_col: 0,
                start_row: 2,
                length: 3,
                direction: Direction::Across
            }
        );
        assert_eq!(
            result[3],
            WordBoundary {
                start_col: 0,
                start_row: 0,
                length: 3,
                direction: Direction::Down,
            }
        )
    }

    #[test]
    fn parse_word_boundaries_big_grid() {
        let c = Crossword::square(String::from(
            "
    *    *     
    *    *     
         *     
   *   *   *   
**    *        
      *     ***
     *    *    
   *       *   
    *    *     
***     *      
        *    **
   *   *   *   
     *         
     *    *    
     *    *    
",
        ))
        .unwrap();

        let result = parse_word_boundaries(&c);

        assert_eq!(result.len(), 80);
        assert_eq!(
            result[0],
            WordBoundary {
                start_col: 0,
                start_row: 0,
                length: 4,
                direction: Direction::Across
            }
        );

        assert_eq!(
            result[1],
            WordBoundary {
                start_col: 5,
                start_row: 0,
                length: 4,
                direction: Direction::Across
            }
        );

        assert_eq!(
            result[41],
            WordBoundary {
                start_col: 0,
                start_row: 0,
                length: 4,
                direction: Direction::Down
            }
        );

        assert_eq!(
            result[79],
            WordBoundary {
                start_col: 14,
                start_row: 11,
                length: 4,
                direction: Direction::Down
            }
        );
    }
}
