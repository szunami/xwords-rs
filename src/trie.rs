use std::collections::HashMap;

struct TrieNode {
    contents: Option<char>,
    children: HashMap<char, TrieNode>,
    isTerminal: bool,
}

impl TrieNode {

    fn add_child(mut self, node: TrieNode) {
        if self.children.get(&node.contents.unwrap()).is_none() {
            self.children.insert(node.contents.unwrap(), node);
        }
    }
}

struct Trie {
    root: TrieNode
}

impl Trie {
    fn build(words: Vec<String>) -> Trie {

        let root = TrieNode{contents: None, children: HashMap::new(), isTerminal: false};

        for word in words {

            let curr = root;

            for letter in word.as_bytes() {

                let newNode = TrieNode{
                    contents: Some(*letter as char),
                    children: HashMap::new(),
                    isTerminal: false,
                };

                curr.add_child(newNode);
            }

        }

        todo!()
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
    use super::Trie;

    #[test]
    fn it_works() {
        let trie = Trie::build(vec![
            String::from("cat")
        ]);

        assert!(
            trie.is_word(String::from("cat"))
        );
        // assert!(
        //     !trie.is_word(String::from("cbt"))
        // );

        // assert_eq!(
        //     vec![String::from("cat")],
        //     trie.words(String::from("c t"))
        // );


    }

}