extern crate rustc_hash;
use rustc_hash::FxHashMap;

pub fn bigrams(words: &[String]) -> FxHashMap<(char, char), usize> {
    let mut result = FxHashMap::default();

    for word in words {
        for bigram in word
            .chars()
            .into_iter()
            .zip(word.chars().skip(1).into_iter())
        {
            let count = result.entry(bigram).or_insert(0);
            *count += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use std::{fs::File, time::Instant};

    use crate::ngram::bigrams;

    #[test]
    fn it_works() {
        let input = vec![String::from("ABC"), String::from("ABRACADABRA")];
        let actual = bigrams(&input);
        let key = ('A', 'B');
        assert_eq!(3, *actual.get(&key).unwrap());
        let key = ('R', 'A');
        assert_eq!(2, *actual.get(&key).unwrap());
    }

    #[test]
    fn it_works_bigly() {
        println!("Building bigrams");
        let now = Instant::now();

        let file = File::open("wordlist.json").unwrap();

        let words: Vec<String> =
            serde_json::from_reader(file).expect("JSON was not well-formatted");
        println!("Done parsing file");
        let result = bigrams(&words);
        println!(
            "Done building bigrams in {} seconds",
            now.elapsed().as_secs()
        );

        let mut z: Vec<((char, char), usize)> = result.into_iter().collect();
        z.sort_by(|(_, count_a), (_, count_b)| count_b.cmp(count_a));

        let top_5: Vec<((char, char), usize)> = z.into_iter().take(5).collect();

        println!("{:?}", top_5);
    }
}
