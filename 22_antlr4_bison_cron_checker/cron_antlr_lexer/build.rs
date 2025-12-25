use antlr4_rust::build::*;

fn main() {
    let mut config = BuildConfig::default();
    config.out_dir = "src/antlr".into();
    generate_lexer(&config, &["grammars/CronLexer.g4"]);
}