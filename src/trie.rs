use crate::File;
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

    fn words<T: Iterator<Item = char> + Clone>(
        &self,
        mut pattern: T,
        partial: &mut String,
        result: &mut Vec<String>,
    ) {
        if self.contents.is_some() {
            partial.push(self.contents.unwrap());
        }

        match pattern.next() {
            Some(new_char) => {
                if new_char == ' ' {
                    for child in self.children.values() {
                        child.words(pattern.clone(), partial, result);
                    }
                } else {
                    if let Some(child) = self.children.get(&new_char) {
                        child.words(pattern, partial, result);
                    }
                }
            }
            None => {
                if self.is_terminal {
                    result.push(partial.clone());
                }
            }
        }

        if self.contents.is_some() {
            partial.pop();
        }
    }

    pub fn is_viable<T: Iterator<Item = char> + Clone>(&self, mut chars: T) -> bool {
        match chars.next() {
            None => self.is_terminal,

            Some(c) => {
                if c == ' ' {
                    for child in self.children.values() {
                        if child.is_viable(chars.clone()) {
                            return true;
                        }
                    }
                    false
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
    pub fn load_default() -> Result<Trie, String> {
        let file = File::open("./trie.bincode").unwrap();
        let load = bincode::deserialize_from::<File, Trie>(file);
        load.map_err(|_| String::from("Failed to load trie."))
    }

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

    pub fn words<T: Iterator<Item = char> + Clone>(&self, pattern: T) -> Vec<String> {
        let mut result = Vec::with_capacity(4);
        let mut partial = String::with_capacity(4);
        self.root.words(pattern, &mut partial, &mut result);
        result
    }

    pub fn is_viable<T: Iterator<Item = char> + Clone>(&self, chars: T) -> bool {
        self.root.is_viable(chars)
    }
}

#[cfg(test)]
mod tests {

    use crate::File;
    use rustc_hash::FxHashMap;

    use std::collections::HashSet;

    use super::{Trie, TrieNode};

    #[test]
    #[ignore]
    fn rebuild_serialized_trie() {
        let file = File::open("wordlist.json").unwrap();
        let words = serde_json::from_reader(file).expect("JSON was not well-formatted");
        let trie = Trie::build(words);
        let trie_file = File::create("trie.bincode").unwrap();
        let trie_result = bincode::serialize_into(trie_file, &trie);
        assert!(trie_result.is_ok());
    }

    #[test]
    fn test_trie_load() {
        let file = File::open("./trie.bincode").unwrap();
        let load = bincode::deserialize_from::<File, Trie>(file);
        assert!(load.is_ok());
    }

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

        let iter = String::from("b ss");
        let actual: HashSet<String> = trie.words(iter.chars()).iter().cloned().collect();
        assert_eq!(expected, actual,)
    }
}
