use crate::crossword::CrosswordWordIterator;
use core::cmp::Ordering;
use std::collections::HashMap;

use crate::Crossword;

#[derive(Eq, PartialEq, Debug)]
pub struct FrequencyOrderableCrossword {
    pub(crate) crossword: Crossword,
    space_count: usize,
    fillability_score: usize,
}

impl FrequencyOrderableCrossword {
    pub(crate) fn new(
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
        self.fillability_score.cmp(&other.fillability_score)
    }
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

    result
}

#[derive(Eq, PartialEq, Debug)]
pub(crate) struct WordScore {
    length: usize,
    space_count: usize,
    fillability_score: usize,
    start_col: usize,
    start_row: usize,
}

impl PartialOrd for WordScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WordScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // shorter words are more fillable
        if self.length != other.length {
            return other.length.cmp(&self.length);
        }

        // more spaces wins
        if self.space_count != other.space_count {
            return self.space_count.cmp(&other.space_count);
        }
        // higher fillability wins
        if self.fillability_score != other.fillability_score {
            return self.fillability_score.cmp(&other.fillability_score);
        }

        // tiebreaker
        if self.start_col != other.start_col {
            return self.start_col.cmp(&other.start_col);
        }

        return self.start_row.cmp(&other.start_row);
    }
}

pub(crate) fn score_iter(
    iter: &CrosswordWordIterator,
    bigrams: &HashMap<(char, char), usize>,
) -> WordScore {
    let mut fillability_score = std::usize::MAX;
    for (prev, curr) in iter.clone().zip(iter.clone().skip(1)) {
        let score = *bigrams.get(&(prev, curr)).unwrap_or(&std::usize::MIN);
        if fillability_score > score {
            fillability_score = score;
        }
    }

    WordScore {
        length: iter.word_boundary.length,
        space_count: iter.clone().filter(|c| *c == ' ').count(),
        fillability_score,
        start_col: iter.word_boundary.start_col,
        start_row: iter.word_boundary.start_row,
    }
}

#[cfg(test)]
mod tests {
    use crate::default_words;
    use crate::index_words;
    use crate::order::score_iter;
    use crate::order::CrosswordWordIterator;
    use crate::parse::WordBoundary;
    use crate::Direction::Across;
    use std::cmp::Ordering;

    use crate::bigrams;
    use crate::order::score_crossword;
    use crate::order::WordScore;
    use crate::Crossword;

    use super::FrequencyOrderableCrossword;

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

    #[test]
    fn score_iter_works() {
        let bigrams = bigrams(&vec![String::from("ASDF"), String::from("DF")]);

        let c = Crossword::new(String::from("ASDF            ")).unwrap();
        let w = WordBoundary {
            direction: Across,
            length: 4,
            start_col: 0,
            start_row: 0,
        };
        let input = CrosswordWordIterator::new(&c, &w);
        assert_eq!(
            WordScore {
                length: 4,
                space_count: 0,
                fillability_score: 1,
                start_col: input.word_boundary.start_col,
                start_row: input.word_boundary.start_row,
            },
            score_iter(&input, &bigrams)
        );

        let w = WordBoundary {
            direction: Across,
            length: 2,
            start_col: 2,
            start_row: 0,
        };
        let input = CrosswordWordIterator::new(&c, &w);
        assert_eq!(
            WordScore {
                length: 2,
                fillability_score: 2,
                space_count: 0,
                start_col: input.word_boundary.start_col,
                start_row: input.word_boundary.start_row,
            },
            score_iter(&input, &bigrams)
        );
    }

    #[test]
    fn word_score_ord_works() {
        assert_eq!(
            WordScore {
                length: 4,
                space_count: 5,
                fillability_score: 1,
                start_col: 0,
                start_row: 0,
            }
            .cmp(&WordScore {
                length: 3,
                space_count: 10,
                fillability_score: 2,
                start_col: 0,
                start_row: 0,
            }),
            Ordering::Less
        );

        assert_eq!(
            WordScore {
                length: 3,
                space_count: 5,
                fillability_score: 1,
                start_col: 0,
                start_row: 0,
            }
            .cmp(&WordScore {
                length: 3,
                space_count: 10,
                fillability_score: 2,
                start_col: 0,
                start_row: 0,
            }),
            Ordering::Less
        );

        assert_eq!(
            WordScore {
                length: 9,
                space_count: 5,
                fillability_score: 3,
                start_col: 0,
                start_row: 0,
            }
            .cmp(&WordScore {
                length: 9,
                space_count: 5,
                fillability_score: 2,
                start_col: 0,
                start_row: 0,
            }),
            Ordering::Greater
        );
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
