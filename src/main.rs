use clap::{Arg, Command};
use crown::rewriting::rewrite_c_to_rust;

fn main() {
    let matches = Command::new("Crown")
        .version("1.0")
        .about("C/C++ to Rust Code Converter")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("The C/C++ file to convert")
                .takes_value(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("The file to save the converted Rust code")
                .takes_value(true),
        )
        .get_matches();

    let input_file = matches.value_of("input").unwrap();
    let output_file = matches.value_of("output").unwrap();

    let input_code = std::fs::read_to_string(input_file)
        .expect("Failed to read input file");

    let converted_code = rewrite_c_to_rust(&input_code);

    std::fs::write(output_file, converted_code)
        .expect("Failed to write output file");

    println!("Code has been converted and saved to {}", output_file);
}