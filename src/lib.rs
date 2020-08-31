use std::fmt;

#[derive(PartialEq, Debug)]
struct Crossword {
    contents: String,
    width: usize,
    height: usize,
}

impl Clone for Crossword {
    fn clone(&self) -> Self {
        Crossword {
            contents: self.contents.clone(),
            width: self.width,
            height: self.height,
        }
    }
}

impl fmt::Display for Crossword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                write!(
                    f,
                    "{}",
                    self.contents.as_bytes()[row * self.width + col] as char
                );
                if col != self.width - 1 {
                    write!(f, " ");
                }
            }
            write!(f, "\n");

            if row != self.height - 1 {
                write!(f, "\n");
            }
        }
        Ok(())
    }
}

fn fill_crossword(crossword: &Crossword) -> Crossword {
    // parse crossword into partially filled words
    // fill a word
    crossword.clone()
}

fn parse_words(crossword: &Crossword) -> Vec<Word> {
    let mut result = vec![];

    let byte_array = crossword.contents.as_bytes();

    let mut current_word = "".to_owned();
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
                current_word.push(current_char)
            } else {
                let new_word = Word {
                    contents: current_word,
                    start_row: start_row.unwrap(),
                    start_col: start_col.unwrap(),
                    length: length,
                    direction: Direction::Across,
                };
                result.push(new_word);
                current_word = "".to_owned();
                length = 0;
                start_row = None;
                start_col = None;
            }
        }
        // have to process end of row
        if current_word.len() > 0 {
            let new_word = Word {
                contents: current_word,
                start_row: start_row.unwrap(),
                start_col: start_col.unwrap(),
                length: length,
                direction: Direction::Across,
            };
            result.push(new_word);
            current_word = "".to_owned();
            length = 0;
            start_row = None;
            start_col = None;
        }
    }

    result
}

#[derive(Debug, PartialEq)]
enum Direction {
    Across,
    Down,
}
#[derive(Debug, PartialEq)]
struct Word {
    contents: String,
    start_row: usize,
    start_col: usize,
    length: usize,
    direction: Direction,
}

#[cfg(test)]
mod tests {
    use crate::{fill_crossword, parse_words, Crossword, Word};

    #[test]
    fn it_works() {
        let c = Crossword {
            contents: String::from("abcdefghi"),
            width: 3,
            height: 3,
        };

        println!("{}", c);
    }

    #[test]
    fn fill_works() {
        let c = Crossword {
            contents: String::from("         "),
            width: 3,
            height: 3,
        };
        assert_eq!(fill_crossword(&c), c);
    }

    #[test]
    fn parse_works() {
        let c = Crossword {
            contents: String::from("abcdefghi"),
            width: 3,
            height: 3,
        };
        let result = parse_words(&c);

        assert_eq!(result.len(), 3);
        assert_eq!(
            result[0],
            Word {
                contents: String::from("abc"),
                start_col: 0,
                start_row: 0,
                length: 3,
                direction: crate::Direction::Across
            }
        );
        assert_eq!(
            result[1],
            Word {
                contents: String::from("def"),
                start_col: 0,
                start_row: 1,
                length: 3,
                direction: crate::Direction::Across
            }
        );
    }
}
