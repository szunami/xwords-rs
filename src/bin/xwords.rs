extern crate clap;
use std::{fs::File, sync::Arc};

use clap::{App, Arg};
use xwords::{
    crossword::Crossword,
    default_indexes,
    fill::{
        parallel::ParallelFiller, simple::SimpleFiller, single_threaded::SingleThreadedFiller,
        Filler,
    },
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
            Arg::with_name("algorithm")
                .short("a")
                .long("algorithm")
                .value_name("ALGORITHM")
                .possible_values(&["simple", "single_threaded", "parallel"])
                .default_value("single_threaded"),
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

    let algorithm = matches
        .value_of("algorithm")
        .expect("failed to load algorithm");

    if matches.is_present("profile") {
        let guard = pprof::ProfilerGuard::new(1000).unwrap();
        std::thread::spawn(move || loop {
            match guard.report().build() {
                Ok(report) => {
                    let file = File::create("flamegraph.svg").unwrap();
                    report.flamegraph(file).unwrap();
                }
                Err(_) => {}
            };
            std::thread::sleep(std::time::Duration::from_secs(5))
        });
    }

    let output = match algorithm {
        "single_threaded" => {
            let (bigrams, trie) = default_indexes();
            SingleThreadedFiller::new(&trie, &bigrams).fill(&input)
        }
        "simple" => {
            let (_bigrams, trie) = default_indexes();
            SimpleFiller::new(&trie).fill(&input)
        }
        "parallel" => {
            let (bigrams, trie) = default_indexes();
            ParallelFiller::new(Arc::new(trie), Arc::new(bigrams)).fill(&input)
        }
        _ => {
            return Err(String::from("Failed to parse algorithm"));
        }
    };

    match output {
        Ok(output) => {
            println!("{}", output);
        }
        Err(_) => return Err(String::from("Failed to fill crossword")),
    }
    Ok(())
}
