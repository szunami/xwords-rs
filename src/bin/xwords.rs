extern crate clap;
use xwords::fill::Fill;
use std::{fs::File};
use xwords::trie::Trie;

use clap::{App, Arg};
use xwords::{
    crossword::Crossword,
    fill::filler::Filler,
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
            Arg::with_name("width")
                .short("w")
                .long("width")
                .value_name("HEIGHT")
                .help("Input crossword height. Required if input is not a square"),
        )
        .arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .value_name("HEIGHT")
                .help("Input crossword height. Required if input is not a square"),
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

    let input = match (matches.value_of("width"), matches.value_of("height")) {
        (Some(width), Some(height)) => {
            let width = width.parse().expect("Failed to parse width");
            let height = height.parse().expect("Failed to parse height");
            Crossword::rectangle(input, width, height).expect("Failed to parse crossword")
        }
        (None, None) => Crossword::square(input).expect("Failed to parse crossword"),
        (None, Some(_)) => return Err(String::from("Width specified but not height.")),
        (Some(_), None) => return Err(String::from("Height specified but not width.")),
    };

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
    let output = Filler::new(&trie).fill(&input);

    match output {
        Ok(output) => {
            println!("{}", output);
        }
        Err(_) => return Err(String::from("Failed to fill crossword")),
    }
    Ok(())
}
