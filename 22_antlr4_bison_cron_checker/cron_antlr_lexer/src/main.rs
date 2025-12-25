use std::env;

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
        println!("{:<12} {:?}", name, t.get_text().unwrap_or_default());
    }
}

#[cfg(not(has_antlr))]
fn run_with_antlr_or_fallback(input: String) {
    // Fallback: simple whitespace tokenizer — prints each token on its own line
    for token in input.split_whitespace() {
        println!("{}", token);
    }
}