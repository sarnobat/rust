use clap::{App, Arg};

fn main() {
    let matches = App::new("My CLI App")
        .version("1.0")
        .author("Your Name")
        .about("A simple CLI application")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Sets the output file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Enables verbose mode"),
        )
        .get_matches();

    // Accessing the parsed arguments
    let input_file = matches.value_of("input").unwrap(); // unwrap() is safe here because it's required
    println!("Input file: {}", input_file);

    if let Some(output_file) = matches.value_of("output") {
        println!("Output file: {}", output_file);
    }

    if matches.is_present("verbose") {
        println!("Verbose mode enabled");
    }
}