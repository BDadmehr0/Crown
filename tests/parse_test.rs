use crown::parser::clang_parser::parse_c_code;

#[test]
fn test_parse_function() {
    let code = r#"
    int main() {
        return 0;
    }
    "#;

    let ast = parse_c_code(code).unwrap();
    assert!(ast.contains("main"));
}

#[test]
fn test_parse_variable() {
    let code = r#"
    int a = 10;
    "#;

    let ast = parse_c_code(code).unwrap();
    assert!(ast.contains("a"));
}

