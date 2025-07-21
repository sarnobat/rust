use clap::{Arg, Command};
use regex::Regex;
use std::collections::HashMap;
use std::io::{self, BufRead};
// use std::fs;
// use std::path::Path;

fn main() {
    // 4) cli options
    {
        let matches = Command::new("example")
            .arg(Arg::new("debug").long("debug").help("Enable debug mode"))
            .get_matches();

        if matches.contains_id("debug") {
            println!("Debug mode is on");
        }
    }

    let mut counts = HashMap::new();

    // 1) stdin loop (with optional file arg)
    let stdin = io::stdin();
    let handle = stdin.lock();

    for line_result in handle.lines() {
        match line_result {
            Ok(line) => {
                if line.trim() == "exit" {
                    break;
                }
                // 1) Print to stdout
                println!("[debug] {}", line);

                // 3) Parse File path

                // 2) Regex capture groups extracted and read separately
                let re = Regex::new(r"^(?P<dir>.*/)?(?P<file>[^/]+?)\.(?P<ext>[^./]+)$").unwrap();
                // let path = line;
                if let Some(caps) = re.captures(&line) {
                    let dir = caps.name("dir").map_or("", |m| m.as_str()).to_string();
                    let file = caps.name("file").map_or("", |m| m.as_str());
                    let ext = caps.name("ext").map_or("", |m| m.as_str()).to_string();

                    println!("\tDirectory: {}", dir);
                    println!("\tFilename: {}", file);
                    println!("\tExtension: {}", ext);

                    // 5) Read and write to a map
                    *counts.entry(ext).or_insert(0) += 1;

                    println!("{:?}", counts);

                    // 6) Call a shell program instead
                    let output = std::process::Command::new("date")
                        .arg("+%s")
                        .output()
                        .expect("Failed to execute command");

                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        println!("\tEpoch seconds: {}", stdout.trim());
                    } else {
                        eprintln!("Command failed with status: {:?}", output.status);
                    }

                    // 11) create json object
                } else {
                    println!("No match");
                }
            }
            Err(err) => {
                eprintln!("Error reading line: {}", err);
                break;
            }
        }
    }
}
