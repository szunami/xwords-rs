use xwords::{crossword::Crossword, fill_crossword_with_default_wordlist};

fn main() -> Result<(), String> {
    let empty_crossword = Crossword::square(String::from(
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
