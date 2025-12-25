use chumsky::prelude::*;
use std::env;

#[derive(Debug, Clone)]
enum Expr {
    Number(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn add(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Add(Box::new(lhs), Box::new(rhs))
    }

    fn mul(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Mul(Box::new(lhs), Box::new(rhs))
    }
}

fn fold_expr(parts: Vec<Expr>, combine: fn(Expr, Expr) -> Expr) -> Expr {
    parts
        .into_iter()
        .reduce(combine)
        .expect("parser guarantees at least one expression")
}

fn expr_parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    recursive(|expr| {
        let atom = text::int(10)
            .map(|s: String| Expr::Number(s.parse::<i64>().expect("valid integer")))
            .or(expr.clone().delimited_by(just('('), just(')')))
            .padded();

        let product = atom
            .clone()
            .separated_by(just('*').padded())
            .at_least(1)
            .map(|parts| fold_expr(parts, Expr::mul));

        product
            .clone()
            .separated_by(just('+').padded())
            .at_least(1)
            .map(|parts| fold_expr(parts, Expr::add))
    })
    .then_ignore(end())
}

fn eval(expr: &Expr) -> i64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Add(lhs, rhs) => eval(lhs) + eval(rhs),
        Expr::Mul(lhs, rhs) => eval(lhs) * eval(rhs),
    }
}

fn main() {
    let input = env::args().skip(1).collect::<Vec<_>>().join(" ");
    let source = if input.trim().is_empty() {
        "2 + 3 * (4 + 5)".to_string()
    } else {
        input
    };

    match expr_parser().parse(source.as_str()) {
        Ok(ast) => {
            println!("Parsed AST: {ast:?}");
            println!("Evaluation result: {}", eval(&ast));
        }
        Err(errors) => {
            for error in errors {
                eprintln!("Parse error: {error}");
            }
            std::process::exit(1);
        }
    }
}
