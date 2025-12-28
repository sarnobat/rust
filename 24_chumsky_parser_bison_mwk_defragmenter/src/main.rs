use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{self, Read};

static CHUNK_START_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^===\s").expect("valid chunk regex"));

fn is_chunk_start(line: &str) -> bool {
    let trimmed = line.strip_suffix('\r').unwrap_or(line);
    CHUNK_START_RE.is_match(trimmed)
}

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut chunks: Vec<String> = Vec::new();
    let mut current_chunk: Option<String> = None;

    for segment in input.split_inclusive('\n') {
        let mut line = segment.strip_suffix('\n').unwrap_or(segment);
        line = line.strip_suffix('\r').unwrap_or(line);

        if is_chunk_start(line) {
            if let Some(chunk) = current_chunk.take() {
                chunks.push(chunk);
            }
            current_chunk = Some(String::new());
        }

        if let Some(chunk) = current_chunk.as_mut() {
            chunk.push_str(segment);
        }
    }

    if let Some(chunk) = current_chunk.take() {
        chunks.push(chunk);
    }

    println!("{chunks:#?}");

    Ok(())
}
