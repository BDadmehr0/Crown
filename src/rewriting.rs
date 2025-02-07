// src/rewriting.rs

pub fn rewrite_c_to_rust(ast: &str) -> String {
    let mut result = ast.to_string();

    // تبدیل اشاره‌گرها به مرجع در Rust
    result = result.replace("int* ", "&i32 ");  // نمونه ساده برای اشاره‌گرها

    // تبدیل malloc به Box
    result = result.replace("malloc(", "Box::new(");
    // حذف free
    result = result.replace("free(", "");

    // تبدیل sizeof(int) به sizeof(i32) به شکل مورد نظر شما
    result = result.replace("sizeof(int)", "sizeof(i32)");

    // تبدیل حلقه‌های for به Rust
    result = result.replace("for (int i = 0; i < n; i++)", "for i in 0..n");

    result
}
