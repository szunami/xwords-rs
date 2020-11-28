extern crate clap;
use std::fs::File;
use xwords::trie::Trie;

use clap::{App, Arg};
use xwords::{
    crossword::Crossword,
    fill::{simple::SimpleFiller, Filler},
};

fn main() -> Result<(), String> {
    let matches = App::new("xwords")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Input crossword location")
                .required(true),
        )
        .arg(
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .takes_value(false),
        )
        .get_matches();

    let input = matches.value_of("input").expect("input not included");
    let input = std::fs::read_to_string(input).expect("failed to read input");
    let input = Crossword::new(input).expect("failed to parse input");

    if matches.is_present("profile") {
        let guard = pprof::ProfilerGuard::new(100).unwrap();
        std::thread::spawn(move || loop {
            if let Ok(report) = guard.report().build() {
                let file = File::create("flamegraph.svg").unwrap();
                report.flamegraph(file).unwrap();
            }
            std::thread::sleep(std::time::Duration::from_secs(5))
        });
    }

    let trie = Trie::load_default().expect("Failed to load trie");
    let output = SimpleFiller::new(&trie).fill(&input);

    match output {
        Ok(output) => {
            println!("{}", output);
        }
        Err(_) => return Err(String::from("Failed to fill crossword")),
    }
    Ok(())
}
