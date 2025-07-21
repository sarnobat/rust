use regex::Regex;

use std::io::{self, BufRead};

use clap::{Arg, Command};
use std::process::Command;

fn main() {
    // 4) cli options
    let matches = Command::new("example")
        .arg(Arg::new("debug").long("debug").help("Enable debug mode"))
        .get_matches();

    if matches.contains_id("debug") {
        println!("Debug mode is on");
    }
    println!("Hello ");

    // 1) stdin loop (with optional file arg)
    let stdin = io::stdin();
    let handle = stdin.lock();

    for line_result in handle.lines() {
        match line_result {
            Ok(line) => {
                if line.trim() == "exit" {
                    break;
                }
                /// 1) Print to stdout
                println!("[debug] {}", line);

                /// 3) Parse File path

                /// 2) Regex capture groups extracted and read separately
                let re = Regex::new(r"^(?P<dir>.*/)?(?P<file>[^/]+?)\.(?P<ext>[^./]+)$").unwrap();
                let path = line;
                if let Some(caps) = re.captures(&path) {
                    let dir = caps.name("dir").map_or("", |m| m.as_str());
                    let file = caps.name("file").map_or("", |m| m.as_str());
                    let ext = caps.name("ext").map_or("", |m| m.as_str());

                    println!("Directory: {}", dir);
                    println!("Filename: {}", file);
                    println!("Extension: {}", ext);
                    // 5) Read and write to a map

                    /// 6) Call a shell program instead
                    let output = Command::new("date")
                        .arg("+%s")
                        .output()
                        .expect("Failed to execute command");

                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        println!("Epoch seconds: {}", stdout.trim());
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
