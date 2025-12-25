use antlr4_rust::common_token_stream::CommonTokenStream;
use antlr4_rust::input_stream::InputStream;
use antlr4_rust::token::TOKEN_EOF;

mod antlr {
    pub mod cronlexer;
}

use antlr::cronlexer::{CronLexer, CronLexerTokenType};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "0 12 * * MON-FRI echo \"hello world\" > /tmp/test".to_string()
    };

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