use std::{collections::HashMap, fmt};

#[derive(Clone)]
struct TrieNode {
    contents: Option<char>,
    children: HashMap<char, TrieNode>,
    isTerminal: bool,
}

impl TrieNode {
    fn add_sequence(mut self, chars: &str) -> TrieNode {
        match chars.as_bytes().get(0) {
            Some(val) => {
                match self.children.get(&(*val as char)) {
                    Some(child) => {
                        self.children
                            .insert(*val as char, child.clone().add_sequence(&chars[1..]));
                    }
                    None => {
                        // create child and iterate on it
                        self.children.insert(
                            *val as char,
                            TrieNode {
                                children: HashMap::new(),
                                contents: Some(*val as char),
                                isTerminal: false,
                            }
                            .add_sequence(&chars[1..]),
                        );
                    }
                }
            }
            None => {
                self.isTerminal = true;
            }
        }

        self
    }

    fn display_helper(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        depth: usize,
        firstChild: bool,
    ) -> std::result::Result<(), std::fmt::Error> {
        if !firstChild {
            for _ in 0..depth {
                write!(f, "\t")?;
            }
        } else {
            write!(f, "\t")?;
        }
        write!(f, "{}", self.contents.unwrap_or('*'))?;

        if self.isTerminal {
            write!(f, "'")?;
        }

        if self.children.len() == 0 {
            return write!(f, "\n");
        }

        for (index, key) in self.children.keys().into_iter().enumerate() {
            self.children
                .get(key)
                .unwrap()
                .display_helper(f, depth + 1, index == 0);
        }

        Ok(())
    }

    fn words(&self, pattern: String, partial: String) -> Vec<String> {
        let mut newPartial = partial.clone();
        if self.contents.is_some() {
            newPartial.push(self.contents.unwrap());
        }

        if pattern.len() == 0 {
            if self.isTerminal {
                return vec![newPartial];
            }
            return vec![];
        }

        let mut newPattern = pattern[1..].to_owned();

        let newChar = pattern.as_bytes()[0] as char;

        if newChar == ' ' {
            let mut result = vec![];
            for child in self.children.values() {
                let tmp = child.words(newPattern.clone(), newPartial.clone());
                result.extend(tmp.clone());
            }
            return result;
        }

        match self.children.get(&newChar) {
            Some(child) => {
                return child.words(newPattern, newPartial);
            }
            None => {
                return vec![];
            }
        }
    }

    fn is_word(&self, pattern: String) -> bool {
        if pattern.len() == 0 {
            return self.isTerminal;
        }

        let mut newPattern = pattern[1..].to_owned();

        let newChar = pattern.as_bytes()[0] as char;

        match self.children.get(&newChar) {
            Some(child) => child.is_word(newPattern),
            None => false,
        }
    }
}

impl fmt::Display for TrieNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.display_helper(f, 1, true)
    }
}

struct Trie {
    root: TrieNode,
}

impl fmt::Display for Trie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        self.root.fmt(f)
    }
}

impl Trie {
    fn build(words: Vec<String>) -> Trie {
        let mut root = TrieNode {
            contents: None,
            children: HashMap::new(),
            isTerminal: false,
        };

        for (index, word) in words.iter().enumerate() {
            root = root.clone().add_sequence(&word);
        }

        Trie { root }
    }

    fn words(&self, pattern: String) -> Vec<String> {
        self.root.words(pattern, String::from(""))
    }

    fn is_word(&self, pattern: String) -> bool {
        self.root.is_word(pattern)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Trie, TrieNode};

    #[test]
    fn display_works() {
        let mut root = TrieNode {
            contents: None,
            children: HashMap::new(),
            isTerminal: false,
        };

        root.children.insert(
            'b',
            TrieNode {
                contents: Some('b'),
                children: HashMap::new(),
                isTerminal: false,
            },
        );

        let mut c = TrieNode {
            contents: Some('c'),
            children: HashMap::new(),
            isTerminal: false,
        };

        c.children.insert(
            'd',
            TrieNode {
                contents: Some('d'),
                children: HashMap::new(),
                isTerminal: false,
            },
        );

        root.children.insert('c', c);

        println!("{}", root);
    }

    #[test]
    fn add_sequence_works() {
        let mut root = TrieNode {
            contents: Some('a'),
            children: HashMap::new(),
            isTerminal: false,
        };

        let newRoot = root.add_sequence("itsyaboi");

        println!("{}", newRoot);

        let anotherRoot = newRoot.add_sequence("wereallyouthere");

        println!("{}", anotherRoot)
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

        println!("{}", trie);

        assert_eq!(
            vec![String::from("bass"), String::from("bess")],
            trie.words(String::from("b ss")),
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

        assert!(trie.is_word(String::from("bass")));
        assert!(trie.is_word(String::from("bats")));
        assert!(trie.is_word(String::from("be")));
        assert!(!trie.is_word(String::from("bat")));
    }
}
