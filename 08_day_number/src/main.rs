use chrono::{Datelike, Local};

fn main() {

    // Current date
    let today = Local::now().date_naive();

    // Day of year (1..=366)
    let day_of_year: u32 = today.ordinal();

    println!("{}", day_of_year);
}
