# Crown

Crown is an **experimental, conservative C-to-Rust source converter**. It demonstrates a small set of straightforward rewrites; it is not a general C/C++ transpiler, compiler, or security tool. Converted code must be reviewed, completed, and tested by a Rust developer.

## Current scope

Crown currently recognizes a deliberately narrow subset:

- `int main()` / `int main(void)` and simple `int` function signatures/parameters;
- simple `int` local declarations;
- basic ascending integer `for` loops (`i++` or `i += 1`);
- simple `printf` calls with a string literal, optionally using `%d`, `%i`, or `%s` and one argument;
- `return 0;` in `main` and preprocessor directive lines (such as `#include`).

Unsupported lines are generally preserved instead of being guessed at. In particular, Crown does **not** translate pointer ownership, `malloc`/`free`, structs, macros, arbitrary expressions, most library calls, or C++. Removing a header directive does not port the declarations it provided. Output can therefore be invalid Rust and is never guaranteed safe or behaviorally equivalent.

The input preflight checks balanced delimiters and terminated strings/comments. It is not a full C parser and does not type-check or compile the input.

## Requirements

- Rust and Cargo (stable; edition 2021).
- No external Clang/LLVM installation is needed.

## Build and test

```sh
cargo test
cargo build --release
```

## Usage

Write to standard output:

```sh
cargo run -- -i input.c
```

Write to a file:

```sh
cargo run -- -i input.c -o output.rs
```

Use `-o -` to explicitly select standard output. Errors are reported on standard error with a non-zero exit status. Crown also prints a reminder to review every conversion.

For example, this input:

```c
#include <stdio.h>
int main() {
    printf("hello");
    return 0;
}
```

converts to:

```rust
fn main() {
    println!("hello");
}
```

Always run `rustfmt`, compile the generated Rust, compare behavior with the original program, and manually audit memory safety and error handling before using it.

## Contributing

Contributions are welcome. Please add tests for supported transformations and clearly document unsupported or behavior-changing cases. A converter rule must not imply that raw pointers or manual allocation have become safe automatically.

## License

GPL-3.0. See [LICENSE](LICENSE).
