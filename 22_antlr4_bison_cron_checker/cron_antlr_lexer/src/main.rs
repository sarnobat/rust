use std::env;
use std::path::Path;
use std::io::{self, Read};

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
        eprintln!("{:<12} {:?}", name, text);
        check_and_report(&text);
    }
}

#[cfg(not(has_antlr))]
fn run_with_antlr_or_fallback(input: String) {
    // Fallback: tokenizer that treats anything between single or double
    // quotes as a single token (supports backslash escapes inside quotes).
    for (token, quoted) in tokenize_preserving_quotes(&input) {
        if !quoted && token.trim_start().starts_with('#') {
            // comment token: skip entirely (no stdout or stderr)
            continue;
        }
        eprintln!("{}", token);
        check_and_report(&token);
    }
}

fn tokenize_preserving_quotes(s: &str) -> Vec<(String, bool)> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            if !cur.is_empty() {
                tokens.push((cur.clone(), false));
                cur.clear();
            }
            continue;
        }

        if c == '"' || c == '\'' {
            // start of quoted token; collect inner content, handle escapes
            let quote = c;
            let mut inner = String::new();
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    if let Some(esc) = chars.next() {
                        // interpret common escapes (keep literal for simplicity)
                        inner.push(esc);
                    }
                    continue;
                }
                if ch == quote {
                    break;
                }
                inner.push(ch);
            }
            tokens.push((inner, true));
            continue;
        }

        // normal unquoted char
        cur.push(c);
    }

    if !cur.is_empty() {
        tokens.push((cur, false));
    }
    tokens
}

fn check_and_report(token: &str) {
    if token.contains('/') {
        let path = Path::new(token);
        if path.exists() {
            eprintln!("[trace] File exists: {}", token);
        } else {
            println!("[error] File not found: {}", token);
        }
    } else {
        eprintln!("[trace] Not a file: {}", token);
    }
}