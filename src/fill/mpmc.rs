use crate::{crossword::Crossword, Filler};

pub struct MPMCFiller {}

impl Filler for MPMCFiller {
    fn fill(self, _: &Crossword) -> std::result::Result<Crossword, String> {
        Err(String::from("Unable to do the thing :("))
    }
}

#[cfg(test)]
mod tests {
    use super::MPMCFiller;
    use crate::{fill::Filler, Crossword};

    #[test]
    fn fill_crossword_works() {
        let filler = MPMCFiller {};

        let input = Crossword::new(String::from("                ")).unwrap();

        let result = filler.fill(&input);

        assert!(result.is_ok());

        println!("{}", result.unwrap());
    }
}
