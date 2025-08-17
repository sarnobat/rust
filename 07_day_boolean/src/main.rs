use chrono::{Datelike, Local};
use std::{env, process};

fn main() {
    // Current day-of-year (1..=366)
    let today = Local::now().date_naive();
    let day_of_year: u32 = today.ordinal();

    // Get divisor arg
    let mut args = env::args().skip(1);
    let divisor_str = match args.next() {
        Some(s) => s,
        None => {
            eprintln!("[warning] to see what day of the year it is, run dayofyear.py");
            eprintln!("[error] specify a divisor");
            process::exit(1);
        }
    };

    // Parse divisor (require non-zero positive int to avoid undefined cases)
    let divisor: u32 = match divisor_str.parse() {
        Ok(0) | Err(_) => {
            eprintln!("[error] divisor must be a non-zero integer");
            process::exit(1);
        }
        Ok(d) => d,
    };

    let modulus = day_of_year % divisor;

    if modulus == 0 {
        eprintln!("[debug] >>\t{} {} {}", divisor, day_of_year, modulus);
        println!("0");
        process::exit(0);
    } else {
        println!("1");
        eprintln!("[debug] \t{} {} {}", divisor, day_of_year, modulus);
        process::exit(1);
    }
}
