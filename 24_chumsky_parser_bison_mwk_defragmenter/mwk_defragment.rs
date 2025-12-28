use std::io::{self, Read};

fn is_chunk_start(line: &str) -> bool {
    let trimmed = line.strip_suffix('\r').unwrap_or(line);
    if !trimmed.starts_with("===") {
        return false;
    }

    trimmed.chars().nth(3).map(|c| c.is_whitespace()).unwrap_or(false)
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
