use std::env;
use std::path::Path;

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
    let args: Vec<String> = env::args().collect();
    let input = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "0 12 * * MON-FRI echo \"hello world\" > /tmp/test".to_string()
    };

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
        println!("{:<12} {:?}", name, text);
        check_and_report(&text);
    }
}

#[cfg(not(has_antlr))]
fn run_with_antlr_or_fallback(input: String) {
    // Fallback: tokenizer that treats anything between single or double
    // quotes as a single token (supports backslash escapes inside quotes).
    for token in tokenize_preserving_quotes(&input) {
        println!("{}", token);
        check_and_report(&token);
    }
}

fn tokenize_preserving_quotes(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            if !cur.is_empty() {
                tokens.push(cur.clone());
                cur.clear();
            }
            continue;
        }

        if c == '"' || c == '\'' {
            // start of quoted token; include the opening quote
            let quote = c;
            cur.push(quote);
            while let Some(ch) = chars.next() {
                cur.push(ch);
                if ch == '\\' {
                    // escape next char if present
                    if let Some(esc) = chars.next() {
                        cur.push(esc);
                    }
                    continue;
                }
                if ch == quote {
                    break;
                }
            }
            tokens.push(cur.clone());
            cur.clear();
            continue;
        }

        // normal unquoted char
        cur.push(c);
    }

    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

fn check_and_report(token: &str) {
    if token.contains('/') {
        let path = Path::new(token);
        if path.exists() {
            eprintln!("File exists: {}", token);
        } else {
            eprintln!("File not found: {}", token);
        }
    } else {
        eprintln!("Not a file: {}", token);
    }
}