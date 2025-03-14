use clap::{Arg, Command};
use crown::rewriting::rewrite_c_to_rust;
use crown::parser::clang_parser::parse_c_code;

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
                .num_args(1),  // تغییر این بخش
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("The file to save the converted Rust code")
                .num_args(1),  // تغییر این بخش
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").unwrap_or_else(|| {
        eprintln!("Error: Missing input file. Use -i <file> or --input <file>");
        std::process::exit(1);
    });

    let output_file = matches.get_one::<String>("output").expect("Missing output file");
    
    let input_code = std::fs::read_to_string(input_file)
        .expect("Failed to read input file");

    let ast = parse_c_code(&input_code).expect("Failed to parse C code");

    let converted_code = rewrite_c_to_rust(&ast);


    std::fs::write(output_file, converted_code)
        .expect("Failed to write output file");

    println!("Code has been converted and saved to {}", output_file);
}

