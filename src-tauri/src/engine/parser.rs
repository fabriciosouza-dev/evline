use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Op(char),
    LeftParen,
    RightParen,
    Function(String),
}

/// Word operators mapped to their symbol equivalents
fn word_to_op(word: &str) -> Option<char> {
    match word.to_lowercase().as_str() {
        // EN
        "plus" | "and" | "with" => Some('+'),
        "minus" | "subtract" | "without" => Some('-'),
        "times" | "multiplied" | "mul" => Some('*'),
        "divide" | "divided" => Some('/'),
        "mod" => Some('%'),
        // PT-BR
        "mais" => Some('+'),
        "menos" => Some('-'),
        "vezes" => Some('*'),
        "dividido" => Some('/'),
        _ => None,
    }
}

pub fn tokenize(expr: &str, variables: &HashMap<String, f64>) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Hex: 0x..., Binary: 0b..., Octal: 0o...
        if c == '0' && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next == 'x' || next == 'X' {
                i += 2;
                let start = i;
                while i < chars.len() && chars[i].is_ascii_hexdigit() {
                    i += 1;
                }
                if i == start {
                    return Err("Invalid hex number".to_string());
                }
                let hex_str: String = chars[start..i].iter().collect();
                let val = u64::from_str_radix(&hex_str, 16).map_err(|_| format!("Invalid hex: 0x{}", hex_str))?;
                tokens.push(Token::Number(val as f64));
                continue;
            } else if next == 'b' || next == 'B' {
                i += 2;
                let start = i;
                while i < chars.len() && (chars[i] == '0' || chars[i] == '1') {
                    i += 1;
                }
                if i == start {
                    return Err("Invalid binary number".to_string());
                }
                let bin_str: String = chars[start..i].iter().collect();
                let val = u64::from_str_radix(&bin_str, 2).map_err(|_| format!("Invalid binary: 0b{}", bin_str))?;
                tokens.push(Token::Number(val as f64));
                continue;
            } else if next == 'o' || next == 'O' {
                i += 2;
                let start = i;
                while i < chars.len() && chars[i] >= '0' && chars[i] <= '7' {
                    i += 1;
                }
                if i == start {
                    return Err("Invalid octal number".to_string());
                }
                let oct_str: String = chars[start..i].iter().collect();
                let val = u64::from_str_radix(&oct_str, 8).map_err(|_| format!("Invalid octal: 0o{}", oct_str))?;
                tokens.push(Token::Number(val as f64));
                continue;
            }
        }

        // Numbers (including decimals)
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_' || chars[i] == ',') {
                // Don't consume comma if it's followed by a space (could be list separator)
                if chars[i] == ',' {
                    if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                        i += 1;
                    } else {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            // Support k, M, B suffixes
            let num_str: String = chars[start..i].iter().filter(|c| **c != '_' && **c != ',').collect();
            let multiplier = if i < chars.len() {
                match chars[i] {
                    'k' | 'K' => { i += 1; 1_000.0 }
                    'M' => { i += 1; 1_000_000.0 }
                    'B' => { i += 1; 1_000_000_000.0 }
                    _ => 1.0,
                }
            } else {
                1.0
            };
            let val: f64 = num_str.parse().map_err(|_| format!("Invalid number: {}", num_str))?;
            tokens.push(Token::Number(val * multiplier));
            continue;
        }

        // Operators
        if "+-*/^%".contains(c) {
            // Handle negative numbers (unary minus)
            if c == '-' && (tokens.is_empty() || matches!(tokens.last(), Some(Token::Op(_)) | Some(Token::LeftParen))) {
                i += 1;
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                if i > start {
                    let num_str: String = chars[start..i].iter().collect();
                    let val: f64 = num_str.parse().map_err(|_| format!("Invalid number: -{}", num_str))?;
                    tokens.push(Token::Number(-val));
                } else {
                    tokens.push(Token::Op('-'));
                }
                continue;
            }
            tokens.push(Token::Op(c));
            i += 1;
            continue;
        }

        // Parentheses
        if c == '(' {
            tokens.push(Token::LeftParen);
            i += 1;
            continue;
        }
        if c == ')' {
            tokens.push(Token::RightParen);
            i += 1;
            continue;
        }

        // R$ prefix (two characters) — must be checked before identifiers
        if c == 'R' && i + 1 < chars.len() && chars[i + 1] == '$' {
            i += 2;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_' || chars[i] == ',') {
                if chars[i] == ',' {
                    if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                        i += 1;
                    } else {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            if i > start {
                let num_str: String = chars[start..i].iter().filter(|ch| **ch != '_' && **ch != ',').collect();
                let val: f64 = num_str.parse().map_err(|_| format!("Invalid number: {}", num_str))?;
                tokens.push(Token::Number(val));
            }
            continue;
        }

        // Identifiers (variables, functions, or word operators)
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let ident: String = chars[start..i].iter().collect();

            // Check for word operators first
            if let Some(op) = word_to_op(&ident) {
                // Skip "by" after "multiplied" or "divided"
                let rest: String = chars[i..].iter().collect();
                let trimmed = rest.trim_start();
                if trimmed.starts_with("by") {
                    // Skip "by"
                    let skip = rest.len() - trimmed.len() + 2;
                    i += skip;
                }
                tokens.push(Token::Op(op));
                continue;
            }

            // Check for built-in constants
            match ident.as_str() {
                "pi" | "PI" => tokens.push(Token::Number(std::f64::consts::PI)),
                "e" | "E" => tokens.push(Token::Number(std::f64::consts::E)),
                "tau" => tokens.push(Token::Number(std::f64::consts::TAU)),
                _ => {
                    // Check if it's a function (followed by '(')
                    let mut j = i;
                    while j < chars.len() && chars[j].is_whitespace() {
                        j += 1;
                    }
                    if j < chars.len() && chars[j] == '(' {
                        tokens.push(Token::Function(ident));
                    } else if let Some(&val) = variables.get(&ident) {
                        tokens.push(Token::Number(val));
                    } else {
                        return Err(format!("Unknown variable: {}", ident));
                    }
                }
            }
            continue;
        }

        // Currency symbols ($, €, £, ¥) — treat as prefix for a number
        if c == '$' || c == '€' || c == '£' || c == '¥' {
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_' || chars[i] == ',') {
                if chars[i] == ',' {
                    if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                        i += 1;
                    } else {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            if i > start {
                let num_str: String = chars[start..i].iter().filter(|ch| **ch != '_' && **ch != ',').collect();
                let val: f64 = num_str.parse().map_err(|_| format!("Invalid number: {}", num_str))?;
                tokens.push(Token::Number(val));
            }
            continue;
        }

        return Err(format!("Unexpected character: {}", c));
    }

    Ok(tokens)
}
