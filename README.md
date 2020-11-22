# xwords

![](https://github.com/szunami/xwords-rs/workflows/Build/badge.svg)
[![](http://meritbadge.herokuapp.com/xwords)](https://crates.io/crates/xwords)

`xwords` is a library with utilities to fill crossword puzzles.

### Caveat Emptor
This is foremost a hobbyist project for me to learn a bit about profiling and optimizing rust. I am more than happy to accept contributions or to consider feature requests, but please be aware that the future of this project is somewhat uncertain.

## Quickstart

On my machine, the below snippet runs in about 3 seconds.

```rust
use xwords::{crossword::Crossword, fill_crossword_with_default_wordlist};

fn main() -> Result<(), String> {
    let empty_crossword = Crossword::new(String::from(
        "
    *    *     
    *    *     
         *     
   *   *   *   
**    *        
      *     ***
     *    *    
   *       *   
    *    *     
***     *      
        *    **
   *   *   *   
     *         
     *    *    
     *    *    
",
    ))?;
    let filled_crossword = fill_crossword_with_default_wordlist(&empty_crossword)?;
    println!("{}", filled_crossword);
    Ok(())
}

/*
ZETA*TWIT*VOWEL
ETAT*IANA*EVOKE
RINTINTIN*REVIE
OCT*TIE*TUI*ENR
**ATHA*TASTINGS
TOLEAN*ILIES***
ISIAC*TEAN*STEM
ZAT*ACHATES*HRA
AYES*SETE*TYEES
***TUTSI*URALIC
VENERATE*SEWA**
ORA*TRO*UES*TOA
WISHI*NETASSETS
ETHIC*EVIL*USTO
RUEDA*SWAL*OTSU
*/
```

Behind the scenes, this snippet loads an indexed wordlist, and iteratively fills the input with valid words.
