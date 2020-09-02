use std::{collections::HashSet, fmt, fs::File};

#[macro_use]
extern crate lazy_static;
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
struct Crossword {
    contents: String,
    width: usize,
    height: usize,
}

// impl Clone for Crossword {
//     fn clone(&self) -> Self {
//         Crossword {
//             contents: self.contents.clone(),
//             width: self.width,
//             height: self.height,
//         }
//     }
// }

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

fn fill_crossword(crossword: &Crossword) -> Result<Crossword, String> {
    // parse crossword into partially filled words
    // fill a word
    let mut candidates = vec![crossword.clone()];
    let mut visited_candidates: HashSet<Crossword> = HashSet::new();
    visited_candidates.insert(crossword.to_owned());

    loop {
        if candidates.len() == 0 {
            return Err(String::from("Failed to fill."));
        }

        let candidate = candidates.pop().unwrap();
        visited_candidates.insert(candidate.to_owned());

        let words = parse_words(crossword);
        for word in words {
            // find valid fills for word;
            // for each fill:
            //   are all complete words legit?
            //     if so, push

            let potential_fills = find_fills(word);

            for potential_fill in potential_fills {
                let new_candidate = fill_one_word(&candidate, potential_fill);
                // TODO: are all complete words legit?
                if visited_candidates.contains(&new_candidate) {
                    candidates.push(new_candidate);
                }
            }
        }
    }
}

fn fill_one_word(candidate: &Crossword, potential_fill: Word) -> Crossword {
    let mut result_contents = candidate.contents.clone();

    match potential_fill.direction {
        Direction::Across => {
            let mut bytes = result_contents.into_bytes();

            for index in 0..potential_fill.contents.len() {
                let col = potential_fill.start_col + index;

                bytes[potential_fill.start_row * candidate.width + col] =
                    potential_fill.contents.as_bytes()[index];
            }
            unsafe { result_contents = String::from_utf8_unchecked(bytes) }
        }
        Direction::Down => {
            let mut bytes = result_contents.into_bytes();

            for index in 0..potential_fill.contents.len() {
                let row = potential_fill.start_row + index;

                bytes[row * candidate.width + potential_fill.start_col] =
                    potential_fill.contents.as_bytes()[index];
            }
            unsafe { result_contents = String::from_utf8_unchecked(bytes) }
        }
    }

    Crossword {
        contents: result_contents,
        ..*candidate
    }
}

fn find_fills(word: Word) -> Vec<Word> {
    let mut result = vec![];

    for real_word in all_words.iter() {
        if real_word.len() != word.contents.len() {
            continue;
        }
        let mut is_valid = true;
        for i in 0..word.contents.len() {
            if word.contents.as_bytes()[i] == b' '
                || word.contents.as_bytes()[i] == real_word.as_bytes()[i]
            {
                continue;
            } else {
                is_valid = false;
            }
        }
        if is_valid {
            result.push(Word {
                contents: real_word.to_string(),
                ..word.clone()
            })
        }
    }

    result
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
                current_word.push(current_char)
            } else {
                let new_word = Word {
                    contents: current_word,
                    start_row: start_row.unwrap(),
                    start_col: start_col.unwrap(),
                    length: length,
                    direction: Direction::Down,
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
                direction: Direction::Down,
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

#[derive(Debug, PartialEq, Clone)]
enum Direction {
    Across,
    Down,
}
#[derive(Debug, PartialEq, Clone)]
struct Word {
    contents: String,
    start_row: usize,
    start_col: usize,
    length: usize,
    direction: Direction,
}

lazy_static! {
    static ref all_words: Vec<String> = {
        let mut file = File::open("wordlist.json").unwrap();

        let json: serde_json::Value =
            serde_json::from_reader(file).expect("JSON was not well-formatted");

        match json.as_object() {
            Some(obj) => {
                return obj.keys().into_iter().map(String::to_owned).collect();
            }
            None => return vec![],
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{
        fill_crossword, fill_one_word, find_fills, parse_words, Crossword, Direction, Word,
    };
    use std::fs::File;

    #[test]
    fn it_works() {
        let c = Crossword {
            contents: String::from("abcdefghi"),
            width: 3,
            height: 3,
        };

        println!("{}", c);
    }

    // #[test]
    // fn fill_works() {
    //     let c = Crossword {
    //         contents: String::from("         "),
    //         width: 3,
    //         height: 3,
    //     };
    //     assert_eq!(fill_crossword(&c), c);
    // }

    #[test]
    fn parse_works() {
        let c = Crossword {
            contents: String::from("abcdefghi"),
            width: 3,
            height: 3,
        };
        let result = parse_words(&c);

        assert_eq!(result.len(), 6);
        assert_eq!(
            result[0],
            Word {
                contents: String::from("abc"),
                start_col: 0,
                start_row: 0,
                length: 3,
                direction: Direction::Across
            }
        );
        assert_eq!(
            result[1],
            Word {
                contents: String::from("def"),
                start_col: 0,
                start_row: 1,
                length: 3,
                direction: Direction::Across
            }
        );
        assert_eq!(
            result[2],
            Word {
                contents: String::from("ghi"),
                start_col: 0,
                start_row: 2,
                length: 3,
                direction: Direction::Across
            }
        );
        assert_eq!(
            result[3],
            Word {
                contents: String::from("adg"),
                start_col: 0,
                start_row: 0,
                length: 3,
                direction: Direction::Down,
            }
        )
    }

    #[test]
    fn fill_one_word_works() {
        let c = Crossword {
            contents: String::from("abcdefghi"),
            width: 3,
            height: 3,
        };

        let result = assert_eq!(
            fill_one_word(
                &c,
                Word {
                    contents: String::from("cat"),
                    start_col: 0,
                    start_row: 0,
                    length: 3,
                    direction: Direction::Across,
                }
            ),
            Crossword {
                contents: String::from("catdefghi"),
                width: 3,
                height: 3,
            }
        );

        assert_eq!(
            fill_one_word(
                &c,
                Word {
                    contents: String::from("cat"),
                    start_col: 0,
                    start_row: 0,
                    length: 3,
                    direction: Direction::Down,
                }
            ),
            Crossword {
                contents: String::from("cbcaefthi"),
                width: 3,
                height: 3,
            }
        );
    }

    #[test]
    fn find_fill_works() {
        let input = Word {
            contents: String::from("   "),
            length: 3,
            start_row: 0,
            start_col: 0,
            direction: Direction::Across,
        };
        assert!(find_fills(input.clone()).contains(&Word {
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
        assert!(find_fills(input.clone()).contains(&Word {
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
        assert!(find_fills(input.clone()).contains(&Word {
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
        assert!(find_fills(input.clone()).contains(&Word {
            contents: String::from("CAT"),
            ..input.clone()
        }));
    }
}