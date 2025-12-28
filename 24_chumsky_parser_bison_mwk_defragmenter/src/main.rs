use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{self, Read};

static CHUNK_START_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^===\s").expect("valid chunk regex"));
static HASH_SORT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n#").expect("valid hash chunk regex"));

fn is_chunk_start(line: &str) -> bool {
    let trimmed = line.strip_suffix('\r').unwrap_or(line);
    CHUNK_START_RE.is_match(trimmed)
}

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let (prefix, mut snippets, suffix) = extract_sections(&input);

    if !snippets.is_empty() && suffix.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing closing chunk delimiter",
        ));
    }

    if snippets.is_empty() {
        print!("{prefix}{suffix}");
        return Ok(());
    }

    snippets.sort_by_key(|snippet| !HASH_SORT_RE.is_match(snippet));

    print!("{prefix}");
    for snippet in &snippets {
        print!("{snippet}");
    }
    print!("{suffix}");

    Ok(())
}

fn extract_sections(input: &str) -> (String, Vec<String>, String) {

    //
    // Find the starting line numbers for each snippet
    //

    let mut chunk_starts: Vec<usize> = Vec::new();
    let mut offset = 0usize;

    for segment in input.split_inclusive('\n') {
        let mut line = segment.strip_suffix('\n').unwrap_or(segment);
        line = line.strip_suffix('\r').unwrap_or(line);

        if is_chunk_start(line) {
            chunk_starts.push(offset);
        }

        offset += segment.len();
    }

    if chunk_starts.is_empty() {
        return (input.to_string(), Vec::new(), String::new());
    }

    //
    // Find the start and end line numbers of each snippet
    //
    let mut chunk_ranges: Vec<(usize, usize)> = Vec::with_capacity(chunk_starts.len());
    for (idx, &start) in chunk_starts.iter().enumerate() {
        let end = if idx + 1 < chunk_starts.len() {
            // store the start of the next snippet as the end of this one
            chunk_starts[idx + 1]
        } else {
            // last snippet ending line is the length of the file
            input.len()
        };
        chunk_ranges.push((start, end));
    }

    //
    // Do not move the top chunk before the first heading-3 snippet anywhere
    //
    let prefix_end = chunk_starts[0];
    let prefix = input[..prefix_end].to_string();

    //
    // Extract all chunks except the first and last
    //
    let mut chunks: Vec<_> = chunk_ranges
        .iter()
        .map(|&(start, end)| input[start..end].to_string())
        .collect();

    if chunks.is_empty() {
        return (prefix, Vec::new(), String::new());
    }

    //
    // Do not move the bottom chunk after the last heading-3 snippet anywhere
    //
    let suffix = chunks.pop().unwrap();

    (prefix, chunks, suffix)
}
