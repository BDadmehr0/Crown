use crown::rewriting::rewrite_c_to_rust;

#[test]
fn translates_basic_main_and_output() {
    let source = "#include <stdio.h>\nint main() {\n    printf(\"hello\");\n    return 0;\n}\n";
    assert_eq!(
        rewrite_c_to_rust(source),
        "fn main() {\n    println!(\"hello\");\n}\n"
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
fn keeps_unsupported_pointer_code_visible() {
    let source = "int* ptr;\nptr = malloc(sizeof(int));";
    assert_eq!(rewrite_c_to_rust(source), source);
}
