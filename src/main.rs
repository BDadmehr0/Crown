use clap::{Arg, Command};
use crown::parser::clang_parser::parse_c_code;
use crown::rewriting::rewrite_c_to_rust;
use std::error::Error;
use std::io::{self, Write};

fn main() {
    if let Err(error) = run() {
        eprintln!("crown: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("crown")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Experimental, conservative C-to-Rust source converter")
        .long_about(
            "Converts a documented subset of C to Rust. This is not a general C/C++\n             translator; review the generated Rust and manually port unsupported code.",
        )
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("C source file to convert")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Rust output file (defaults to standard output; use - for stdout)"),
        )
        .get_matches();

    let input_path = matches.get_one::<String>("input").expect("required by clap");
    let source = std::fs::read_to_string(input_path)?;
    let source = parse_c_code(&source)?;
    let converted = rewrite_c_to_rust(&source);

    if let Some(output_path) = matches.get_one::<String>("output") {
        if output_path == "-" {
            io::stdout().write_all(converted.as_bytes())?;
        } else {
            std::fs::write(output_path, converted.as_bytes())?;
            eprintln!("Crown wrote converted source to {output_path}");
        }
    } else {
        io::stdout().write_all(converted.as_bytes())?;
    }

    if source.contains("malloc(")
        || source.contains("free(")
        || source.contains("->")
    {
        eprintln!("warning: pointer or allocation code may not have been translated; inspect the output manually");
    }
    eprintln!("warning: Crown supports only a small C subset; generated Rust requires review and may need manual fixes");
    Ok(())
}
