use std::{collections::HashMap, fmt};

#[derive(Clone)]
struct TrieNode {
    contents: Option<char>,
    children: HashMap<char, TrieNode>,
    isTerminal: bool,
}

impl TrieNode {
    fn add_child<'s>(&'s mut self, val: char, isTerminal: bool) -> &'s TrieNode {
        todo!();

        // match self.children.get(&val) {

        //     Some(child) => {
        //         // child.isTerminal = child.isTerminal || isTerminal;
        //         child
        //     }
        //     None => {
        //         let newNode = TrieNode{
        //             contents: Some(val),
        //             children: HashMap::new(),
        //             isTerminal,
        //         };
        //         self.children.insert(val, newNode);
        //         &newNode
        //     }
        // }

        // if self.children.get(&val).is_none() {

        // }
        // self.children.get(&val)
    }

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
            None => {}
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

    fn words(self, pattern: String) -> Vec<String> {
        todo!()
    }

    fn is_word(self, pattern: String) -> bool {
        todo!()
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
                String::from("asdf"), String::from("asset"),
                String::from("bass"), String::from("baseball"), String::from("bassooon"), String::from("basset"),
                ])
        );
    }
}
