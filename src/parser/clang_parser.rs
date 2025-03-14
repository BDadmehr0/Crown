use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Write;

pub fn parse_c_code(code: &str) -> Result<String, String> {
    let path = Path::new("/usr/bin/clang");

    let mut child = Command::new(path)
        .arg("-cc1")
        .arg("-x")
        .arg("c")
        .arg("-ast-dump")
        .arg("-")
        .arg("-I/usr/include")  // اضافه کردن مسیر هدرها
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(code.as_bytes()).map_err(|e| e.to_string())?;
    }

    let output = child.wait_with_output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        let error_output = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Clang failed with status: {}\n{}", output.status, error_output));
    }

    let ast = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;

    println!("AST Output: {}", ast);

    Ok(ast)
}


#[cfg(test)]
mod tests {
    use super::*;

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
        println!("AST Output: {}", ast);
        assert!(ast.contains("a"));

    }
}

