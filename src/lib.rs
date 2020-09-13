use std::sync::Arc;
use std::sync::{mpsc, Mutex};
use std::{
    collections::{HashSet, VecDeque},
    fmt,
    fs::File,
};

use pprof::ProfilerGuard;

#[macro_use]
extern crate lazy_static;
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Crossword {
    contents: String,
    width: usize,
    height: usize,
}

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
            write!(f, "\n")?;

            if row != self.height - 1 {
                write!(f, "\n")?;
            }
        }
        Ok(())
    }
}

struct CrosswordFillState {
    // Used to ensure we only enqueue each crossword once.
    // Contains crosswords that are queued or have already been visited
    processed_candidates: HashSet<Crossword>,
    candidate_queue: VecDeque<Crossword>,
}

impl CrosswordFillState {
    fn take_candidate(&mut self) -> Option<Crossword> {
        self.candidate_queue.pop_back()
    }

    fn add_candidate(&mut self, candidate: Crossword) {
        if !self.processed_candidates.contains(&candidate) {
            self.candidate_queue.push_back(candidate);
        }
    }
}

pub fn fill_crossword(crossword: &Crossword) -> Result<Crossword, String> {
    // parse crossword into partially filled words
    // fill a word

    let crossword_fill_state = {
        let mut temp_state = CrosswordFillState {
            processed_candidates: HashSet::new(),
            candidate_queue: VecDeque::new(),
        };
        temp_state.add_candidate(crossword.clone());
        temp_state
    };

    let candidates = Arc::new(Mutex::new(crossword_fill_state));
    let (tx, rx) = mpsc::channel();
    // want to spawn multiple threads, have each of them perform the below

    for thread_index in 0..1 {
        let new_arc = Arc::clone(&candidates);
        let new_tx = tx.clone();

        std::thread::spawn(move || {
            println!("Hello from thread {}", thread_index);

            let guard = pprof::ProfilerGuard::new(100).unwrap();

            loop {
                let candidate = {
                    let mut queue = new_arc.lock().unwrap();
                    match queue.take_candidate() {
                        Some(candidate) => candidate,
                        None => continue,
                    }
                };
                let words = parse_words(&candidate);
                let to_fill = words
                    .iter()
                    .max_by_key(|word| {
                        let empty_squares: i32 = word.contents.matches(" ").count() as i32;
                        // we want to identify highly constrained words
                        // very unscientifically: we want longer words, with fewer spaces.
                        if empty_squares == 0 {
                            return -1;
                        }
                        return 2 * word.contents.len() as i32 - empty_squares;
                    })
                    .unwrap();
                // find valid fills for word;
                // for each fill:
                //   are all complete words legit?
                //     if so, push

                let potential_fills = find_fills(to_fill.to_owned());

                for potential_fill in potential_fills {
                    let new_candidate = fill_one_word(&candidate, potential_fill);

                    if is_viable(&new_candidate) {
                        if !new_candidate.contents.contains(" ") {

                            if let Ok(report) = guard.report().build() {
                                let file = File::create(format!(
                                    "flamegraph-{}.svg",
                                    thread_index
                                ))
                                .unwrap();
                                report.flamegraph(file).unwrap();
                            };

                            // return Ok(new_candidate);
                            match new_tx.send(new_candidate.clone()) {
                                Ok(_) => {
                   
                                    println!("Just sent a result.")
                                }
                                Err(err) => println!("Failed to send a result, error was {}", err),
                            }
                        }

                        let mut queue = new_arc.lock().unwrap();
                        queue.add_candidate(new_candidate);
                    }
                }
            }
        });
    }

    match rx.recv() {
        Ok(result) => Ok(result),
        Err(_) => Err(String::from("Failed to receive")),
    }
}

fn is_viable(candidate: &Crossword) -> bool {
    for word in parse_words(candidate) {
        if !ALL_WORDS.contains(&word.contents) && !word.contents.contains(" ") {
            return false;
        }
    }
    true
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

    for real_word in ALL_WORDS.iter() {
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
    static ref ALL_WORDS: HashSet<String> = {
        let file = File::open("wordlist.json").unwrap();

        let json: serde_json::Value =
            serde_json::from_reader(file).expect("JSON was not well-formatted");

        match json.as_object() {
            Some(obj) => {
                return obj.keys().into_iter().map(String::to_owned).collect();
            }
            None => return HashSet::new(),
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::ALL_WORDS;
    use crate::{
        fill_crossword, fill_one_word, find_fills, is_viable, parse_words, Crossword, Direction,
        Word,
    };
    use std::fs::File;
    use std::time::Instant;

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
        // let guard = pprof::ProfilerGuard::new(100).unwrap();

        ALL_WORDS.len();
        let start = Instant::now();

        let c = Crossword {
            contents: String::from("                "),
            width: 4,
            height: 4,
        };
        println!("{}", fill_crossword(&c).unwrap());
        println!("{}", start.elapsed().as_millis());
        // if let Ok(report) = guard.report().build() {
        //     let file = File::create("flamegraph.svg").unwrap();
        //     report.flamegraph(file).unwrap();
        // };
    }

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

        assert_eq!(
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

    #[test]
    fn is_viable_works() {
        assert!(is_viable(&Crossword {
            contents: String::from("         "),
            width: 3,
            height: 3,
        }));

        assert!(!is_viable(&Crossword {
            contents: String::from("ABCDEFGH "),
            width: 3,
            height: 3,
        }));
    }
}
