pub fn rewrite_c_to_rust(ast: &str) -> String {
    let mut result = ast.to_string();

    result = result.replace("int* ", "&i32 ");
    result = result.replace("malloc(", "Box::new(");
    result = result.replace("free(", "");
    result = result.replace("sizeof(int)", "sizeof(i32)");
    result = result.replace("for (int i = 0; i < n; i++)", "for i in 0..n");

    result
}
