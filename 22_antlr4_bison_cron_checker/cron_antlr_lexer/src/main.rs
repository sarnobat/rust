use std::io::{self, Read};
use colored::Colorize;

#[cfg(has_antlr)]
use antlr4_rust::common_token_stream::CommonTokenStream;
#[cfg(has_antlr)]
use antlr4_rust::input_stream::InputStream;
#[cfg(has_antlr)]
use antlr4_rust::token::TOKEN_EOF;

#[cfg(has_antlr)]
mod antlr {
    pub mod cronlexer;
}

#[cfg(not(has_antlr))]
const MONTH_NAMES: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

#[cfg(not(has_antlr))]
const DOW_NAMES: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];

#[cfg(not(has_antlr))]
fn run_with_antlr_or_fallback(input: String) {
    for line in input.lines() {
        lex_line(line.trim_end_matches('\r'));
    }
}

#[cfg(not(has_antlr))]
fn lex_line(line: &str) {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return;
    }

    let mut idx = 0;
    while idx < line.len() {
        let rest = &line[idx..];
        let mut chars = rest.chars();
        let ch = match chars.next() {
            Some(c) => c,
            None => break,
        };

        if ch.is_whitespace() {
            idx += ch.len_utf8();
            continue;
        }

        if ch == '#' {
            break;
        }

        if let Some(token_len) = match_single_char_token(ch) {
            emit_fallback_token(single_char_token_name(ch), &rest[..token_len]);
            idx += token_len;
            continue;
        }

        if let Some((token_name, consumed)) = match_keyword(rest) {
            emit_fallback_token(token_name, &rest[..consumed]);
            idx += consumed;
            continue;
        }

        if let Some(consumed) = match_int(rest) {
            emit_fallback_token("INT", &rest[..consumed]);
            idx += consumed;
            continue;
        }

        if let Some(consumed) = match_path(rest) {
            emit_fallback_token("PATH", &rest[..consumed]);
            idx += consumed;
            continue;
        }

        emit_fallback_token("COMMAND", rest);
        break;
    }
}

#[cfg(not(has_antlr))]
fn match_single_char_token(ch: char) -> Option<usize> {
    match ch {
        '*' | '/' | ',' | '-' => Some(ch.len_utf8()),
        _ => None,
    }
}

#[cfg(not(has_antlr))]
fn single_char_token_name(ch: char) -> &'static str {
    match ch {
        '*' => "STAR",
        '/' => "SLASH",
        ',' => "COMMA",
        '-' => "DASH",
        _ => "COMMAND",
    }
}

#[cfg(not(has_antlr))]
fn match_keyword(input: &str) -> Option<(&'static str, usize)> {
    let mut end = 0;
    for (offset, c) in input.char_indices() {
        if c.is_ascii_alphabetic() {
            end = offset + c.len_utf8();
        } else {
            break;
        }
    }

    if end == 0 {
        return None;
    }

    let candidate = &input[..end];
    let upper = candidate.to_ascii_uppercase();

    if MONTH_NAMES.iter().any(|name| *name == upper) {
        Some(("MONTH_NAME", end))
    } else if DOW_NAMES.iter().any(|name| *name == upper) {
        Some(("DOW_NAMEa", end))
    } else {
        None
    }
}

#[cfg(not(has_antlr))]
fn match_int(input: &str) -> Option<usize> {
    let mut end = 0;
    for (offset, c) in input.char_indices() {
        if c.is_ascii_digit() {
            end = offset + c.len_utf8();
        } else {
            break;
        }
    }

    if end == 0 { None } else { Some(end) }
}

#[cfg(not(has_antlr))]
fn match_path(input: &str) -> Option<usize> {
    let mut chars = input.chars();
    let first = chars.next()?;

    if first == '"' || first == '\'' {
        return consume_quoted_path(input, first);
    }

    if first == '~' {
        return Some(take_until_whitespace(input));
    }

    let len = take_until_whitespace(input);
    if len == 0 {
        return None;
    }

    let segment = &input[..len];
    if segment.contains('/') {
        Some(len)
    } else {
        None
    }
}

#[cfg(not(has_antlr))]
fn consume_quoted_path(input: &str, quote: char) -> Option<usize> {
    let mut escaped = false;
    for (offset, c) in input.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }

        if c == '\\' {
            escaped = true;
            continue;
        }

        if c == quote {
            return Some(offset + c.len_utf8());
        }
    }

    None
}

#[cfg(not(has_antlr))]
fn take_until_whitespace(input: &str) -> usize {
    for (offset, c) in input.char_indices() {
        if c.is_whitespace() {
            return offset;
        }
    }

    input.len()
}

#[cfg(not(has_antlr))]
fn emit_fallback_token(token_name: &str, text: &str) {
    if text.trim_start().starts_with('#') {
        return;
    }

    let trimmed = if !text.is_empty() && text.ends_with(';') {
        text.trim_end_matches(';')
    } else {
        text
    };

    let tt_label = format!("[{}]", token_name).bright_magenta().bold();
    eprintln!(
        "{:<7} {:>10}:{:<5} {:>32} {}",
        tt_label,
        file!().bright_cyan(),
        line!().to_string().green(),
        trimmed.yellow(),
        "main()".yellow()
    );
}

#[cfg(has_antlr)]
use antlr::cronlexer::{CronLexer, CronLexerTokenType};

fn main() {
    // Read all input from stdin
    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("Failed to read stdin: {}", e);
        std::process::exit(1);
    }

    // If stdin is empty, do nothing
    if input.is_empty() {
        return;
    }

    run_with_antlr_or_fallback(input);
}

#[cfg(has_antlr)]
fn run_with_antlr_or_fallback(input: String) {
    let input_stream = InputStream::new(input.as_str());
    let mut lexer = CronLexer::new(input_stream);
    let mut tokens = CommonTokenStream::new(&mut lexer);
    tokens.fill();

    for t in tokens.get_all_tokens() {
        if t.get_token_type() == TOKEN_EOF {
            break;
        }
        let tt = t.get_token_type() as isize;
        let name = CronLexerTokenType::to_string(tt);
        let text = t.get_text().unwrap_or_default();
        if text.trim_start().starts_with('#') {
            // comment token: skip entirely (no stdout or stderr)
            continue;
        }
        let raw = text.as_str();
            let (t, had_semicolon) = if !raw.is_empty() && raw.ends_with(';') {
                (raw.trim_end_matches(';'), true)
            } else {
                (raw, false)
            };
            let tt_label = format!("[{}]", name).bright_magenta().bold();
            eprintln!(
                "{:<7} {:>10}:{:<5} {:>32} {}",
                tt_label,
                file!().bright_cyan(),
                line!().to_string().green(),
                t.yellow(),
                "main()".yellow()
            );
    }
}
