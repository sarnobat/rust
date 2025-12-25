use std::path::Path;
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
            let label = "[TOKEN]".bright_magenta().bold();
            eprintln!(
                "{:<7} {:>10}:{:<5} {:>32} {}",
                label,
                file!().bright_cyan(),
                line!().to_string().green(),
                t.yellow(),
                "main()".yellow()
            );
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
        let raw = token.as_str();
            let (t, had_semicolon) = if !quoted && !raw.is_empty() && raw.ends_with(';') {
                (raw.trim_end_matches(';'), true)
            } else {
                (raw, false)
            };
            let label = "[TOKEN]".bright_magenta().bold();
            eprintln!(
                "{:<7} {:>10}:{:<5} {:>32} {}",
                label,
                file!().bright_cyan(),
                line!().to_string().green(),
                t.yellow(),
                "main()".yellow()
            );
    }
}

fn tokenize_preserving_quotes(s: &str) -> Vec<(String, bool)> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        // If a comment starts at token boundary (# up to newline), capture it as single token
        if c == '#' && cur.is_empty() {
            let mut comment = String::new();
            comment.push('#');
            while let Some(ch) = chars.next() {
                if ch == '\n' {
                    break;
                }
                comment.push(ch);
            }
            tokens.push((comment, false));
            continue;
        }

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
    let t = token;

    // Skip checks for uncreated paths or URLs
    if t.contains('`') {
        eprintln!("{:<12} {:?}", "PATH_UNCREATED", t);
        return;
    }
    if t.starts_with("http:") || t.starts_with("https:") || t.starts_with("ssh:") {
        eprintln!("{:<12} {:?}", "URL", t);
        return;
    }

    // CLI args
    if t.trim_start().starts_with('-') {
        eprintln!("{:<12} {:?}", "CLI_ARG", t);
        return;
    }

    // Tilde expansion
    if t.starts_with('~') {
        match std::env::var("HOME") {
            Ok(home) => {
                let without_tilde = t.trim_start_matches('~');
                let expanded = format!("{}{}", home, without_tilde);
                let path = Path::new(&expanded);
                if path.exists() {
                    eprintln!("[trace] File exists: {} -> {}", token, expanded);
                } else {
                    eprintln!("[error] File not found: {} -> {}", token, expanded);
                }
            }
            Err(_) => {
                eprintln!("[error] Cannot expand '~' (HOME not set): {}", token);
            }
        }
        return;
    }

    // Paths containing a slash
    if t.contains('/') {
        let path = Path::new(t);
        if path.exists() {
            eprintln!("[trace] File exists: {}", token);
        } else {
            eprintln!("[error] File not found: {}", token);
        }
        return;
    }

    // Otherwise, not a file
    eprintln!("[trace] Not a file: {}", token);
}