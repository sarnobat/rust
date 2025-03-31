use clap::{App, Arg};
use std::process;

fn main() {
    let matches = App::new("My App")
        .version("1.0")
        .author("Your Name")
        .about("Greets you by name")
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("NAME")
                .help("Your name")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("help")
                .long("help")
                .help("Prints help information")
        )
        .get_matches();

    if matches.is_present("help") {
        App::new("My App").print_help().unwrap();
        process::exit(0);
    }

    if let Some(name) = matches.value_of("name") {
        println!("Hello {}", name);
    } else{
        println!("Hello ");
    }
}
