use super::parser::Token;

pub fn evaluate(tokens: &[Token]) -> Result<f64, String> {
    if tokens.is_empty() {
        return Err(String::new());
    }

    let mut output: Vec<f64> = Vec::new();
    let mut ops: Vec<OpOrFunc> = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Number(n) => output.push(*n),
            Token::Op(op) => {
                while let Some(top) = ops.last() {
                    if let OpOrFunc::Op(top_op) = top {
                        if precedence(*top_op) >= precedence(*op) && *op != '^' {
                            apply_op(&mut output, *top_op)?;
                            ops.pop();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                ops.push(OpOrFunc::Op(*op));
            }
            Token::LeftParen => ops.push(OpOrFunc::Paren),
            Token::RightParen => {
                while let Some(top) = ops.last() {
                    match top {
                        OpOrFunc::Paren => {
                            ops.pop();
                            // Check if there's a function before the paren
                            if let Some(OpOrFunc::Func(name)) = ops.last().cloned() {
                                ops.pop();
                                apply_function(&mut output, &name)?;
                            }
                            break;
                        }
                        OpOrFunc::Op(op) => {
                            apply_op(&mut output, *op)?;
                            ops.pop();
                        }
                        OpOrFunc::Func(_) => break,
                    }
                }
            }
            Token::Function(name) => {
                ops.push(OpOrFunc::Func(name.clone()));
            }
        }
        i += 1;
    }

    while let Some(top) = ops.pop() {
        match top {
            OpOrFunc::Op(op) => apply_op(&mut output, op)?,
            OpOrFunc::Paren => return Err("Mismatched parentheses".to_string()),
            OpOrFunc::Func(name) => apply_function(&mut output, &name)?,
        }
    }

    output.pop().ok_or_else(|| "Empty expression".to_string())
}

#[derive(Debug, Clone)]
enum OpOrFunc {
    Op(char),
    Paren,
    Func(String),
}

fn precedence(op: char) -> u8 {
    match op {
        '+' | '-' => 1,
        '*' | '/' | '%' => 2,
        '^' => 3,
        _ => 0,
    }
}

fn apply_op(output: &mut Vec<f64>, op: char) -> Result<(), String> {
    let b = output.pop().ok_or("Invalid expression")?;
    let a = output.pop().ok_or("Invalid expression")?;
    let result = match op {
        '+' => a + b,
        '-' => a - b,
        '*' => a * b,
        '/' => {
            if b == 0.0 {
                return Err("Division by zero".to_string());
            }
            a / b
        }
        '%' => a % b,
        '^' => a.powf(b),
        _ => return Err(format!("Unknown operator: {}", op)),
    };
    output.push(result);
    Ok(())
}

fn apply_function(output: &mut Vec<f64>, name: &str) -> Result<(), String> {
    let arg = output.pop().ok_or(format!("Missing argument for {}", name))?;
    let result = match name {
        "sqrt" => arg.sqrt(),
        "abs" => arg.abs(),
        "sin" => arg.sin(),
        "cos" => arg.cos(),
        "tan" => arg.tan(),
        "asin" => arg.asin(),
        "acos" => arg.acos(),
        "atan" => arg.atan(),
        "ln" => arg.ln(),
        "log" | "log10" => arg.log10(),
        "log2" => arg.log2(),
        "exp" => arg.exp(),
        "ceil" => arg.ceil(),
        "floor" => arg.floor(),
        "round" => arg.round(),
        _ => return Err(format!("Unknown function: {}", name)),
    };
    output.push(result);
    Ok(())
}
