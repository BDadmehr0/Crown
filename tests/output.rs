fn main() {
    let mut __crown_tokens = std::collections::VecDeque::<String>::new();

    let mut number1: i32 = 0;
    let mut number2: i32 = 0;
    let mut sum: i32 = 0;

    print!("Enter two integers: ");
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let __crown_value = loop {
        if let Some(token) = __crown_tokens.pop_front() { break token; }
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).expect("failed to read stdin");
        __crown_tokens.extend(line.split_whitespace().map(str::to_owned));
    };
    number1 = __crown_value.parse::<i32>().expect("invalid input value");
    let __crown_value = loop {
        if let Some(token) = __crown_tokens.pop_front() { break token; }
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).expect("failed to read stdin");
        __crown_tokens.extend(line.split_whitespace().map(str::to_owned));
    };
    number2 = __crown_value.parse::<i32>().expect("invalid input value");

    // calculate the sum
    sum = number1 + number2;

    print!("{} + {} = {}", number1, number2, sum);
}
