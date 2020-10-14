use crate::{Crossword, WordBoundary};
use std::{collections::HashMap, fmt};

#[derive(Clone)]
pub struct TrieNode {
    contents: Option<char>,
    children: HashMap<char, TrieNode>,
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
                            children: HashMap::new(),
                            contents: Some(*val as char),
                            is_terminal: false,
                        };
                        // create child and iterate on it
                        self.children.insert(
                            *val as char,
                            tmp.add_sequence(&chars[1..]),
                        );
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

        if self.children.len() == 0 {
            return write!(f, "\n");
        }

        for (index, key) in self.children.keys().into_iter().enumerate() {
            self.children
                .get(key)
                .unwrap()
                .display_helper(f, depth + 1, index == 0)?;
        }

        Ok(())
    }

    // TODO: pattern could be a ref!
    fn words(&self, pattern: String, partial: String) -> Vec<String> {
        let mut new_partial = partial.clone();
        if self.contents.is_some() {
            new_partial.push(self.contents.unwrap());
        }

        if pattern.len() == 0 {
            if self.is_terminal {
                return vec![new_partial];
            }
            return vec![];
        }

        let new_pattern = pattern[1..].to_owned();

        let new_char = pattern.as_bytes()[0] as char;

        if new_char == ' ' {
            let mut result = vec![];
            for child in self.children.values() {
                let tmp = child.words(new_pattern.clone(), new_partial.clone());
                result.extend(tmp.clone());
            }
            return result;
        }

        match self.children.get(&new_char) {
            Some(child) => {
                return child.words(new_pattern, new_partial);
            }
            None => {
                return vec![];
            }
        }
    }

    // fn words(&self, crossword: &Crossword, word_boundary: &WordBoundary, index: usize) -> Vec<String> {
    //     todo!();
    // }

    fn is_word(&self, pattern: &str) -> bool {
        if pattern.len() == 0 {
            return self.is_terminal;
        }

        let new_pattern = &pattern[1..];

        let new_char = pattern.as_bytes()[0] as char;

        match self.children.get(&new_char) {
            Some(child) => child.is_word(new_pattern),
            None => false,
        }
    }
}

impl fmt::Display for TrieNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.display_helper(f, 1, true)
    }
}

pub struct Trie {
    pub root: TrieNode,
}

impl  fmt::Display for Trie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.root.fmt(f)
    }
}

impl  Trie {
    pub fn build(words: Vec<String>) -> Trie {
        let mut root = TrieNode {
            contents: None,
            children: HashMap::new(),
            is_terminal: false,
        };

        println!("Building {} words", words.len());

        for word in words.iter(){
            root = root.add_sequence(&word);
        }

        println!("Done building");

        Trie { root }
    }

    pub fn words(&self, pattern: String) -> Vec<String> {
        self.root.words(pattern, String::from(""))
    }

    // pub fn words(&self, crossword: &Crossword, word_boundary: &WordBoundary) -> Vec<String> {
    //     self.root.words(crossword, word_boundary, 0)
    // }

    pub fn is_word(&self, pattern: &str) -> bool {
        self.root.is_word(pattern)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
use std::collections::HashMap;

    use super::{Trie, TrieNode};

    #[test]
    fn display_works() {
        let mut root = TrieNode {
            contents: None,
            children: HashMap::new(),
            is_terminal: false,
        };

        root.children.insert(
            'b',
            TrieNode {
                contents: Some('b'),
                children: HashMap::new(),
                is_terminal: false,
            },
        );

        let mut c = TrieNode {
            contents: Some('c'),
            children: HashMap::new(),
            is_terminal: false,
        };

        c.children.insert(
            'd',
            TrieNode {
                contents: Some('d'),
                children: HashMap::new(),
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
            children: HashMap::new(),
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

        let expected: HashSet<String> = vec![String::from("bass"), String::from("bess")].iter().cloned().collect();
        let actual:HashSet<String> = trie.words(String::from("b ss")).iter().cloned().collect();
        assert_eq!(
            expected,
            actual,
        )
    }

    #[test]
    fn is_word_works() {
        let trie = Trie::build(vec![
            String::from("bass"),
            String::from("bats"),
            String::from("bess"),
            String::from("be"),
        ]);

        println!("{}", trie);

        assert!(trie.is_word("bass"));
        assert!(trie.is_word("bats"));
        assert!(trie.is_word("be"));
        assert!(!trie.is_word("bat"));
    }
}
