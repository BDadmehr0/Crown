//! Conservative, source-level translations for a small and documented C subset.
//!
//! Crown intentionally does not claim to translate arbitrary C or C++ into
//! safe Rust. Unsupported constructs are left visible for manual conversion.

/// Convert a limited set of common C constructs to readable Rust source.
///
/// Supported today: simple `main`/integer function signatures, integer local
/// declarations, literal/limited-format `printf` calls, basic integer `for`
/// loops, and `return 0` from `main`. Other code is retained as-is so that
/// unsupported C is not silently misrepresented as safe Rust.
pub fn rewrite_c_to_rust(source: &str) -> String {
    let mut output = Vec::new();
    let mut brace_depth = 0usize;
    let mut main_depth = None;
    let mut in_block_comment = false;

    for original_line in source.lines() {
        let trimmed = original_line.trim();
        if trimmed.starts_with('#') {
            // Header directives have no direct Rust equivalent. Users must
            // replace declarations/macros from non-standard headers manually.
            continue;
        }
        if trimmed.is_empty() {
            if output.last().is_some_and(|line: &String| !line.is_empty()) {
                output.push(String::new());
            }
            continue;
        }

        let indent = &original_line[..original_line.len() - original_line.trim_start().len()];
        let mut line = trimmed.to_owned();

        if let Some(signature) = convert_function_signature(&line) {
            if signature.starts_with("fn main(") {
                main_depth = Some(brace_depth + 1);
            }
            line = signature;
        }

        if let Some(loop_header) = convert_for_loop(&line) {
            line = loop_header;
        }

        if main_depth.is_some() && line == "return 0;" {
            // Rust's main returns unit; an explicit `return;` is valid but
            // omitting it is more idiomatic and avoids needless output.
            continue;
        }

        if brace_depth > 0 {
            if let Some(declaration) = convert_integer_declaration(&line) {
                line = declaration;
            }
        }
        if let Some(print) = convert_printf(&line) {
            line = print;
        }

        // Track function nesting so a `return 0` inside a nested block in
        // main is still recognized. This is intentionally only a lightweight
        // brace count, not a full lexical parse.
        output.push(format!("{indent}{line}"));
        update_brace_depth(
            &line,
            &mut brace_depth,
            &mut main_depth,
            &mut in_block_comment,
        );
    }

    let mut result = output.join("\n");
    if !result.is_empty() && source.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn update_brace_depth(
    line: &str,
    depth: &mut usize,
    main_depth: &mut Option<usize>,
    in_block_comment: &mut bool,
) {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Code,
        String,
        Character,
        LineComment,
        BlockComment,
    }

    let bytes = line.as_bytes();
    let mut state = if *in_block_comment {
        State::BlockComment
    } else {
        State::Code
    };
    let mut escaped = false;
    let mut index = 0;
    while index < bytes.len() {
        let current = bytes[index];
        let next = bytes.get(index + 1).copied();
        match state {
            State::Code => match (current, next) {
                (b'/', Some(b'/')) => state = State::LineComment,
                (b'/', Some(b'*')) => {
                    state = State::BlockComment;
                    *in_block_comment = true;
                    index += 1;
                }
                (b'"', _) => state = State::String,
                (b'\'', _) => state = State::Character,
                (b'{', _) => *depth += 1,
                (b'}', _) => {
                    if *main_depth == Some(*depth) {
                        *main_depth = None;
                    }
                    *depth = (*depth).saturating_sub(1);
                }
                _ => {}
            },
            State::String | State::Character => {
                if escaped {
                    escaped = false;
                } else if current == b'\\' {
                    escaped = true;
                } else if (state == State::String && current == b'"')
                    || (state == State::Character && current == b'\'')
                {
                    state = State::Code;
                }
            }
            State::LineComment => break,
            State::BlockComment if current == b'*' && next == Some(b'/') => {
                state = State::Code;
                *in_block_comment = false;
                index += 1;
            }
            State::BlockComment => {}
        }
        index += 1;
    }
}

fn convert_function_signature(line: &str) -> Option<String> {
    let signature = line.strip_suffix('{')?.trim_end();
    if signature == "int main()" || signature == "int main(void)" {
        return Some("fn main() {".to_owned());
    }

    let rest = signature.strip_prefix("int ")?;
    let open = rest.find('(')?;
    let name = rest[..open].trim();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    let params = rest[open + 1..].strip_suffix(')')?;
    let params = if params.trim().is_empty() || params.trim() == "void" {
        String::new()
    } else {
        params
            .split(',')
            .map(|parameter| {
                let words: Vec<_> = parameter.split_whitespace().collect();
                if words.len() == 2
                    && words[0] == "int"
                    && words[1]
                        .chars()
                        .next()
                        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
                    && words[1]
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '_')
                {
                    Some(format!("{}: i32", words[1]))
                } else {
                    None
                }
            })
            .collect::<Option<Vec<_>>>()?
            .join(", ")
    };
    Some(format!("fn {name}({params}) -> i32 {{"))
}

fn convert_integer_declaration(line: &str) -> Option<String> {
    let statement = line.strip_suffix(';')?;
    let declaration = statement.strip_prefix("int ")?;
    let (name, initializer) = match declaration.split_once('=') {
        Some((name, value)) => (name.trim(), Some(value.trim())),
        None => (declaration.trim(), None),
    };
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    Some(match initializer {
        Some(value) => format!("let mut {name}: i32 = {value};"),
        None => format!("let mut {name}: i32;"),
    })
}

fn convert_for_loop(line: &str) -> Option<String> {
    let inner = line
        .strip_prefix("for (")?
        .strip_suffix('{')?
        .trim_end()
        .strip_suffix(')')?
        .trim_end();
    let mut clauses = inner.split(';');
    let initializer = clauses.next()?.trim();
    let condition = clauses.next()?.trim();
    let increment = clauses.next()?.trim();
    if clauses.next().is_some() || increment.is_empty() {
        return None;
    }

    let initializer = initializer.strip_prefix("int ").unwrap_or(initializer);
    let (variable, start) = initializer.split_once('=')?;
    let variable = variable.trim();
    let start = start.trim();
    let (condition_variable, end) = condition.split_once('<')?;
    if condition_variable.trim() != variable || !increment.starts_with(variable) {
        return None;
    }
    let suffix = increment.strip_prefix(variable)?.trim();
    if suffix != "++" && suffix != "+= 1" {
        return None;
    }
    Some(format!("for {variable} in {start}..{} {{", end.trim()))
}

fn convert_printf(line: &str) -> Option<String> {
    let call = line.strip_prefix("printf(")?.strip_suffix(';')?.strip_suffix(')')?;
    // Only translate the common printf("format", one_argument) form. More
    // complex C varargs remain untouched for human review.
    let (format, argument) = match split_printf_arguments(call)? {
        Some((format, argument)) => (format, Some(argument)),
        None => (call.trim(), None),
    };
    let literal = format.strip_prefix('"')?.strip_suffix('"')?;
    let (rust_format, placeholders) = convert_printf_format(literal)?;
    let macro_name = if literal.ends_with("\\n") {
        "print"
    } else {
        "println"
    };
    match argument {
        Some(argument) if placeholders == 1 => {
            Some(format!("{macro_name}!(\"{rust_format}\", {argument});"))
        }
        None if placeholders == 0 => Some(format!("{macro_name}!(\"{rust_format}\");")),
        _ => None,
    }
}

fn convert_printf_format(format: &str) -> Option<(String, usize)> {
    let mut output = String::new();
    let mut characters = format.chars();
    let mut placeholders = 0;
    while let Some(character) = characters.next() {
        if character != '%' {
            match character {
                '{' => output.push_str("{{"),
                '}' => output.push_str("}}"),
                _ => output.push(character),
            }
            continue;
        }
        match characters.next()? {
            'd' | 'i' | 's' => {
                output.push_str("{}");
                placeholders += 1;
            }
            '%' => output.push('%'),
            _ => return None,
        }
    }
    Some((output, placeholders))
}

/// Split a printf call's arguments at a comma outside its quoted format string.
/// `None` means malformed/unsupported; `Some(None)` is a single-argument call.
fn split_printf_arguments(call: &str) -> Option<Option<(&str, &str)>> {
    let mut quoted = false;
    let mut escaped = false;
    for (index, character) in call.char_indices() {
        if escaped {
            escaped = false;
        } else if character == '\\' && quoted {
            escaped = true;
        } else if character == '"' {
            quoted = !quoted;
        } else if character == ',' && !quoted {
            return Some(Some((call[..index].trim(), call[index + 1..].trim())));
        }
    }
    if quoted {
        return None;
    }
    Some(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_a_small_complete_program() {
        let c = "#include <stdio.h>\nint main() {\n    printf(\"hello\");\n    return 0;\n}\n";
        assert_eq!(
            rewrite_c_to_rust(c),
            "fn main() {\n    println!(\"hello\");\n}\n"
        );
    }

    #[test]
    fn translates_common_declarations_and_loop() {
        let c = "int main() {\n    int total = 0;\n    for (int i = 0; i < 4; i++) {\n    }\n    return 0;\n}";
        assert_eq!(
            rewrite_c_to_rust(c),
            "fn main() {\n    let mut total: i32 = 0;\n    for i in 0..4 {\n    }\n}"
        );
    }

    #[test]
    fn translates_integer_function_parameters() {
        assert_eq!(
            rewrite_c_to_rust("int add(int left, int right) {\n    return left + right;\n}"),
            "fn add(left: i32, right: i32) -> i32 {\n    return left + right;\n}"
        );
    }

    #[test]
    fn translates_a_simple_printf_format() {
        assert_eq!(
            rewrite_c_to_rust("printf(\"value=%d\", value);"),
            "println!(\"value={}\", value);"
        );
    }

    #[test]
    fn preserves_a_newline_without_adding_an_extra_one() {
        assert_eq!(
            rewrite_c_to_rust(r#"printf("hello\n");"#),
            r#"print!("hello\n");"#
        );
    }

    #[test]
    fn leaves_unsupported_pointer_operations_visible() {
        let source = "int* ptr;\nptr = malloc(sizeof(int));";
        assert_eq!(rewrite_c_to_rust(source), source);
    }
}
