use crate::ngram::bigrams;
use crate::trie::Trie;
use std::collections::HashMap;

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashSet},
    fmt,
    fs::File,
};
use std::{
    hash::Hash,
    sync::{mpsc, Mutex},
};
use std::{sync::Arc};

mod ngram;
pub mod trie;

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

#[derive(Eq, PartialEq, Debug)]
struct FrequencyOrderableCrossword {
    crossword: Crossword,
    space_count: usize,
    fillability_score: usize,
}

impl FrequencyOrderableCrossword {
    fn new(
        crossword: Crossword,
        bigrams: &HashMap<(char, char), usize>,
    ) -> FrequencyOrderableCrossword {
        FrequencyOrderableCrossword {
            space_count: crossword.contents.chars().filter(|c| *c == ' ').count(),
            fillability_score: score_crossword(bigrams, &crossword),
            crossword,
        }
    }
}

impl PartialOrd for FrequencyOrderableCrossword {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FrequencyOrderableCrossword {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {

        // fewer spaces wins
        if self.space_count != other.space_count {
            return other.space_count.cmp(&self.space_count);
        }
        // higher fillability wins
        if self.fillability_score != other.fillability_score {
            return self.fillability_score.cmp(&other.fillability_score);
        }
        // tiebreaker for determinism
        return self.crossword.contents.cmp(&other.crossword.contents);
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
                return Some(result);
            }
            Direction::Down => {
                let char_index = (self.word_boundary.start_row + self.index) * self.crossword.width
                    + self.word_boundary.start_col;
                let result = self.crossword.contents.as_bytes()[char_index] as char;
                self.index += 1;
                return Some(result);
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

struct CrosswordFillState {
    // Used to ensure we only enqueue each crossword once.
    // Contains crosswords that are queued or have already been visited
    processed_candidates: HashSet<Crossword>,
    candidate_queue: BinaryHeap<FrequencyOrderableCrossword>,
    done: bool,
}

impl CrosswordFillState {
    fn take_candidate(&mut self) -> Option<Crossword> {
        self.candidate_queue.pop().map(|x| x.crossword)
    }

    fn add_candidate(&mut self, candidate: Crossword, bigrams: &HashMap<(char, char), usize>) {
        if !self.processed_candidates.contains(&candidate) {
            let orderable = FrequencyOrderableCrossword::new(candidate.clone(), bigrams);

            self.candidate_queue.push(orderable);
            self.processed_candidates.insert(candidate);
        } else {
            println!("Revisiting crossword: {}", candidate);
        }
    }

    fn mark_done(&mut self) {
        self.done = true;
    }
}

pub fn fill_crossword(
    crossword: &Crossword,
    trie: Arc<Trie>,
    bigrams: Arc<HashMap<(char, char), usize>>,
) -> Result<Crossword, String> {
    // parse crossword into partially filled words
    // fill a word

    let crossword_fill_state = {
        let mut temp_state = CrosswordFillState {
            processed_candidates: HashSet::new(),
            candidate_queue: BinaryHeap::new(),
            done: false,
        };
        temp_state.add_candidate(crossword.clone(), bigrams.as_ref());
        temp_state
    };

    let candidates = Arc::new(Mutex::new(crossword_fill_state));
    let (tx, rx) = mpsc::channel();
    // want to spawn multiple threads, have each of them perform the below

    for thread_index in 0..1 {
        let new_arc = Arc::clone(&candidates);
        let new_tx = tx.clone();
        let word_boundaries = parse_word_boundaries(&crossword);

        let trie = trie.clone();
        let bigrams = bigrams.clone();
        let mut candidate_count = 0;

        std::thread::Builder::new()
            .name(format!("{}", thread_index))
            .spawn(move || {
                println!("Hello from thread {}", thread_index);

                

                loop {
                    let candidate = {
                        let mut queue = new_arc.lock().unwrap();
                        if queue.done {
                            return;
                        }
                        match queue.take_candidate() {
                            Some(candidate) => candidate,
                            None => continue,
                        }
                    };

                    candidate_count += 1;

                    if candidate_count % 100 == 0 {
                        println!("{}", candidate);
                    }
                    // println!("Thread {} just got a candidate", thread_index);

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

                    let potential_fills = find_fills(to_fill.clone(), trie.as_ref());
                    for potential_fill in potential_fills {
                        let new_candidate = fill_one_word(&candidate, potential_fill);

                        if is_viable(&new_candidate, &word_boundaries, trie.as_ref()) {
                            if !new_candidate.contents.contains(" ") {
                                let mut queue = new_arc.lock().unwrap();
                                queue.mark_done();

                                match new_tx.send(new_candidate.clone()) {
                                    Ok(_) => {
                                        println!("Just sent a result.");
                                        return;
                                    }
                                    Err(err) => {
                                        println!("Failed to send a result, error was {}", err);
                                        return;
                                    }
                                }
                            }

                            let mut queue = new_arc.lock().unwrap();
                            queue.add_candidate(new_candidate, bigrams.as_ref());
                        }
                    }
                }
            })
            .unwrap();
    }

    match rx.recv() {
        Ok(result) => {
            let queue = candidates.lock().unwrap();

            println!("Processed {} candidates", queue.processed_candidates.len());
            Ok(result)
        }
        Err(_) => Err(String::from("Failed to receive")),
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
                // If we don't have any data yet, just keep going
                if start_row == None {
                    continue;
                }
                let new_word = Word {
                    contents: current_word,
                    start_row: start_row.unwrap(),
                    start_col: start_col.unwrap(),
                    length,
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
                length,
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
                if start_row == None {
                    continue;
                }
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
                length,
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

fn parse_word_boundaries(crossword: &Crossword) -> Vec<WordBoundary> {
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
                length: length,
                direction: Direction::Across,
            };
            result.push(new_word);
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
            } else {
                if start_row == None {
                    continue;
                }
                let new_word = WordBoundary {
                    start_row: start_row.unwrap(),
                    start_col: start_col.unwrap(),
                    length: length,
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
                length: length,
                direction: Direction::Down,
            };
            result.push(new_word);
            length = 0;
            start_row = None;
            start_col = None;
        }
    }

    return result;
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
    return serde_json::from_reader(file).expect("JSON was not well-formatted");
}

pub fn index_words(raw_data: Vec<String>) -> (HashMap<(char, char), usize>, Trie) {
    let bigram = bigrams(&raw_data);
    let trie = Trie::build(raw_data);
    return (bigram, trie);
}

fn score_crossword(bigrams: &HashMap<(char, char), usize>, crossword: &Crossword) -> usize {
    let mut result = std::usize::MAX;
    let byte_array = crossword.contents.as_bytes();
    for row in 0..crossword.height {
        for col in 1..(crossword.width - 1) {
            let current_char = byte_array[row * crossword.width + col] as char;
            let prev_char = byte_array[row * crossword.width + col - 1] as char;
            let score = {
                // TODO: bigrams as a type
                let tmp;
                if current_char == ' ' || prev_char == ' ' {
                    tmp = std::usize::MAX;
                } else {
                    let key = (prev_char, current_char);
                    tmp = *bigrams.get(&key).unwrap_or(&std::usize::MIN)
                }
                tmp
            };
            if result > score {
                result = score;
            }
        }
    }
    for row in 1..(crossword.height - 1) {
        for col in 0..crossword.width {
            let current_char = byte_array[row * crossword.width + col] as char;
            let prev_char = byte_array[(row - 1) * crossword.width + col] as char;
            let score = {
                // TODO: bigrams as a type
                let tmp;
                if current_char == ' ' || prev_char == ' ' {
                    tmp = std::usize::MAX;
                } else {
                    let key = (prev_char, current_char);
                    tmp = *bigrams.get(&key).unwrap_or(&std::usize::MIN)
                }
                tmp
            };
            if result > score {
                result = score;
            }
        }
    }

    return result;
}

#[cfg(test)]
mod tests {

    use crate::{default_words, index_words, score_crossword};
    use crate::{ngram::bigrams, FrequencyOrderableCrossword};
    use std::{
        cmp::Ordering, collections::HashSet, fs::File, sync::Arc,
        time::Instant,
    };

    use crate::{
        fill_crossword, fill_one_word, find_fills, is_viable, parse_words, Crossword,
        CrosswordWordIterator, Direction, Word,
    };
    use crate::{parse_word_boundaries, WordBoundary};

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
    fn bigger_parse_works() {
        let c = Crossword::new(String::from(
            "
**   **
*     *
       
       
       
*     *
**   **
",
        ))
        .unwrap();
        let result = parse_words(&c);

        assert_eq!(
            result[0],
            Word {
                contents: String::from("   "),
                start_col: 2,
                start_row: 0,
                length: 3,
                direction: Direction::Across
            }
        );

        assert_eq!(
            result[1],
            Word {
                contents: String::from("     "),
                start_col: 1,
                start_row: 1,
                length: 5,
                direction: Direction::Across
            }
        );

        assert_eq!(
            result[2],
            Word {
                contents: String::from("       "),
                start_col: 0,
                start_row: 2,
                length: 7,
                direction: Direction::Across
            }
        );

        assert_eq!(
            result[7],
            Word {
                contents: String::from("   "),
                start_col: 0,
                start_row: 2,
                length: 3,
                direction: Direction::Down
            }
        );
    }

    #[test]
    fn parse_works() {
        let c = Crossword::new(String::from(
            "
abc
def
ghi
",
        ))
        .unwrap();
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
    fn parse_word_boundaries_works() {
        let c = Crossword::new(String::from(
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
    fn fill_one_word_works() {
        let c = Crossword::new(String::from(
            "
abc
def
ghi
",
        ))
        .unwrap();

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
            Crossword::new(String::from(
                "
cat
def
ghi
",
            ))
            .unwrap()
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
            Crossword::new(String::from(
                "
cbc
aef
thi
",
            ))
            .unwrap()
        );
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
    fn fill_crossword_works() {
        let (bigrams, trie) = index_words(default_words());

        let input = Crossword::new(String::from("                ")).unwrap();

        let result = fill_crossword(&input, Arc::new(trie), Arc::new(bigrams));

        assert!(result.is_ok());

        println!("{}", result.unwrap());
    }

    #[test]
    fn puz_2020_10_12_works() {
        // let guard = pprof::ProfilerGuard::new(100).unwrap();
        // std::thread::spawn(move || loop {
        //     match guard.report().build() {
        //         Ok(report) => {
        //             let file = File::create("flamegraph.svg").unwrap();
        //             report.flamegraph(file).unwrap();
        //         }
        //         Err(_) => {}
        //     };
        //     std::thread::sleep(std::time::Duration::from_secs(5))
        // });

        let real_puz = Crossword::new(String::from(
            "
  S *    *     
  E *    *     
BARITONES*     
  V*   *   *   
**E   *        
RATEDR*     ***
  I  *    * C  
  M*       *R  
  E *    *  I  
***     *   T  
        *   I**
   *   *   *C  
     *GOODTHING
     *    * S  
     *    * M  
",
        ))
        .unwrap();

        println!("{}", real_puz);

        let now = Instant::now();
        let (bigrams, trie) = index_words(default_words()
            
        //     vec![
        //     String::from("BEST"),
        //     String::from("FRAN"),
        //     String::from("BANAL"),
        //     String::from("AVER"),
        //     String::from("LEGO"),
        //     String::from("ALIBI"),
        //     String::from("BARITONES"),
        //     String::from("NACHO"),
        //     String::from("ENV"),
        //     String::from("OWE"),
        //     String::from("ETD"),
        //     String::from("HON"),
        //     String::from("ELLE"),
        //     String::from("BROILERS"),
        //     String::from("RATEDR"),
        //     String::from("AINTI"),
        //     String::from("AMITY"),
        //     String::from("BING"),
        //     String::from("ACDC"),
        //     String::from("MMM"),
        //     String::from("AMALGAM"),
        //     String::from("RUE"),
        //     String::from("POET"),
        //     String::from("ALES"),
        //     String::from("AMINO"),
        //     String::from("ACIDY"),
        //     String::from("GRATES"),
        //     String::from("ENDZONES"),
        //     String::from("AGRI"),
        //     String::from("KIA"),
        //     String::from("ASA"),
        //     String::from("BRO"),
        //     String::from("COE"),
        //     String::from("EVILS"),
        //     String::from("GOODTHING"),
        //     String::from("BERET"),
        //     String::from("LANE"),
        //     String::from("ISTO"),
        //     String::from("YAYAS"),
        //     String::from("ETON"),
        //     String::from("DMVS"),
        //     String::from("BABE"),
        //     String::from("RAMP"),
        //     String::from("EKEBY"),
        //     String::from("EVAN"),
        //     String::from("AMMO"),
        //     String::from("NIVEA"),
        //     String::from("SERVETIME"),
        //     String::from("DAIRY"),
        //     String::from("TRI"),
        //     String::from("LET"),
        //     String::from("TAZ"),
        //     String::from("LEA"),
        //     String::from("TOLDYA"),
        //     String::from("COASTS"),
        //     String::from("FLOWER"),
        //     String::from("MAINS"),
        //     String::from("RENE"),
        //     String::from("BALDEAGLE"),
        //     String::from("AGE"),
        //     String::from("BAILEYS"),
        //     String::from("OAT"),
        //     String::from("NOSERINGS"),
        //     String::from("BONO"),
        //     String::from("TONGA"),
        //     String::from("GARDEN"),
        //     String::from("BANDIT"),
        //     String::from("MARGOT"),
        //     String::from("ALA"),
        //     String::from("LIA"),
        //     String::from("MAR"),
        //     String::from("HID"),
        //     String::from("NICHE"),
        //     String::from("CRITICISM"),
        //     String::from("ABHOR"),
        //     String::from("DUNE"),
        //     String::from("ONTV"),
        //     String::from("LIONS"),
        //     String::from("CEOS"),
        //     String::from("EGOS"),
        // ]
    );

        let filled_puz = fill_crossword(&real_puz, Arc::new(trie), Arc::new(bigrams)).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
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

    #[test]
    fn score_crossword_words() {
        let words = vec![
            String::from("ABC"),
            String::from("DEF"),
            String::from("GHI"),
            String::from("ADG"),
            String::from("BEH"),
            String::from("CFI"),
        ];

        let bigrams = bigrams(&words);

        let crossword = Crossword::new(String::from(
            "
ABC
DEF
GHI
",
        ))
        .unwrap();

        assert_eq!(1, score_crossword(&bigrams, &crossword));

        let crossword = Crossword::new(String::from(
            "
AXX
DEF
GHI
",
        ))
        .unwrap();
        assert_eq!(0, score_crossword(&bigrams, &crossword));

        let crossword = Crossword::new(String::from(
            "
   
DEF
GHI
",
        ))
        .unwrap();
        assert_eq!(1, score_crossword(&bigrams, &crossword));
    }
}
