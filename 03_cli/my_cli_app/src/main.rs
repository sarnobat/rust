// use clap::{App, Arg};
use std::io::{self, BufRead};

fn main() {
    let matches = App::new("My CLI App")
        .version("1.0")
        .author("Sridhar Sarnobat")
        .about("A simple CLI tool written in rust")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Enables verbose mode"),
        )
        .get_matches();

    let input_file = matches.value_of("input").unwrap();
    println!("Input file: {}", input_file);

    if matches.is_present("verbose") {
        println!("Verbose mode enabled");
    }
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                println!("Read line: {}", line);
            }
            Err(error) => {
                eprintln!("Error reading line: {}", error);
                break; // Or handle the error differently
            }
        }
    }
}
