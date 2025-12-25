use chumsky::prelude::*;
use colored::Colorize;
use std::{env, io::{self, Read}, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Star,
    Slash,
    Comma,
    Dash,
    MonthName(String),
    DowName(String),
    Int(i64),
    HttpUrl(String),
    SshUrl(String),
    Url(String),
    Path(String),
    StringLiteral(String),
    CliOption(String),
    Program(String),
}

fn cron_lexer() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let month = choice((
        just("JAN").map(|_| Token::MonthName("JAN".to_string())),
        just("FEB").map(|_| Token::MonthName("FEB".to_string())),
        just("MAR").map(|_| Token::MonthName("MAR".to_string())),
        just("APR").map(|_| Token::MonthName("APR".to_string())),
        just("MAY").map(|_| Token::MonthName("MAY".to_string())),
        just("JUN").map(|_| Token::MonthName("JUN".to_string())),
        just("JUL").map(|_| Token::MonthName("JUL".to_string())),
        just("AUG").map(|_| Token::MonthName("AUG".to_string())),
        just("SEP").map(|_| Token::MonthName("SEP".to_string())),
        just("OCT").map(|_| Token::MonthName("OCT".to_string())),
        just("NOV").map(|_| Token::MonthName("NOV".to_string())),
        just("DEC").map(|_| Token::MonthName("DEC".to_string())),
    ));

    let dow = choice((
        just("SUN").map(|_| Token::DowName("SUN".to_string())),
        just("MON").map(|_| Token::DowName("MON".to_string())),
        just("TUE").map(|_| Token::DowName("TUE".to_string())),
        just("WED").map(|_| Token::DowName("WED".to_string())),
        just("THU").map(|_| Token::DowName("THU".to_string())),
        just("FRI").map(|_| Token::DowName("FRI".to_string())),
        just("SAT").map(|_| Token::DowName("SAT".to_string())),
    ));

    let int = text::int(10).map(|s: String| {
        let value = s
            .parse::<i64>()
            .expect("text::int(10) only produces digit strings");
        Token::Int(value)
    });

    let url_tail = none_of(" \t\r\n#")
        .repeated()
        .at_least(1)
        .collect::<String>();

    let http_url = choice((just("https://"), just("http://")))
        .then(url_tail.clone())
        .map(|(prefix, rest)| Token::HttpUrl(format!("{prefix}{rest}")));

    let ssh_url = just("ssh://")
        .then(url_tail.clone())
        .map(|(prefix, rest)| Token::SshUrl(format!("{prefix}{rest}")));

    let other_url = just("ftp://")
        .then(url_tail.clone())
        .map(|(prefix, rest)| Token::Url(format!("{prefix}{rest}")));

    let escaped_char = just('\\').ignore_then(any());

    let dq_inner = choice((escaped_char.clone(), none_of("\\\"\r\n")));
    let sq_inner = choice((escaped_char.clone(), none_of("\\'\r\n")));

    let quoted_string = choice((
        dq_inner
            .repeated()
            .collect::<String>()
            .delimited_by(just('"'), just('"')),
        sq_inner
            .repeated()
            .collect::<String>()
            .delimited_by(just('\''), just('\'')),
    ));

    let non_ws = filter(|c: &char| !c.is_whitespace() && *c != ';');
    let non_ws_no_slash =
        filter(|c: &char| !c.is_whitespace() && *c != '/' && *c != ';');
    let rel_first_char = filter(|c: &char| {
        !c.is_whitespace() && *c != '/' && *c != '*' && *c != ';'
    });
    let abs_first_char =
        filter(|c: &char| {
            !c.is_whitespace() && *c != '/' && *c != ';' && !c.is_ascii_digit()
        });

    let tilde_path = just('~')
        .then(non_ws.repeated())
        .map(|(tilde, rest)| {
            let mut s = String::new();
            s.push(tilde);
            for c in rest {
                s.push(c);
            }
            Token::Path(s)
        });

    let abs_path = just('/')
        .then(
            abs_first_char
                .then(non_ws_no_slash.repeated())
                .map(|(head, rest)| {
                    let mut segment = Vec::new();
                    segment.push(head);
                    segment.extend(rest);
                    segment
                }),
        )
        .then(
            just('/')
                .then(non_ws_no_slash.repeated())
                .repeated(),
        )
        .map(|((first_slash, first_segment), tail)| {
            let mut s = String::new();
            s.push(first_slash);
            for c in first_segment {
                s.push(c);
            }
            for (slash, segment) in tail {
                s.push(slash);
                for c in segment {
                    s.push(c);
                }
            }
            Token::Path(s)
        });

    let rel_path = rel_first_char
        .then(non_ws_no_slash.repeated())
        .map(|(head, rest)| {
            let mut segment = Vec::new();
            segment.push(head);
            segment.extend(rest);
            segment
        })
        .then(
            just('/')
                .then(non_ws_no_slash.repeated())
                .repeated()
                .at_least(1),
        )
        .map(|(first, tail)| {
            let mut s = String::new();
            for c in first {
                s.push(c);
            }
            for (slash, segment) in tail {
                s.push(slash);
                for c in segment {
                    s.push(c);
                }
            }
            Token::Path(s)
        });

    let path = choice((tilde_path, abs_path, rel_path));
    let string_literal = quoted_string.map(Token::StringLiteral);

    let opt_char = filter(|c: &char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'));
    let long_opt = just::<char, &str, Simple<char>>("--")
        .ignore_then(filter(|c: &char| c.is_ascii_alphanumeric()))
        .then(opt_char.repeated())
        .map(|(first, rest)| {
            let mut s = String::from("--");
            s.push(first);
            for c in rest {
                s.push(c);
            }
            Token::CliOption(s)
        });

    let program = filter(|c: &char| !c.is_whitespace() && *c != '#' && *c != '/')
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(Token::Program);

    let token = choice((
        http_url,
        ssh_url,
        other_url,
        string_literal,
        path,
        long_opt,
        just('*').to(Token::Star),
        just('/').to(Token::Slash),
        just(',').to(Token::Comma),
        just('-').to(Token::Dash),
        month,
        dow,
        int,
        program,
    ))
    .boxed();

    let comment = just('#')
        .ignore_then(none_of("\r\n").repeated())
        .ignored();

    let skip = choice((filter(|c: &char| c.is_whitespace()).ignored(), comment))
        .repeated()
        .ignored();

    token
        .padded_by(skip.clone())
        .repeated()
        .then_ignore(skip)
        .then_ignore(end())
}

fn token_label(token: &Token) -> &'static str {
    match token {
        Token::Star => "STAR",
        Token::Slash => "SLASH",
        Token::Comma => "COMMA",
        Token::Dash => "DASH",
        Token::MonthName(_) => "MONTH",
        Token::DowName(_) => "DOW",
        Token::Int(_) => "INT",
        Token::HttpUrl(_) => "HTTP",
        Token::SshUrl(_) => "SSH",
        Token::Url(_) => "URL",
        Token::Path(_) => "PATH",
        Token::StringLiteral(_) => "STRING",
        Token::CliOption(_) => "OPTION",
        Token::Program(_) => "PROGRAM",
    }
}

fn main() {
    let mut ignore_existing = false;
    for arg in env::args().skip(1) {
        if arg == "--ignore-existing" {
            ignore_existing = true;
        } else {
            eprintln!("Unknown argument: {arg}");
            std::process::exit(2);
        }
    }

    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() {
        eprintln!("Failed to read stdin");
        std::process::exit(1);
    }
    let source = buffer.trim();
    if source.is_empty() {
        eprintln!("Provide cron text via stdin");
        std::process::exit(1);
    }

    match cron_lexer().parse(source) {
        Ok(tokens) => {
            let mut paths = Vec::new();

            for token in tokens {
                if let Token::Path(ref p) = token {
                    paths.push(p.clone());
                }

                let tt_label = token_label(&token);
                let t = format!("{token:?}");
                eprintln!(
                    "{:<7} {:>10}:{:<5} {:>32} {}",
                    tt_label,
                    file!().bright_cyan(),
                    line!().to_string().green(),
                    t.yellow(),
                    "main()".yellow()
                );
            }

            for path in paths {
                let skip_exists_check = path.contains('`') 
                    || path.contains(':')
                    || path.contains('*');
                if skip_exists_check {
                    continue;
                }
                if Path::new(&path).exists() {
                    if ignore_existing {
                        continue;
                    }
                }

                println!("{path}");
            }
        }
        Err(errors) => {
            for error in errors {
                eprintln!("Lexer error: {error}");
            }
            std::process::exit(1);
        }
    }
}
