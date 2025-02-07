// tests/rewriting_test.rs

use crown::rewriting::rewrite_c_to_rust;

#[test]
fn test_rewrite_pointer() {
    let c_code = "int* ptr;";
    let rust_code = rewrite_c_to_rust(c_code);
    assert_eq!(rust_code, "&i32 ptr;");
}

#[test]
fn test_rewrite_malloc() {
    let c_code = "malloc(sizeof(int));";
    let rust_code = rewrite_c_to_rust(c_code);
    assert_eq!(rust_code, "Box::new(sizeof(i32));");
}

#[test]
fn test_rewrite_for_loop() {
    let c_code = "for (int i = 0; i < n; i++) { ... }";
    let rust_code = rewrite_c_to_rust(c_code);
    assert_eq!(rust_code, "for i in 0..n { ... }");
}
