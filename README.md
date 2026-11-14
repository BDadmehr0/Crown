# Crown

Crown is an experimental C-to-Rust converter for common, small console programs. It performs source-level rewrites; it is not a complete C/C++ compiler or a security tool. Generated code must be reviewed, compiled, behavior-tested, and manually audited.

## Supported subset

The converter currently handles these common patterns when written in a straightforward, one-statement-per-line style:

- `main` and simple functions with scalar parameters/return types;
- scalar local declarations, including comma-separated variables, and fixed-size one- or multi-dimensional arrays with brace initializers;
- simple `if`, `else if`, `while`, `do`/`while`, and monotonic `for` loops using `<`, `<=`, `>`, or `>=` with `++`, `--`, or a literal step;
- basic `switch` statements with single `case` labels and `break`, plus simple ternary expressions;
- standalone `++` / `--` statements;
- `printf` with common `%d`, `%i`, `%u`, `%f`, `%lf`, `%c`, `%s`, `%x`, `%X`, `%o` and `%%` formats, including multiple arguments;
- token-based `%d`, `%i`, `%u`, `%f`, `%lf`, `%c`, and `%s` `scanf` input in `main`;
- `return 0;` from `main` and simple `#include` removal.

For example, this program:

```c
#include <stdio.h>
int main() {
    int number1, number2, sum;
    printf("Enter two integers: ");
    scanf("%d %d", &number1, &number2);
    sum = number1 + number2;
    printf("%d + %d = %d", number1, number2, sum);
    return 0;
}
```

converts to Rust with mutable `i32` locals, line-based stdin token parsing, and Rust formatting macros. The generated version is saved as [tests/output.rs](tests/output.rs), with the C fixture in [tests/input.c](tests/input.c).

## Important limitations

C and Rust have different type systems, memory models, and runtime behavior. There is no universal syntax-to-syntax conversion that can make arbitrary C valid and safe Rust. Crown currently does not translate pointer ownership, structs, unions, enums, function pointers, macros, dynamic allocation, file I/O, complex declarators, or most library APIs. C format strings and input behavior are only approximated for the listed conversions. Unsupported lines may remain C and make the output fail to compile; even compilable output may change behavior. The preflight parser checks only delimiter/string/comment balance, not C syntax or types.

Crown emits a general review warning. Do not treat generated output as safe or production-ready without review.

## Requirements and usage

Requires stable Rust and Cargo; no Clang/LLVM installation is needed.

```sh
cargo test
cargo build --release
cargo run -- -i input.c                 # write Rust to stdout
cargo run -- -i input.c -o output.rs    # write Rust to a file
```

Use `-o -` to explicitly select stdout. Errors are reported on stderr with a non-zero exit status. Run `rustfmt`, compile the generated Rust, compare behavior against the C program, and manually audit memory safety before using it.

## Contributing

Contributions are welcome. Add regression tests for each new conversion and document behavior changes and unsupported cases. Never imply that translating a pointer or allocation expression automatically makes it safe.

## License

GPL-3.0. See [LICENSE](LICENSE).
