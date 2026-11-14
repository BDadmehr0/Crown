use crown::rewriting::rewrite_c_to_rust;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn translates_basic_main_and_output() {
    let source = "#include <stdio.h>\nint main() {\n    printf(\"hello\");\n    return 0;\n}\n";
    assert_eq!(
        rewrite_c_to_rust(source),
        "fn main() {\n    print!(\"hello\");\n}\n"
    );
}

#[test]
fn translates_integer_declarations_and_loops() {
    let source = "int main() {\n    int sum = 0;\n    for (int i = 0; i < n; i++) {\n    }\n    return 0;\n}";
    assert_eq!(
        rewrite_c_to_rust(source),
        "fn main() {\n    let mut sum: i32 = 0;\n    for i in 0..n {\n    }\n}"
    );
}

#[test]
fn translates_the_reported_input_program() {
    let source = include_str!("input.c");
    let rust = rewrite_c_to_rust(source);
    assert_eq!(rust, include_str!("output.rs"));
    assert!(!rust.contains("scanf"));
}

#[test]
fn generated_two_integer_program_compiles_with_rustc() {
    let rust = rewrite_c_to_rust(include_str!("input.c"));
    let directory = std::env::temp_dir().join(format!("crown-test-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let source_path = directory.join("generated.rs");
    let binary_path = directory.join("generated-program");
    fs::write(&source_path, rust).unwrap();

    let result = Command::new("rustc")
        .arg("--edition=2021")
        .arg(&source_path)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("Rust toolchain should provide rustc in CI");
    assert!(
        result.status.success(),
        "generated Rust did not compile:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );

    let mut child = Command::new(&binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"6 7\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"Enter two integers: 6 + 7 = 13");
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn leaves_unsupported_pointer_code_visible() {
    let source = "int* ptr;\nptr = malloc(sizeof(int));";
    assert_eq!(rewrite_c_to_rust(source), source);
}
