use std::collections::HashMap;

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
                        self.children.insert(*val as char, child.clone().add_sequence(&chars[1..]));
                    }
                    None => {
                        // create child and iterate on it
                        self.children.insert(*val as char, TrieNode{
                            children: HashMap::new(),
                            contents: Some(*val as char),
                            isTerminal: false,
                        }.add_sequence(&chars[1..]));
                    }
                }
            }
            None => {}
        }

        self
    }
}

struct Trie {
    root: TrieNode,
}

impl Trie {
    fn build(words: Vec<String>) -> Trie {
        todo!();
        // let root = TrieNode {
        //     contents: None,
        //     children: HashMap::new(),
        //     isTerminal: false,
        // };

        // let word = words.first().unwrap();

        // let b = word.as_bytes().first().unwrap();

        // let newNode = TrieNode {
        //     contents: Some(*b as char),
        //     children: HashMap::new(),
        //     isTerminal: false,
        // };
        // root.add_child(newNode);

        // for word in words {
        //     let curr = root;

        //     let b = word.as_bytes().first().unwrap();

            // for letter in word.as_bytes() {
            //     let newNode = TrieNode {
            //         contents: Some(*letter as char),
            //         children: HashMap::new(),
            //         isTerminal: false,
            //     };

            //     curr.add_child(newNode);
            //     curr = newNode;
            // }
        // }

        // Trie {
        //     root,
        // }
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
    fn it_works() {
        let root = TrieNode{
            contents: None,
            children: HashMap::new(),
            isTerminal: false,
        };

        root.add_sequence("asdf");


        // let trie = Trie::build(vec![String::from("cat")]);

        // assert!(trie.is_word(String::from("cat")));
        // assert!(
        //     !trie.is_word(String::from("cbt"))
        // );

        // assert_eq!(
        //     vec![String::from("cat")],
        //     trie.words(String::from("c t"))
        // );
    }
}
