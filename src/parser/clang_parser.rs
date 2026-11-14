use std::fmt;

/// A lightweight source preflight used by Crown's deliberately small converter.
///
/// This is not a C compiler or a complete C parser: it checks delimiter and
/// quoted-string balance, then returns the original source for source-level
/// rewriting. Keeping the source (rather than a compiler's textual AST dump)
/// is important because the rewriter operates on C syntax, not Clang's debug
/// representation.
pub fn parse_c_code(code: &str) -> Result<String, ParseError> {
    validate_delimiters(code)?;
    Ok(code.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: &'static str,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} on line {}", self.message, self.line)
    }
}

impl std::error::Error for ParseError {}

fn validate_delimiters(code: &str) -> Result<(), ParseError> {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Code,
        String,
        Character,
        LineComment,
        BlockComment,
    }

    let bytes = code.as_bytes();
    let mut stack: Vec<(u8, usize)> = Vec::new();
    let mut state = State::Code;
    let mut escaped = false;
    let mut line = 1;
    let mut index = 0;

    while index < bytes.len() {
        let current = bytes[index];
        let next = bytes.get(index + 1).copied();
        if current == b'\n' {
            line += 1;
            if state == State::LineComment {
                state = State::Code;
            }
        }

        match state {
            State::Code => match (current, next) {
                (b'/', Some(b'/')) => {
                    state = State::LineComment;
                    index += 1;
                }
                (b'/', Some(b'*')) => {
                    state = State::BlockComment;
                    index += 1;
                }
                (b'"', _) => {
                    state = State::String;
                    escaped = false;
                }
                (b'\'', _) => {
                    state = State::Character;
                    escaped = false;
                }
                (b'{' | b'(' | b'[', _) => stack.push((current, line)),
                (b'}' | b')' | b']', _) => {
                    let expected = match current {
                        b'}' => b'{',
                        b')' => b'(',
                        _ => b'[',
                    };
                    match stack.pop() {
                        Some((opening, _)) if opening == expected => {}
                        _ => {
                            return Err(ParseError {
                                line,
                                message: "unmatched closing delimiter",
                            })
                        }
                    }
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
            State::LineComment => {}
            State::BlockComment if current == b'*' && next == Some(b'/') => {
                state = State::Code;
                index += 1;
            }
            State::BlockComment => {}
        }
        index += 1;
    }

    if matches!(state, State::String | State::Character) {
        return Err(ParseError {
            line,
            message: "unterminated string or character literal",
        });
    }
    if state == State::BlockComment {
        return Err(ParseError {
            line,
            message: "unterminated block comment",
        });
    }
    if let Some((_, opening_line)) = stack.pop() {
        return Err(ParseError {
            line: opening_line,
            message: "unclosed delimiter",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_source_instead_of_a_debug_ast() {
        let code = "int main(void) { return 0; }";
        assert_eq!(parse_c_code(code).unwrap(), code);
    }

    #[test]
    fn ignores_delimiters_in_strings_and_comments() {
        let code = "int main() { printf(\"{[(]\"); /* } */ return 0; }";
        assert!(parse_c_code(code).is_ok());
    }

    #[test]
    fn reports_unclosed_delimiters() {
        let error = parse_c_code("int main() {\n return 0;").unwrap_err();
        assert_eq!(error.message, "unclosed delimiter");
        assert_eq!(error.line, 1);
    }
}
