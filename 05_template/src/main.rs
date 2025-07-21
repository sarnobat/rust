use std::io::{self, BufRead};

use clap::{Arg, Command};

fn main() {
    let matches = Command::new("example")
        .arg(Arg::new("debug").long("debug").help("Enable debug mode"))
        .get_matches();

    if matches.contains_id("debug") {
        println!("Debug mode is on");
    }
    println!("Hello ");
    let stdin = io::stdin();
    let handle = stdin.lock();

    for line_result in handle.lines() {
        match line_result {
            Ok(line) => {
                if line.trim() == "exit" {
                    break;
                }
                println!("You typed: {}", line);
            }
            Err(err) => {
                eprintln!("Error reading line: {}", err);
                break;
            }
        }
    }
}
