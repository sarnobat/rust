use std::io::{self, BufRead};

fn main() {

    ///
    /// Iterate over stdin
    ///
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

