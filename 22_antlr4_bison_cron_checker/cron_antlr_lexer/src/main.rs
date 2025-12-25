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

#[cfg(not(has_antlr))]
fn run_with_antlr_or_fallback(_input: String) {
    eprintln!("ERROR: No generated ANTLR lexer found (src/antlr/cronlexer.rs).\nPlease generate the lexer from grammars/CronLexer.g4 or enable ANTLR build support and re-run the build.");
    std::process::exit(2);
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

