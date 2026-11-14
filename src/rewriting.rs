//! Conservative source-level conversions for common C console programs.
//! This is intentionally not a full C or C++ compiler.

/// Convert a useful, documented subset of C to Rust source. Constructs that
/// cannot be translated confidently are generally left visible for review.
pub fn rewrite_c_to_rust(source: &str) -> String {
    let mut output = Vec::new();
    let mut depth = 0usize;
    let mut main_depth = None;
    let mut block_comment = false;
    let mut input_enabled = source.contains("scanf(");
    let mut switch_stack: Vec<(usize, bool, bool)> = Vec::new();

    for original in source.lines() {
        let trimmed = original.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.is_empty() {
            if output.last().is_some_and(|line: &String| !line.is_empty()) {
                output.push(String::new());
            }
            continue;
        }

        let indent = &original[..original.len() - original.trim_start().len()];
        let (code, comment) = split_line_comment(trimmed);
        let mut line = code.trim().to_owned();
        let comment = comment.unwrap_or("");
        if line.is_empty() {
            output.push(format!("{indent}{comment}"));
            continue;
        }

        if let Some(function) = function_signature(&line) {
            let main = function.starts_with("fn main(");
            if main {
                main_depth = Some(depth + 1);
            }
            output.push(format!("{indent}{function}"));
            adjust_depth(&function, &mut depth, &mut main_depth, &mut block_comment);
            if main && input_enabled {
                output.push(format!("{indent}    let mut __crown_tokens = std::collections::VecDeque::<String>::new();"));
                input_enabled = false;
            }
            continue;
        }

        if let Some(expression) = switch_header(&line) {
            let rust = format!("match {expression} {{");
            output.push(format!("{indent}{rust}"));
            adjust_depth(&rust, &mut depth, &mut main_depth, &mut block_comment);
            switch_stack.push((depth, false, false));
            continue;
        }

        if let Some(pattern) = case_label(&line) {
            if let Some((switch_depth, open, has_default)) = switch_stack.last_mut() {
                if pattern == "_" { *has_default = true; }
                if *open && depth == *switch_depth + 1 {
                    output.push(format!("{indent}}}"));
                    adjust_depth("}", &mut depth, &mut main_depth, &mut block_comment);
                    *open = false;
                }
                if depth == *switch_depth {
                    let arm = format!("{pattern} => {{");
                    output.push(format!("{indent}{arm}"));
                    adjust_depth(&arm, &mut depth, &mut main_depth, &mut block_comment);
                    *open = true;
                    continue;
                }
            }
        }

        if line == "}" {
            if let Some((switch_depth, open, has_default)) = switch_stack.last().copied() {
                if open && depth == switch_depth + 1 {
                    output.push(format!("{indent}}}"));
                    adjust_depth("}", &mut depth, &mut main_depth, &mut block_comment);
                    if let Some(state) = switch_stack.last_mut() {
                        state.1 = false;
                    }
                }
                if depth == switch_depth {
                    if !has_default {
                        output.push(format!("{indent}_ => {{}}"));
                    }
                    switch_stack.pop();
                }
            }
        }
        if line == "break;"
            && switch_stack
                .last()
                .is_some_and(|(switch_depth, open, _)| *open && depth == *switch_depth + 1)
        {
            continue;
        }

        if line == "do {" {
            line = "loop {".to_owned();
        }
        if let Some(condition) = do_while_close(&line) {
            let check = format!("if !({condition}) {{ break; }}");
            output.push(format!("{indent}{check}"));
            adjust_depth(&check, &mut depth, &mut main_depth, &mut block_comment);
            output.push(format!("{indent}}}{comment}"));
            adjust_depth("}", &mut depth, &mut main_depth, &mut block_comment);
            continue;
        }

        if let Some(converted) = for_loop(&line) {
            line = converted;
        } else if let Some(converted) = else_if(&line) {
            line = converted;
        } else if let Some(converted) = condition(&line, "if") {
            line = converted;
        } else if let Some(converted) = condition(&line, "while") {
            line = converted;
        }

        if main_depth.is_some() && line == "return 0;" {
            continue;
        }

        if depth > 0 {
            if let Some(declarations) = declaration(&line) {
                for (index, item) in declarations.iter().enumerate() {
                    let suffix = if index + 1 == declarations.len() { comment } else { "" };
                    output.push(format!("{indent}{item}{suffix}"));
                    adjust_depth(item, &mut depth, &mut main_depth, &mut block_comment);
                }
                continue;
            }
        }

        if line.starts_with("scanf(") {
            if let Some(reads) = scanf(&line) {
                output.push(format!("{indent}let _ = std::io::Write::flush(&mut std::io::stdout());"));
                for (read_index, read) in reads.iter().enumerate() {
                    for (statement_index, statement) in read.iter().enumerate() {
                        let suffix = if read_index + 1 == reads.len() && statement_index + 1 == read.len() { comment } else { "" };
                        output.push(format!("{indent}{statement}{suffix}"));
                    }
                }
                continue;
            }
        }

        if let Some(converted) = printf(&line) {
            line = converted;
        } else if let Some(converted) = increment(&line) {
            line = converted;
        } else {
            line = ternary(&line);
        }
        output.push(format!("{indent}{line}{comment}"));
        adjust_depth(&line, &mut depth, &mut main_depth, &mut block_comment);
    }

    let mut result = output.join("\n");
    if !result.is_empty() && source.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn split_line_comment(line: &str) -> (&str, &str) {
    let mut quoted = false;
    let mut escaped = false;
    let bytes = line.as_bytes();
    for i in 0..bytes.len().saturating_sub(1) {
        match bytes[i] {
            b'\\' if quoted => escaped = !escaped,
            b'"' if !escaped => quoted = !quoted,
            b'/' if !quoted && bytes[i + 1] == b'/' => return (&line[..i], &line[i..]),
            _ => escaped = false,
        }
    }
    (line, "")
}

fn function_signature(line: &str) -> Option<String> {
    let signature = line.strip_suffix('{')?.trim_end();
    let open = signature.find('(')?;
    let close = signature.rfind(')')?;
    if close < open { return None; }
    let head = signature[..open].trim();
    let split = head.rfind(char::is_whitespace)?;
    let return_type = c_type(head[..split].trim())?;
    let name = head[split..].trim();
    if !identifier(name) || !signature[close + 1..].trim().is_empty() {
        return None;
    }
    let raw = signature[open + 1..close].trim();
    let parameters = if raw.is_empty() || raw == "void" {
        Vec::new()
    } else {
        split_top(raw, ',')?.into_iter().map(|parameter| {
            let split = parameter.trim().rfind(char::is_whitespace)?;
            let ty = c_type(parameter[..split].trim())?;
            let name = parameter[split..].trim();
            identifier(name).then(|| format!("{name}: {ty}"))
        }).collect::<Option<Vec<_>>>()?
    };
    if name == "main" {
        if !parameters.is_empty() { return None; }
        Some("fn main() {".into())
    } else {
        let ret = if return_type == "()" { String::new() } else { format!(" -> {return_type}") };
        Some(format!("fn {name}({}){ret} {{", parameters.join(", ")))
    }
}

fn c_type(ty: &str) -> Option<&'static str> {
    match ty.trim() {
        "void" => Some("()"), "char" => Some("char"), "signed char" => Some("i8"), "unsigned char" => Some("u8"),
        "short" | "short int" | "signed short" | "signed short int" => Some("i16"),
        "unsigned short" | "unsigned short int" => Some("u16"),
        "int" | "signed" | "signed int" => Some("i32"), "unsigned" | "unsigned int" => Some("u32"),
        "long" | "long int" | "long long" | "long long int" | "signed long" | "signed long int" => Some("i64"),
        "unsigned long" | "unsigned long int" | "unsigned long long" | "unsigned long long int" => Some("u64"),
        "float" => Some("f32"), "double" | "long double" => Some("f64"), "_Bool" => Some("bool"),
        _ => None,
    }
}

fn default_value(ty: &str) -> &'static str {
    match ty { "f32" | "f64" => "0.0", "char" => "'\\0'", "bool" => "false", _ => "0" }
}

fn declaration(line: &str) -> Option<Vec<String>> {
    let statement = line.strip_suffix(';')?.trim();
    let types = ["unsigned long long int", "unsigned long long", "long long int", "long long", "unsigned short int", "unsigned short", "signed short int", "signed short", "unsigned long int", "unsigned long", "signed long int", "signed long", "unsigned int", "signed int", "unsigned char", "signed char", "short int", "long int", "double", "float", "char", "int", "short", "long", "_Bool"];
    let (ctype, rest) = types.iter().find_map(|ty| statement.strip_prefix(*ty).and_then(|s| s.strip_prefix(' ')).map(|s| (*ty, s)))?;
    let ty = c_type(ctype)?;
    split_top(rest, ',')?.into_iter().map(|item| declarator(item.trim(), ty)).collect()
}

fn declarator(item: &str, ty: &str) -> Option<String> {
    let (left, init) = split_once_top(item, '=')
        .map(|(left, right)| (left.trim(), Some(right.trim())))
        .unwrap_or((item.trim(), None));
    let mut dimensions = Vec::new();
    let name = if let Some(mut index) = left.find('[') {
        let name = &left[..index];
        while index < left.len() {
            if left.as_bytes()[index] != b'[' {
                return None;
            }
            let close = left[index + 1..].find(']')? + index + 1;
            dimensions.push(left[index + 1..close].trim().to_owned());
            index = close + 1;
        }
        name
    } else {
        left
    };
    let name = name.trim();
    if !identifier(name) {
        return None;
    }

    if dimensions.len() == 1 && ty == "char" && init.map_or(true, |value| value.starts_with('"')) {
        let value = match init {
            Some(value) if value.starts_with('"') => format!("{value}.to_owned()"),
            Some(value) => value.to_owned(),
            None => "String::new()".to_owned(),
        };
        return Some(format!("let mut {name}: String = {value};"));
    }
    if !dimensions.is_empty() {
        if dimensions[0].is_empty() {
            dimensions[0] = array_len(init?)?;
        }
        if dimensions.iter().any(String::is_empty) {
            return None;
        }
        let mut array_type = ty.to_owned();
        let mut default = default_value(ty).to_owned();
        for dimension in dimensions.iter().rev() {
            array_type = format!("[{array_type}; {dimension}]");
            default = format!("[{default}; {dimension}]");
        }
        let value = match init {
            Some(value) if value.starts_with('{') && value.ends_with('}') => c_array_initializer(value)?,
            Some(value) => value.to_owned(),
            None => default,
        };
        return Some(format!("let mut {name}: {array_type} = {value};"));
    }

    let value = init.map(ternary).unwrap_or_else(|| default_value(ty).to_owned());
    Some(format!("let mut {name}: {ty} = {value};"))
}

fn c_array_initializer(value: &str) -> Option<String> {
    let mut output = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for character in value.chars() {
        if quoted {
            output.push(character);
            if escaped { escaped = false; }
            else if character == '\\' { escaped = true; }
            else if character == '"' { quoted = false; }
        } else {
            match character {
                '"' => { quoted = true; output.push(character); }
                '{' => output.push('['),
                '}' => output.push(']'),
                _ => output.push(character),
            }
        }
    }
    if quoted { None } else { Some(output) }
}

fn array_len(init: &str) -> Option<String> {
    let values = init.strip_prefix('{')?.strip_suffix('}')?;
    Some(split_top(values, ',')?.len().to_string())
}

fn printf(line: &str) -> Option<String> {
    let call = line.strip_prefix("printf(")?.strip_suffix(';')?.trim_end().strip_suffix(')')?;
    let args = split_top(call, ',')?;
    let fmt = args.first()?.trim().strip_prefix('"')?.strip_suffix('"')?;
    let (format, count) = rust_format(fmt)?;
    if count != args.len() - 1 { return None; }
    if count == 0 { Some(format!("print!(\"{format}\");")) }
    else { Some(format!("print!(\"{format}\", {});", args[1..].join(", "))) }
}

fn rust_format(fmt: &str) -> Option<(String, usize)> {
    let chars: Vec<char> = fmt.chars().collect();
    let mut result = String::new();
    let mut count = 0;
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '{' => { result.push_str("{{"); i += 1; }
            '}' => { result.push_str("}}"); i += 1; }
            '%' => {
                i += 1;
                if chars.get(i) == Some(&'%') { result.push('%'); i += 1; continue; }
                let mut plus = false;
                while i < chars.len() && "+- #0".contains(chars[i]) { plus |= chars[i] == '+'; i += 1; }
                while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                let precision = if chars.get(i) == Some(&'.') {
                    i += 1; let start = i;
                    while i < chars.len() && chars[i].is_ascii_digit() { i += 1; }
                    Some(chars[start..i].iter().collect::<String>())
                } else { None };
                while i < chars.len() && "hlLzjt".contains(chars[i]) { i += 1; }
                let spec = *chars.get(i)?; i += 1;
                let mut rust_spec = match spec {
                    'd' | 'i' | 'u' | 's' | 'c' => if plus { "+" } else { "" }.to_owned(),
                    'f' | 'F' | 'e' | 'E' | 'g' | 'G' => String::new(),
                    'x' => "x".into(), 'X' => "X".into(), 'o' => "o".into(),
                    _ => return None,
                };
                if matches!(spec, 'f' | 'F' | 'e' | 'E' | 'g' | 'G') {
                    if plus { rust_spec.push('+'); }
                    if let Some(p) = precision { if p.is_empty() { return None; } rust_spec.push('.'); rust_spec.push_str(&p); }
                }
                if rust_spec.is_empty() { result.push_str("{}"); }
                else { result.push_str(&format!("{{:{rust_spec}}}")); }
                count += 1;
            }
            ch => { result.push(ch); i += 1; }
        }
    }
    Some((result, count))
}

fn scanf(line: &str) -> Option<Vec<Vec<String>>> {
    let call = line.strip_prefix("scanf(")?.strip_suffix(';')?.trim_end().strip_suffix(')')?;
    let args = split_top(call, ',')?;
    let fmt = args.first()?.trim().strip_prefix('"')?.strip_suffix('"')?;
    let specs = scan_types(fmt)?;
    if specs.len() != args.len() - 1 { return None; }
    specs.iter().zip(&args[1..]).map(|(ty, target)| {
        let target = target.trim().strip_prefix('&').unwrap_or(target.trim());
        if target.is_empty() { return None; }
        let mut lines = vec![
            "let __crown_value = loop {".to_owned(),
            "    if let Some(token) = __crown_tokens.pop_front() { break token; }".to_owned(),
            "    let mut line = String::new();".to_owned(),
            "    std::io::stdin().read_line(&mut line).expect(\"failed to read stdin\");".to_owned(),
            "    __crown_tokens.extend(line.split_whitespace().map(str::to_owned));".to_owned(),
            "};".to_owned(),
        ];
        let assignment = match *ty {
            "i32" | "u32" | "i64" | "u64" | "f32" | "f64" => format!("{target} = __crown_value.parse::<{ty}>().expect(\"invalid input value\");"),
            "char" => format!("{target} = __crown_value.chars().next().expect(\"empty character input\");"),
            "String" => format!("{target} = __crown_value;"),
            _ => return None,
        };
        lines.push(assignment);
        Some(lines)
    }).collect()
}

fn scan_types(fmt: &str) -> Option<Vec<&'static str>> {
    let c: Vec<char> = fmt.chars().collect();
    let mut types = Vec::new();
    let mut i = 0;
    while i < c.len() {
        if c[i] != '%' { i += 1; continue; }
        i += 1;
        if c.get(i) == Some(&'%') { i += 1; continue; }
        while i < c.len() && "hlLzjt".contains(c[i]) { i += 1; }
        let length = c[i.saturating_sub(1)];
        let spec = *c.get(i)?; i += 1;
        types.push(match spec {
            'd' | 'i' if length == 'l' => "i64",
            'd' | 'i' => "i32",
            'u' if length == 'l' => "u64",
            'u' => "u32",
            'f' if length == 'l' => "f64",
            'f' => "f32",
            'c' => "char", 's' => "String",
            _ => return None,
        });
    }
    Some(types)
}

fn switch_header(line: &str) -> Option<String> {
    let e = line.strip_prefix("switch (")?.strip_suffix(") {")?.trim();
    (!e.is_empty()).then(|| e.to_owned())
}
fn case_label(line: &str) -> Option<String> {
    if line == "default:" { return Some("_".into()); }
    let v = line.strip_prefix("case ")?.strip_suffix(':')?.trim();
    (!v.is_empty()).then(|| v.to_owned())
}

fn ternary(line: &str) -> String {
    let semi = line.trim_end().ends_with(';');
    let body = if semi { line.trim_end().strip_suffix(';').unwrap_or(line).trim_end() } else { line.trim_end() };
    let Some((q, colon)) = ternary_bounds(body) else { return line.to_owned(); };
    let prefix = body[..q].trim_end();
    let a = body[q+1..colon].trim(); let b = body[colon+1..].trim();
    if a.is_empty() || b.is_empty() { return line.to_owned(); }
    let (leading, condition) = if let Some(ret) = prefix.strip_prefix("return ") { ("return ".to_owned(), ret.trim()) }
        else if let Some(eq) = assignment_operator(prefix) { (format!("{} = ", prefix[..eq].trim_end()), prefix[eq+1..].trim()) }
        else { (String::new(), prefix) };
    if condition.is_empty() { return line.to_owned(); }
    format!("{leading}if {} {{ {} }} else {{ {} }}{}", ternary(condition), ternary(a), ternary(b), if semi { ";" } else { "" })
}
fn ternary_bounds(s: &str) -> Option<(usize, usize)> {
    let (mut p, mut b, mut c) = (0i32, 0i32, 0i32); let (mut q, mut nested) = (None, 0usize);
    let (mut quoted, mut escaped) = (false, false);
    for (i, ch) in s.char_indices() {
        if quoted { if escaped { escaped=false; } else if ch=='\\' { escaped=true; } else if ch=='"' { quoted=false; } continue; }
        match ch {
            '"' => quoted=true, '('=>p+=1, ')'=>p-=1, '['=>b+=1, ']'=>b-=1, '{'=>c+=1, '}'=>c-=1,
            '?' if p==0 && b==0 && c==0 => if q.is_none(){q=Some(i)}else{nested+=1},
            ':' if p==0 && b==0 && c==0 && q.is_some() => if nested==0{return Some((q?,i))}else{nested-=1},
            _=>{}
        }
    }
    None
}
fn assignment_operator(s: &str) -> Option<usize> {
    let bytes=s.as_bytes(); let (mut p,mut b)=(0i32,0i32); let (mut quoted,mut escaped)=(false,false);
    for (i,ch) in s.char_indices() {
        if quoted { if escaped{escaped=false}else if ch=='\\'{escaped=true}else if ch=='"'{quoted=false}; continue; }
        match ch {
            '"'=>quoted=true,'('=>p+=1,')'=>p-=1,'['=>b+=1,']'=>b-=1,
            '=' if p==0&&b==0 => { let prev=i.checked_sub(1).and_then(|x|bytes.get(x)).copied(); let next=bytes.get(i+1).copied();
                if !matches!(prev,Some(b'=')|Some(b'!')|Some(b'<')|Some(b'>'))&&next!=Some(b'='){return Some(i)} },
            _=>{}
        }
    }
    None
}

fn do_while_close(line: &str) -> Option<String> {
    Some(line.strip_prefix("} while (")?.strip_suffix(");")?.trim().to_owned())
}
fn else_if(line: &str) -> Option<String> {
    Some(format!("}} else if {} {{", line.strip_prefix("} else if (")?.strip_suffix(") {")?))
}
fn condition(line: &str, keyword: &str) -> Option<String> {
    Some(format!("{keyword} {} {{", line.strip_prefix(keyword)?.strip_prefix(" (")?.strip_suffix(") {")?))
}
fn for_loop(line: &str) -> Option<String> {
    let inside = line.strip_prefix("for (")?.strip_suffix('{')?.trim_end().strip_suffix(')')?.trim_end();
    let mut clauses = inside.split(';');
    let init = clauses.next()?.trim();
    let cond = clauses.next()?.trim();
    let inc = clauses.next()?.trim();
    if clauses.next().is_some() { return None; }
    let init = init.strip_prefix("int ").unwrap_or(init);
    let (var, start) = init.split_once('=')?;
    let var = var.trim();
    let start = start.trim();
    let (op, condition_var, end) = if let Some((v, e)) = cond.split_once("<=") {
        ("<=", v, e)
    } else if let Some((v, e)) = cond.split_once("<") {
        ("<", v, e)
    } else if let Some((v, e)) = cond.split_once(">=") {
        (">=", v, e)
    } else if let Some((v, e)) = cond.split_once('>') {
        (">", v, e)
    } else { return None; };
    if condition_var.trim() != var { return None; }

    let post_inc = format!("{var}++");
    let pre_inc = format!("++{var}");
    let post_dec = format!("{var}--");
    let pre_dec = format!("--{var}");
    let plus_prefix = format!("{var} += ");
    let minus_prefix = format!("{var} -= ");
    let (ascending, step) = if inc == post_inc.as_str() || inc == pre_inc.as_str() {
        (true, 1)
    } else if inc == post_dec.as_str() || inc == pre_dec.as_str() {
        (false, 1)
    } else if let Some(n) = inc.strip_prefix(plus_prefix.as_str()) {
        (true, n.trim().parse::<usize>().ok()?)
    } else if let Some(n) = inc.strip_prefix(minus_prefix.as_str()) {
        (false, n.trim().parse::<usize>().ok()?)
    } else { return None; };
    let descending = op == ">" || op == ">=";
    if ascending == descending || step == 0 { return None; }

    let end = end.trim();
    let base = match op {
        "<" => format!("{start}..{end}"),
        "<=" => format!("{start}..={end}"),
        ">" => format!("({end} + 1..={start})"),
        ">=" => format!("({end}..={start})"),
        _ => return None,
    };
    let mut iterator = if descending { format!("{base}.rev()") } else { base };
    if step > 1 { iterator = format!("({iterator}).step_by({step})"); }
    Some(format!("for {var} in {iterator} {{"))
}
fn increment(line: &str) -> Option<String> {
    let s=line.strip_suffix(';')?.trim();
    for (suffix,op) in [("++","+= 1"),("--","-= 1")] {
        if let Some(name)=s.strip_suffix(suffix){let n=name.trim();return identifier(n).then(||format!("{n} {op};"));}
    }
    for (prefix,op) in [("++","+= 1"),("--","-= 1")] {
        if let Some(name)=s.strip_prefix(prefix){let n=name.trim();return identifier(n).then(||format!("{n} {op};"));}
    }
    None
}

fn split_top(s: &str, separator: char) -> Option<Vec<&str>> {
    let (mut p,mut b,mut c)=(0i32,0i32,0i32); let (mut quoted,mut escaped)=(false,false); let mut start=0;let mut out=Vec::new();
    for (i,ch) in s.char_indices(){
        if quoted{if escaped{escaped=false}else if ch=='\\'{escaped=true}else if ch=='"'{quoted=false};continue;}
        match ch{'"'=>quoted=true,'('=>p+=1,')'=>p-=1,'['=>b+=1,']'=>b-=1,'{'=>c+=1,'}'=>c-=1,
            x if x==separator&&p==0&&b==0&&c==0=>{out.push(&s[start..i]);start=i+ch.len_utf8();},_=>{}}
        if p<0||b<0||c<0{return None;}
    }
    if quoted||p!=0||b!=0||c!=0{return None;}out.push(&s[start..]);Some(out)
}
fn split_once_top(s:&str,separator:char)->Option<(&str,&str)>{let mut p=0i32;let mut b=0i32;let mut c=0i32;let(mut quoted,mut escaped)=(false,false);
    for(i,ch)in s.char_indices(){if quoted{if escaped{escaped=false}else if ch=='\\'{escaped=true}else if ch=='"'{quoted=false};continue;}
        match ch{'"'=>quoted=true,'('=>p+=1,')'=>p-=1,'['=>b+=1,']'=>b-=1,'{'=>c+=1,'}'=>c-=1,x if x==separator&&p==0&&b==0&&c==0=>return Some((&s[..i],&s[i+1..])),_=>{}}}None}
fn identifier(s:&str)->bool{let mut c=s.chars();matches!(c.next(),Some(x)if x=='_'||x.is_ascii_alphabetic())&&c.all(|x|x=='_'||x.is_ascii_alphanumeric())}

fn adjust_depth(line:&str,depth:&mut usize,main:&mut Option<usize>,block:&mut bool){
    #[derive(Clone,Copy,PartialEq)]enum S{Code,Str,Char,Line,Block}
    let b=line.as_bytes();let mut s=if *block{S::Block}else{S::Code};let(mut esc,mut i)=(false,0);
    while i<b.len(){let x=b[i];let n=b.get(i+1).copied();match s{
        S::Code=>match(x,n){(b'/',Some(b'/'))=>s=S::Line,(b'/',Some(b'*'))=>{s=S::Block;*block=true;i+=1},(b'"',_)=>s=S::Str,(b'\'',_)=>s=S::Char,(b'{',_)=>*depth+=1,(b'}',_)=>{if *main==Some(*depth){*main=None}*depth=(*depth).saturating_sub(1)},_=>{}},
        S::Str|S::Char=>if esc{esc=false}else if x==b'\\'{esc=true}else if(s==S::Str&&x==b'"')||(s==S::Char&&x==b'\''){s=S::Code},
        S::Line=>break,S::Block if x==b'*'&&n==Some(b'/')=>{s=S::Code;*block=false;i+=1},S::Block=>{}}
        i+=1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn reported_program_is_translated(){
        let c = r#"#include <stdio.h>
int main() {
    int number1, number2, sum;
    printf("Enter two integers: ");
    scanf("%d %d", &number1, &number2);
    sum = number1 + number2;
    printf("%d + %d = %d", number1, number2, sum);
    return 0;
}
"#;
        let out=rewrite_c_to_rust(c);assert!(out.contains("let mut number1: i32 = 0;"));assert!(out.contains("number2 = __crown_value.parse::<i32>()"));assert!(out.contains("print!(\"{} + {} = {}\", number1, number2, sum);"));assert!(!out.contains("scanf("));
    }
    #[test] fn translates_arrays_and_extended_loops(){
        assert_eq!(declaration("int matrix[2][2] = {{1, 2}, {3, 4}};"),Some(vec!["let mut matrix: [[i32; 2]; 2] = [[1, 2], [3, 4]];".into()]));
        assert_eq!(for_loop("for (int i = 10; i > 0; i--) {"),Some("for i in (0 + 1..=10).rev() {".into()));
        assert_eq!(for_loop("for (int i = 0; i <= 10; i += 2) {"),Some("for i in (0..=10).step_by(2) {".into()));
    }
    #[test] fn translates_control_flow_extensions(){
        assert_eq!(ternary("largest = a > b ? a : b;"),"largest = if a > b { a } else { b };");
        assert_eq!(do_while_close("} while (n < 10);"),Some("n < 10".into()));
        assert_eq!(switch_header("switch (choice) {"),Some("choice".into()));
    }
    #[test] fn translates_switch(){
        let out=rewrite_c_to_rust("int main() {\n int n = 1;\n switch (n) {\n case 1:\n  printf(\"one\");\n  break;\n default:\n  printf(\"other\");\n }\n return 0;\n}");
        assert!(out.contains("match n {"));assert!(out.contains("1 => {"));assert!(out.contains("_ => {"));assert!(!out.contains("break;"));
    }
}
