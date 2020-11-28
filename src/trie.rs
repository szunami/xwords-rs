use crate::crossword::CrosswordWordIterator;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fmt;
#[derive(Clone, Serialize, Deserialize)]
pub struct TrieNode {
    contents: Option<char>,
    children: FxHashMap<char, TrieNode>,
    is_terminal: bool,
}

impl TrieNode {
    fn add_sequence(mut self, chars: &str) -> TrieNode {
        match chars.as_bytes().get(0) {
            Some(val) => {
                match self.children.remove_entry(&(*val as char)) {
                    Some((_, child)) => {
                        self.children
                            .insert(*val as char, child.add_sequence(&chars[1..]));
                    }
                    None => {
                        let tmp = TrieNode {
                            children: FxHashMap::default(),
                            contents: Some(*val as char),
                            is_terminal: false,
                        };
                        // create child and iterate on it
                        self.children
                            .insert(*val as char, tmp.add_sequence(&chars[1..]));
                    }
                }
            }
            None => {
                self.is_terminal = true;
            }
        }

        self
    }

    fn display_helper(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        depth: usize,
        first_child: bool,
    ) -> std::result::Result<(), std::fmt::Error> {
        if !first_child {
            for _ in 0..depth {
                write!(f, "\t")?;
            }
        } else {
            write!(f, "\t")?;
        }
        write!(f, "{}", self.contents.unwrap_or('*'))?;

        if self.is_terminal {
            write!(f, "'")?;
        }

        if self.children.is_empty() {
            return writeln!(f);
        }

        for (index, key) in self.children.keys().into_iter().enumerate() {
            self.children
                .get(key)
                .unwrap()
                .display_helper(f, depth + 1, index == 0)?;
        }

        Ok(())
    }

    fn words(&self, mut pattern: CrosswordWordIterator, partial: String, result: &mut Vec<String>) {
        let mut new_partial = partial;
        if self.contents.is_some() {
            new_partial.push(self.contents.unwrap());
        }

        match pattern.next() {
            Some(new_char) => {
                if new_char == ' ' {
                    for child in self.children.values() {
                        child.words(pattern.clone(), new_partial.clone(), result);
                    }
                }
                else {
                    match self.children.get(&new_char) {
                        Some(child) => child.words(pattern, new_partial, result),
                        None => {},
                    }
                }
            }
            None => {
                if self.is_terminal {
                    result.push(new_partial);
                }
                return;
            }
        }
    }

    pub fn is_viable(&self, mut chars: CrosswordWordIterator) -> bool {
        match chars.next() {
            None => self.is_terminal,

            Some(c) => {
                if c == ' ' {
                    for child in self.children.values() {
                        if child.is_viable(chars.clone()) {
                            return true;
                        }
                    }
                    return false;
                } else {
                    match self.children.get(&c) {
                        None => false,
                        Some(child) => child.is_viable(chars),
                    }
                }
            }
        }
    }
}

impl fmt::Display for TrieNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.display_helper(f, 1, true)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Trie {
    pub root: TrieNode,
}

impl fmt::Display for Trie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.root.fmt(f)
    }
}

impl Trie {
    pub fn build(words: Vec<String>) -> Trie {
        let mut root = TrieNode {
            contents: None,
            children: FxHashMap::default(),
            is_terminal: false,
        };

        println!("Building {} words", words.len());

        for word in words.iter() {
            root = root.add_sequence(&word);
        }

        println!("Done building");

        Trie { root }
    }

    pub fn words(&self, pattern: CrosswordWordIterator) -> Vec<String> {
        let mut result = Vec::with_capacity(4);
        self.root.words(pattern, String::from(""), &mut result);
        result
    }

    pub fn is_viable(&self, chars: CrosswordWordIterator) -> bool {
        self.root.is_viable(chars)
    }
}

#[cfg(test)]
mod tests {

    use rustc_hash::FxHashMap;

    use std::collections::HashSet;

    use crate::{
        crossword::{Crossword, CrosswordWordIterator, Direction},
        parse::WordBoundary,
    };

    use super::{Trie, TrieNode};

    #[test]
    fn display_works() {
        let mut root = TrieNode {
            contents: None,
            children: FxHashMap::default(),
            is_terminal: false,
        };

        root.children.insert(
            'b',
            TrieNode {
                contents: Some('b'),
                children: FxHashMap::default(),
                is_terminal: false,
            },
        );

        let mut c = TrieNode {
            contents: Some('c'),
            children: FxHashMap::default(),
            is_terminal: false,
        };

        c.children.insert(
            'd',
            TrieNode {
                contents: Some('d'),
                children: FxHashMap::default(),
                is_terminal: false,
            },
        );

        root.children.insert('c', c);

        println!("{}", root);
    }

    #[test]
    fn add_sequence_works() {
        let root = TrieNode {
            contents: Some('a'),
            children: FxHashMap::default(),
            is_terminal: false,
        };

        let new_root = root.add_sequence("itsyaboi");

        println!("{}", new_root);

        let another_root = new_root.add_sequence("wereallyouthere");

        println!("{}", another_root)
    }

    #[test]
    fn build_works() {
        println!(
            "{}",
            Trie::build(vec![
                String::from("asdf"),
                String::from("asset"),
                String::from("bass"),
                String::from("baseball"),
                String::from("bassooon"),
                String::from("basset"),
            ])
        );
    }

    #[test]
    fn words_works() {
        let trie = Trie::build(vec![
            String::from("bass"),
            String::from("bats"),
            String::from("bess"),
            String::from("be"),
        ]);

        let expected: HashSet<String> = vec![String::from("bass"), String::from("bess")]
            .iter()
            .cloned()
            .collect();

        let c = Crossword::new(String::from(
            "
b ss
    
    
    
",
        ))
        .unwrap();

        let word_boundary = WordBoundary::new(0, 0, 4, Direction::Across);
        let iter = CrosswordWordIterator::new(&c, &word_boundary);
        let actual: HashSet<String> = trie.words(iter).iter().cloned().collect();
        assert_eq!(expected, actual,)
    }
}
