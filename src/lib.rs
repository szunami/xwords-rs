use std::fmt;

#[derive(PartialEq, Debug``)]
struct Crossword {
    contents: String,
    width: usize,
    height: usize,
}

impl Clone for Crossword {
    fn clone(&self) -> Self {
        Crossword {
            contents: self.contents.clone(),
            width: self.width,
            height: self.height,
        }
    }
}

impl fmt::Display for Crossword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                write!(
                    f,
                    "{}",
                    self.contents.as_bytes()[row * self.width + col] as char
                );
                if col != self.width - 1 {
                    write!(f, " ");
                }
            }
            write!(f, "\n");

            if row != self.height - 1 {
                write!(f, "\n");
            }
        }
        Ok(())
    }
}

fn fill_crossword(crossword: &Crossword) -> Crossword {
    crossword.clone()
}

#[cfg(test)]
mod tests {
    use crate::{fill_crossword, Crossword};

    #[test]
    fn it_works() {
        let c = Crossword {
            contents: String::from("abcdefghi"),
            width: 3,
            height: 3,
        };

        println!("{}", c);
    }

    #[test]
    fn fill_works() {
        let c = Crossword {
            contents: String::from("         "),
            width: 3,
            height: 3,
        };
        assert_eq!(fill_crossword(&c), c);
    }
}
