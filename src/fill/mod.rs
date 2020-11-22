use crate::Crossword;

pub mod parallel;
pub mod single_threaded;

pub trait Filler {
    fn fill(&self, crossword: &Crossword) -> Result<Crossword, String>;
}
