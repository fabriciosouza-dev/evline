use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod parser;
mod evaluator;
mod units;
pub mod i18n;
pub mod currency;
mod dates;

use i18n::{Keywords, Locale, get_keywords};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineResult {
    pub line: usize,
    pub input: String,
    pub output: String,
    pub result_type: ResultType,
    pub numeric_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResultType {
    Number,
    Currency { symbol: String },
    Unit { unit: String },
    Date,
    Error,
    Comment,
    Empty,
}

pub struct Engine {
    variables: HashMap<String, f64>,
    locale: Locale,
    keywords: Keywords,
    line_values: Vec<Option<f64>>,
    currency_rates: HashMap<String, f64>,
    prev_value: Option<f64>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            locale: Locale::En,
            keywords: get_keywords(Locale::En),
            line_values: Vec::new(),
            currency_rates: HashMap::new(),
            prev_value: None,
        }
    }

    pub fn set_locale(&mut self, locale: Locale) {
        self.locale = locale;
        self.keywords = get_keywords(locale);
    }

    pub fn set_currency_rates(&mut self, rates: HashMap<String, f64>) {
        self.currency_rates = rates;
    }

    pub fn evaluate_document(&mut self, text: &str, auto_detect: bool) -> Vec<LineResult> {
        self.variables.clear();
        self.line_values.clear();
        self.prev_value = None;

        if auto_detect {
            let detected = i18n::detect_locale(text);
            self.set_locale(detected);
        }

        let lines: Vec<&str> = text.lines().collect();
        let mut results = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let result = self.evaluate_line(i, line);
            if let Some(val) = result.numeric_value {
                self.line_values.push(Some(val));
                self.prev_value = Some(val);
            } else {
                self.line_values.push(None);
            }
            results.push(result);
        }

        results
    }

    pub fn get_sum(&self) -> f64 {
        self.line_values.iter().filter_map(|v| *v).sum()
    }

    pub fn get_average(&self) -> f64 {
        let values: Vec<f64> = self.line_values.iter().filter_map(|v| *v).collect();
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }

    fn evaluate_line(&mut self, line_num: usize, input: &str) -> LineResult {
        let trimmed = input.trim();

        // Empty line - resets sum/avg context
        if trimmed.is_empty() {
            return LineResult {
                line: line_num,
                input: input.to_string(),
                output: String::new(),
                result_type: ResultType::Empty,
                numeric_value: None,
            };
        }

        // Comment lines (// or #)
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            return LineResult {
                line: line_num,
                input: input.to_string(),
                output: String::new(),
                result_type: ResultType::Comment,
                numeric_value: None,
            };
        }

        // Easter eggs
        if let Some((egg, egg_val)) = self.try_easter_egg(trimmed) {
            return LineResult {
                line: line_num,
                input: input.to_string(),
                output: egg,
                result_type: ResultType::Date,
                numeric_value: egg_val,
            };
        }

        // Strip label (text before ':')
        let expr = self.strip_label(trimmed);

        // Try date arithmetic first
        if let Some(result) = self.try_date_arithmetic(expr) {
            return match result {
                Ok(date_str) => LineResult {
                    line: line_num,
                    input: input.to_string(),
                    output: date_str,
                    result_type: ResultType::Date,
                    numeric_value: None,
                },
                Err(e) => LineResult {
                    line: line_num,
                    input: input.to_string(),
                    output: e,
                    result_type: ResultType::Error,
                    numeric_value: None,
                },
            };
        }

        // Try inverse percentage: "20% of what is 30"
        if let Some(result) = self.try_inverse_percentage(expr) {
            return self.make_result(line_num, input, result);
        }

        // Try advanced percentage: "20% of $10", "5% on $30", "6% off 40"
        if let Some(result) = self.try_advanced_percentage(expr) {
            return self.make_result(line_num, input, result);
        }

        // Variable assignment: name = expr
        if let Some(eq_pos) = expr.find('=') {
            let left = expr[..eq_pos].trim();
            let right = expr[eq_pos + 1..].trim();

            if is_valid_identifier(left) && !right.is_empty() {
                let result = self.eval_expr(right);
                if let Ok((val, _rt)) = &result {
                    self.variables.insert(left.to_string(), *val);
                }
                return self.make_result(line_num, input, result);
            }
        }

        // Plain expression
        let result = self.eval_expr(expr);
        self.make_result(line_num, input, result)
    }

    fn strip_label<'a>(&self, input: &'a str) -> &'a str {
        if let Some(colon_pos) = input.find(':') {
            let before = &input[..colon_pos];
            let is_label = !before.contains(|c: char| "+-*/()^%=".contains(c))
                && !before.trim().is_empty();
            if is_label {
                return input[colon_pos + 1..].trim();
            }
        }
        input
    }

    fn make_result(
        &self,
        line_num: usize,
        input: &str,
        result: Result<(f64, ResultType), String>,
    ) -> LineResult {
        match result {
            Ok((val, rt)) => {
                let output = self.format_result(val, &rt);
                LineResult {
                    line: line_num,
                    input: input.to_string(),
                    output,
                    result_type: rt,
                    numeric_value: Some(val),
                }
            }
            Err(e) => LineResult {
                line: line_num,
                input: input.to_string(),
                output: self.translate_error(&e),
                result_type: ResultType::Error,
                numeric_value: None,
            },
        }
    }

    // === Easter eggs ===

    fn try_easter_egg(&self, expr: &str) -> Option<(String, Option<f64>)> {
        let lower = expr.to_lowercase();
        let pt = self.locale == Locale::PtBr;
        match lower.as_str() {
            "hello" | "olá" | "ola" => Some(("Hello, World! 👋".to_string(), None)),
            "404" => Some((if pt { "Não Encontrado 🚫" } else { "Not Found 🚫" }.to_string(), Some(404.0))),
            "evline" => Some(("❤️ v0.2.1".to_string(), None)),
            "credits" | "créditos" | "creditos" => Some((if pt { "Feito com ☕ por Fabricio" } else { "Made with ☕ by Fabricio" }.to_string(), None)),
            // Star Wars
            "may the force be with you" | "que a força esteja com você" | "que a forca esteja com voce" => {
                Some((if pt { "⭐ Sempre." } else { "⭐ Always." }.to_string(), None))
            }
            "i am your father" | "eu sou seu pai" => Some(("Nooooo! 😱".to_string(), None)),
            // Matrix
            "matrix" => Some((if pt { "Acorde, Neo... 💊" } else { "Wake up, Neo... 💊" }.to_string(), None)),
            // Lord of the Rings
            "one ring" | "um anel" => Some((if pt { "Um Anel para todos governar 💍" } else { "One Ring to rule them all 💍" }.to_string(), Some(1.0))),
            "you shall not pass" | "você não pode passar" | "voce nao pode passar" => {
                Some(("🧙 — Gandalf".to_string(), None))
            }
            "my precious" | "meu precioso" => Some(("Gollum! Gollum! 💍".to_string(), None)),
            // Avatar: The Last Airbender
            "yip yip" => Some((if pt { "🦬 Appa levanta voo!" } else { "🦬 Appa takes flight!" }.to_string(), None)),
            "toph" => Some((if pt { "Melhor dobradora de terra do mundo! 🪨" } else { "Greatest earthbender in the world! 🪨" }.to_string(), None)),
            // Thanos
            "thanos" => Some((if pt { "Eu sou inevitável... 🫰" } else { "I am inevitable... 🫰" }.to_string(), None)),
            // Barrel roll
            "do a barrel roll" => Some(("🛫 Aileron roll!".to_string(), None)),
            // Gravity
            "gravity" | "gravidade" => Some((if pt { "🍎 Newton aprova!" } else { "🍎 Newton approves!" }.to_string(), None)),
            // Tilt
            "askew" | "tilt" | "torto" => Some(("↗️ ~tilted~".to_string(), None)),
            // Blink
            "blink" | "piscar" => Some(("👁️ <blink> is back!".to_string(), None)),
            _ => None,
        }
    }

    fn translate_error(&self, err: &str) -> String {
        if self.locale != Locale::PtBr {
            return err.to_string();
        }
        // Translate common errors to PT-BR
        if err.starts_with("Unknown variable: ") {
            let var = &err["Unknown variable: ".len()..];
            return format!("Variável desconhecida: {}", var);
        }
        if err.starts_with("Invalid number") {
            return err.replace("Invalid number", "Número inválido");
        }
        if err == "No previous value" {
            return "Sem valor anterior".to_string();
        }
        if err == "Invalid percentage" {
            return "Porcentagem inválida".to_string();
        }
        if err == "Division by zero" {
            return "Divisão por zero".to_string();
        }
        if err == "Invalid expression" {
            return "Expressão inválida".to_string();
        }
        if err == "Empty expression" {
            return "Expressão vazia".to_string();
        }
        if err == "Mismatched parentheses" {
            return "Parênteses não correspondem".to_string();
        }
        if err.starts_with("Unknown function: ") {
            let f = &err["Unknown function: ".len()..];
            return format!("Função desconhecida: {}", f);
        }
        if err.starts_with("Unexpected character: ") {
            let c = &err["Unexpected character: ".len()..];
            return format!("Caractere inesperado: {}", c);
        }
        if err.starts_with("Cannot convert") {
            return err.replace("Cannot convert", "Não é possível converter");
        }
        err.to_string()
    }

    fn format_result(&self, val: f64, rt: &ResultType) -> String {
        let num = format_number(val);
        match rt {
            ResultType::Currency { symbol } => format!("{}{}", symbol, num),
            ResultType::Unit { unit } => format!("{} {}", num, unit),
            _ => num,
        }
    }

    fn eval_expr(&self, expr: &str) -> Result<(f64, ResultType), String> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Err(String::new());
        }

        // Check for "prev" keyword
        if self.is_prev_keyword(expr) {
            return match self.prev_value {
                Some(val) => Ok((val, ResultType::Number)),
                None => Err("No previous value".to_string()),
            };
        }

        // Check for "sum"/"total"/"soma" keyword
        if self.is_sum_keyword(expr) {
            return Ok((self.get_sum(), ResultType::Number));
        }

        // Check for "average"/"avg"/"média" keyword
        if self.is_average_keyword(expr) {
            return Ok((self.get_average(), ResultType::Number));
        }

        // Check if expression starts with prev and has operations
        if let Some(result) = self.try_prev_expression(expr) {
            return result;
        }

        // Check if expression starts with sum and has operations
        if let Some(result) = self.try_sum_expression(expr) {
            return result;
        }

        // Check for currency conversion: "$100 in BRL" or "100 USD in EUR" or "100 dollars in euros"
        if let Some(result) = self.try_currency_conversion(expr) {
            return result;
        }

        // Check for unit conversion: "10 km in miles"
        if let Some(result) = self.try_unit_conversion(expr) {
            return result;
        }

        // Check for discount keyword: "$20 - 5% discount"
        if let Some(result) = self.try_discount(expr) {
            return result;
        }

        // Check for percentage operations: "100 + 10%"
        if let Some(result) = self.try_percentage(expr) {
            return result;
        }

        // Check for currency prefix: "$7 * 4"
        if let Some(result) = self.try_currency_expr(expr) {
            return result;
        }

        // Check for currency name in expression: "20 dollars * 3"
        if let Some(result) = self.try_currency_name_expr(expr) {
            return result;
        }

        // Plain arithmetic
        let tokens = parser::tokenize(expr, &self.variables)?;
        let val = evaluator::evaluate(&tokens)?;
        Ok((val, ResultType::Number))
    }

    /// Evaluate without conversions (avoids recursion)
    fn eval_expr_inner(&self, expr: &str) -> Result<(f64, ResultType), String> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Err(String::new());
        }

        if self.is_prev_keyword(expr) {
            return match self.prev_value {
                Some(val) => Ok((val, ResultType::Number)),
                None => Err("No previous value".to_string()),
            };
        }

        if self.is_sum_keyword(expr) {
            return Ok((self.get_sum(), ResultType::Number));
        }

        if self.is_average_keyword(expr) {
            return Ok((self.get_average(), ResultType::Number));
        }

        if let Some(result) = self.try_percentage(expr) {
            return result;
        }

        if let Some(result) = self.try_currency_expr(expr) {
            return result;
        }

        let tokens = parser::tokenize(expr, &self.variables)?;
        let val = evaluator::evaluate(&tokens)?;
        Ok((val, ResultType::Number))
    }

    // === Keyword detection ===

    fn is_prev_keyword(&self, expr: &str) -> bool {
        let lower = expr.to_lowercase();
        lower == "prev" || lower == "anterior" || lower == "previous"
    }

    fn is_sum_keyword(&self, expr: &str) -> bool {
        let lower = expr.to_lowercase();
        self.keywords.sum.iter().any(|k| lower == *k)
    }

    fn is_average_keyword(&self, expr: &str) -> bool {
        let lower = expr.to_lowercase();
        matches!(lower.as_str(), "average" | "avg" | "média" | "media")
    }

    // === Prev expression ===

    fn try_prev_expression(&self, expr: &str) -> Option<Result<(f64, ResultType), String>> {
        let lower = expr.to_lowercase();
        for kw in &["prev", "anterior", "previous"] {
            if lower.starts_with(kw) && lower.len() > kw.len() {
                let rest = expr[kw.len()..].trim();
                if rest.is_empty() {
                    return None;
                }
                let prev_val = match self.prev_value {
                    Some(v) => v,
                    None => return Some(Err("No previous value".to_string())),
                };
                let new_expr = format!("{} {}", prev_val, rest);
                return Some(self.eval_expr(&new_expr));
            }
        }
        None
    }

    // === Sum expression ===

    fn try_sum_expression(&self, expr: &str) -> Option<Result<(f64, ResultType), String>> {
        let lower = expr.to_lowercase();
        for kw in self.keywords.sum.iter() {
            if lower.starts_with(kw) {
                let rest = expr[kw.len()..].trim();
                if rest.is_empty() {
                    return None;
                }
                let sum_val = self.get_sum();
                let new_expr = format!("{} {}", sum_val, rest);
                return Some(self.eval_expr(&new_expr));
            }
        }

        // "sum in USD" pattern
        for kw in self.keywords.sum.iter() {
            for conv in self.keywords.conversion.iter() {
                let prefix = format!("{} {}", kw, conv);
                if lower.starts_with(&prefix) {
                    let target = expr[prefix.len()..].trim();
                    let sum_val = self.get_sum();
                    if let Some(rate) = self.get_currency_rate(target) {
                        let converted = sum_val * rate;
                        let symbol = currency_symbol(target);
                        return Some(Ok((converted, ResultType::Currency { symbol })));
                    }
                    // Try currency name
                    if let Some(code) = currency_name_to_code(target) {
                        if let Some(rate) = self.get_currency_rate(&code) {
                            let converted = sum_val * rate;
                            let symbol = currency_symbol(&code);
                            return Some(Ok((converted, ResultType::Currency { symbol })));
                        }
                    }
                }
            }
        }

        None
    }

    // === Currency conversion ===

    fn try_currency_conversion(&self, expr: &str) -> Option<Result<(f64, ResultType), String>> {
        for conv_kw in self.keywords.conversion.iter() {
            let sep = format!(" {} ", conv_kw);
            if let Some(pos) = expr.to_lowercase().find(&sep) {
                let left = expr[..pos].trim();
                let right = expr[pos + sep.len()..].trim();

                // Resolve target: code or name
                let target = resolve_currency(right)?;

                // Try parsing left as a currency value
                if let Some((value, source_currency)) = self.parse_currency_value(left) {
                    if let Some(result) = self.convert_currency(value, &source_currency, &target) {
                        let symbol = currency_symbol(&target);
                        return Some(Ok((result, ResultType::Currency { symbol })));
                    } else {
                        return Some(Err(format!("Cannot convert to {}", target)));
                    }
                }

                // Otherwise evaluate left as expression
                match self.eval_expr_inner(left) {
                    Ok((value, _)) => {
                        if let Some(rate) = self.get_currency_rate(&target) {
                            let converted = value * rate;
                            let symbol = currency_symbol(&target);
                            return Some(Ok((converted, ResultType::Currency { symbol })));
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        None
    }

    fn parse_currency_value(&self, s: &str) -> Option<(f64, String)> {
        let s = s.trim();

        // "$100", "R$100", "€100", "£100"
        for (prefix, currency) in &[("$", "USD"), ("€", "EUR"), ("£", "GBP"), ("R$", "BRL"), ("¥", "JPY")] {
            if s.starts_with(prefix) {
                let num_str = s[prefix.len()..].trim();
                if let Ok(val) = self.parse_number(num_str) {
                    return Some((val, currency.to_string()));
                }
            }
        }

        // "100 USD" or "100 dollars" or "100 reais"
        let parts: Vec<&str> = s.rsplitn(2, char::is_whitespace).collect();
        if parts.len() == 2 {
            let potential_currency = parts[0].trim();
            let num_part = parts[1].trim();
            if let Ok(val) = self.parse_number(num_part) {
                // Try as code
                if is_currency_code(&potential_currency.to_uppercase()) {
                    return Some((val, potential_currency.to_uppercase()));
                }
                // Try as name
                if let Some(code) = currency_name_to_code(potential_currency) {
                    return Some((val, code));
                }
            }
        }

        // "100USD" (no space)
        if s.len() > 3 {
            let potential_code = &s[s.len()-3..];
            if potential_code.chars().all(|c| c.is_ascii_uppercase()) && is_currency_code(potential_code) {
                if let Ok(val) = self.parse_number(&s[..s.len()-3]) {
                    return Some((val, potential_code.to_string()));
                }
            }
        }

        None
    }

    fn parse_number(&self, s: &str) -> Result<f64, String> {
        let clean: String = s.chars().filter(|c| *c != ',' && *c != '_' && *c != ' ').collect();
        clean.parse::<f64>().map_err(|_| format!("Invalid number: {}", s))
    }

    fn convert_currency(&self, value: f64, from: &str, to: &str) -> Option<f64> {
        let from_rate = self.get_currency_rate(from)?;
        let to_rate = self.get_currency_rate(to)?;
        let usd_value = value / from_rate;
        Some(usd_value * to_rate)
    }

    fn get_currency_rate(&self, code: &str) -> Option<f64> {
        let upper = code.to_uppercase();
        if upper == "USD" {
            return Some(1.0);
        }
        self.currency_rates.get(&upper).copied()
    }

    // === Currency name in expression: "20 dollars * 3" ===

    fn try_currency_name_expr(&self, expr: &str) -> Option<Result<(f64, ResultType), String>> {
        // Match: "<number> <currency_name> <operator> <rest>"
        let lower = expr.to_lowercase();
        for (name, code) in CURRENCY_NAMES.iter() {
            let pattern = format!(" {}", name);
            if let Some(pos) = lower.find(&pattern) {
                let before = &expr[..pos].trim();
                let after = &expr[pos + pattern.len()..].trim();
                
                // Before should be a number
                if let Ok(val) = self.parse_number(before) {
                    if after.is_empty() {
                        let symbol = currency_symbol(code);
                        return Some(Ok((val, ResultType::Currency { symbol })));
                    }
                    // If there's more (like * 3), evaluate it
                    let full_expr = format!("{}{}", val, after);
                    match self.eval_expr_inner(&full_expr) {
                        Ok((result, _)) => {
                            let symbol = currency_symbol(code);
                            return Some(Ok((result, ResultType::Currency { symbol })));
                        }
                        Err(e) => return Some(Err(e)),
                    }
                }
            }
        }
        None
    }

    // === Unit conversion ===

    fn try_unit_conversion(&self, expr: &str) -> Option<Result<(f64, ResultType), String>> {
        for conv_kw in self.keywords.conversion.iter() {
            let sep = format!(" {} ", conv_kw);
            if let Some(pos) = expr.to_lowercase().find(&sep) {
                let left = &expr[..pos].trim();
                let right = &expr[pos + sep.len()..].trim();

                // Don't match currency codes or currency names
                if is_currency_code(&right.to_uppercase()) || currency_name_to_code(right).is_some() {
                    continue;
                }

                if let Some(result) = units::try_convert_parts(left, right) {
                    return Some(result.map(|(val, unit)| (val, ResultType::Unit { unit })));
                }
            }
        }
        None
    }

    // === Discount: "$20 - 5% discount" ===

    fn try_discount(&self, expr: &str) -> Option<Result<(f64, ResultType), String>> {
        let lower = expr.to_lowercase();
        // "discount" or "desconto" at the end
        let discount_kw = if lower.ends_with("discount") {
            Some("discount")
        } else if lower.ends_with("desconto") {
            Some("desconto")
        } else {
            None
        };

        if let Some(kw) = discount_kw {
            let without_kw = expr[..expr.len() - kw.len()].trim();
            // Should be "expr - N%"
            return self.try_percentage(without_kw);
        }
        None
    }

    // === Percentages ===

    fn try_percentage(&self, expr: &str) -> Option<Result<(f64, ResultType), String>> {
        let expr_trimmed = expr.trim();

        for op in [" + ", " - ", " * "] {
            if let Some(pos) = expr_trimmed.rfind(op) {
                let right = expr_trimmed[pos + op.len()..].trim();

                // Check for percentage: either "15%" or "15% something" (trailing word)
                let pct_part = if right.ends_with('%') {
                    Some(&right[..right.len() - 1])
                } else if let Some(pct_pos) = right.find('%') {
                    // "15% emergency" → extract "15" and ignore trailing word
                    let after_pct = right[pct_pos + 1..].trim();
                    // Only treat as trailing label if the rest is purely alphabetic words
                    if after_pct.is_empty() || after_pct.chars().all(|c| c.is_alphabetic() || c.is_whitespace()) {
                        Some(right[..pct_pos].trim())
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(pct_str) = pct_part {
                    let left_expr = expr_trimmed[..pos].trim();

                    let left_val = match self.eval_expr(left_expr) {
                        Ok((v, _)) => v,
                        Err(e) => return Some(Err(e)),
                    };
                    let pct_val = match pct_str.parse::<f64>() {
                        Ok(v) => v,
                        Err(_) => return Some(Err("Invalid percentage".to_string())),
                    };

                    let result = match op.trim() {
                        "+" => left_val + (left_val * pct_val / 100.0),
                        "-" => left_val - (left_val * pct_val / 100.0),
                        "*" => left_val * (pct_val / 100.0),
                        _ => return None,
                    };
                    return Some(Ok((result, ResultType::Number)));
                }
            }
        }
        None
    }

    // === Advanced percentage: "20% of $10", "5% on $30", "6% off 40" ===

    fn try_advanced_percentage(&self, expr: &str) -> Option<Result<(f64, ResultType), String>> {
        let lower = expr.to_lowercase();

        // "X% of Y" -> X/100 * Y
        if let Some(result) = self.try_pct_pattern(&lower, expr, " of ", |pct, val| val * pct / 100.0) {
            return Some(result);
        }
        if let Some(result) = self.try_pct_pattern(&lower, expr, " de ", |pct, val| val * pct / 100.0) {
            return Some(result);
        }

        // "X% on Y" -> Y + X% of Y (add percentage)
        if let Some(result) = self.try_pct_pattern(&lower, expr, " on ", |pct, val| val + val * pct / 100.0) {
            return Some(result);
        }
        if let Some(result) = self.try_pct_pattern(&lower, expr, " sobre ", |pct, val| val + val * pct / 100.0) {
            return Some(result);
        }

        // "X% off Y" -> Y - X% of Y (subtract percentage)
        if let Some(result) = self.try_pct_pattern(&lower, expr, " off ", |pct, val| val - val * pct / 100.0) {
            return Some(result);
        }

        None
    }

    fn try_pct_pattern(
        &self,
        lower: &str,
        expr: &str,
        keyword: &str,
        calc: fn(f64, f64) -> f64,
    ) -> Option<Result<(f64, ResultType), String>> {
        if let Some(pos) = lower.find(keyword) {
            let left = expr[..pos].trim();
            let right = expr[pos + keyword.len()..].trim();

            // Left should be "N%"
            if left.ends_with('%') {
                // But not "of what is" pattern
                if lower.contains("what is") || lower.contains("quanto") {
                    return None;
                }

                let pct_str = &left[..left.len() - 1].trim();
                let pct: f64 = match pct_str.parse() {
                    Ok(v) => v,
                    Err(_) => return Some(Err("Invalid percentage".to_string())),
                };

                // Evaluate right side
                match self.eval_expr(right) {
                    Ok((val, rt)) => {
                        let result = calc(pct, val);
                        return Some(Ok((result, rt)));
                    }
                    Err(e) => return Some(Err(e)),
                }
            }
        }
        None
    }

    // === Inverse percentage ===

    fn try_inverse_percentage(&self, expr: &str) -> Option<Result<(f64, ResultType), String>> {
        let lower = expr.to_lowercase();

        for pattern in self.keywords.of_what_is.iter() {
            if let Some(pos) = lower.find(pattern) {
                let left = expr[..pos].trim();
                let right = expr[pos + pattern.len()..].trim();

                if left.ends_with('%') {
                    let pct_str = &left[..left.len() - 1].trim();
                    let pct: f64 = match pct_str.parse() {
                        Ok(v) => v,
                        Err(_) => return Some(Err("Invalid percentage".to_string())),
                    };

                    let (value, unit_opt) = self.parse_value_with_optional_unit(right);
                    match value {
                        Ok(val) => {
                            let result = val * 100.0 / pct;
                            let rt = if let Some(unit) = unit_opt {
                                ResultType::Unit { unit }
                            } else {
                                ResultType::Number
                            };
                            return Some(Ok((result, rt)));
                        }
                        Err(e) => return Some(Err(e)),
                    }
                }
            }
        }
        None
    }

    fn parse_value_with_optional_unit(&self, s: &str) -> (Result<f64, String>, Option<String>) {
        let s = s.trim();
        let parts: Vec<&str> = s.rsplitn(2, char::is_whitespace).collect();
        if parts.len() == 2 {
            let potential_unit = parts[0].trim();
            let num_part = parts[1].trim();
            if let Ok(val) = num_part.parse::<f64>() {
                if units::is_known_unit(potential_unit) {
                    return (Ok(val), Some(potential_unit.to_string()));
                }
            }
        }
        match s.parse::<f64>() {
            Ok(v) => (Ok(v), None),
            Err(_) => match self.eval_expr(s) {
                Ok((v, _)) => (Ok(v), None),
                Err(e) => (Err(e), None),
            },
        }
    }

    // === Currency prefix expression ===

    fn try_currency_expr(&self, expr: &str) -> Option<Result<(f64, ResultType), String>> {
        for (prefix, currency) in &[("$", "USD"), ("€", "EUR"), ("£", "GBP"), ("R$", "BRL"), ("¥", "JPY")] {
            if expr.starts_with(prefix) {
                let rest = expr[prefix.len()..].trim();
                let lower_rest = rest.to_lowercase();
                let has_conversion = self.keywords.conversion.iter().any(|kw| {
                    lower_rest.contains(&format!(" {} ", kw))
                });
                if has_conversion {
                    continue;
                }

                let tokens = match parser::tokenize(rest, &self.variables) {
                    Ok(t) => t,
                    Err(e) => return Some(Err(e)),
                };
                let val = match evaluator::evaluate(&tokens) {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e)),
                };
                let symbol = currency_symbol(currency);
                return Some(Ok((val, ResultType::Currency { symbol })));
            }
        }
        None
    }

    fn try_date_arithmetic(&self, expr: &str) -> Option<Result<String, String>> {
        dates::try_date_arithmetic(expr, &self.keywords)
    }
}

// === Currency name mapping ===

const CURRENCY_NAMES: &[(&str, &str)] = &[
    // English
    ("dollar", "USD"), ("dollars", "USD"),
    ("euro", "EUR"), ("euros", "EUR"),
    ("pound", "GBP"), ("pounds", "GBP"),
    ("yen", "JPY"),
    ("yuan", "CNY"),
    ("franc", "CHF"), ("francs", "CHF"),
    ("rupee", "INR"), ("rupees", "INR"),
    ("won", "KRW"),
    // Portuguese
    ("dólar", "USD"), ("dólares", "USD"), ("dolar", "USD"), ("dolares", "USD"),
    ("real", "BRL"), ("reais", "BRL"),
    ("libra", "GBP"), ("libras", "GBP"),
    ("iene", "JPY"), ("ienes", "JPY"),
    ("franco", "CHF"), ("francos", "CHF"),
    ("peso", "ARS"), ("pesos", "ARS"),
    ("rupia", "INR"), ("rupias", "INR"),
];

fn currency_name_to_code(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    for (n, code) in CURRENCY_NAMES.iter() {
        if lower == *n {
            return Some(code.to_string());
        }
    }
    None
}

/// Resolve a currency string (code or name) to its ISO code
fn resolve_currency(s: &str) -> Option<String> {
    let upper = s.to_uppercase();
    if is_currency_code(&upper) {
        return Some(upper);
    }
    currency_name_to_code(s)
}

fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn format_number(val: f64) -> String {
    if val == val.floor() && val.abs() < 1e15 {
        let int_val = val as i64;
        let s = int_val.to_string();
        add_thousands_separator(&s)
    } else {
        let s = format!("{:.2}", val);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

fn add_thousands_separator(s: &str) -> String {
    let negative = s.starts_with('-');
    let digits = if negative { &s[1..] } else { s };
    if digits.len() <= 3 {
        return s.to_string();
    }
    let mut result = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    let formatted: String = result.chars().rev().collect();
    if negative { format!("-{}", formatted) } else { formatted }
}

fn currency_symbol(code: &str) -> String {
    match code.to_uppercase().as_str() {
        "USD" => "$".to_string(),
        "EUR" => "€".to_string(),
        "GBP" => "£".to_string(),
        "BRL" => "R$".to_string(),
        "JPY" | "CNY" => "¥".to_string(),
        other => format!("{} ", other),
    }
}

fn is_currency_code(s: &str) -> bool {
    let codes = [
        "USD", "EUR", "GBP", "BRL", "JPY", "CNY", "CAD", "AUD", "CHF",
        "ARS", "CLP", "COP", "MXN", "PEN", "UYU", "BOB", "PYG",
        "INR", "KRW", "SGD", "HKD", "NZD", "SEK", "NOK", "DKK",
        "PLN", "CZK", "HUF", "TRY", "ZAR", "RUB", "THB", "TWD",
    ];
    codes.contains(&s.to_uppercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_with_rates() -> Engine {
        let mut e = Engine::new();
        e.set_currency_rates(currency::fallback_rates());
        e
    }

    #[test]
    fn test_basic_arithmetic() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("2 + 3\n10 * 5\n100 / 4", true);
        assert_eq!(results[0].numeric_value, Some(5.0));
        assert_eq!(results[1].numeric_value, Some(50.0));
        assert_eq!(results[2].numeric_value, Some(25.0));
    }

    #[test]
    fn test_word_operators() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("8 times 9\n10 plus 5\n20 minus 3", true);
        assert_eq!(results[0].numeric_value, Some(72.0));
        assert_eq!(results[1].numeric_value, Some(15.0));
        assert_eq!(results[2].numeric_value, Some(17.0));
    }

    #[test]
    fn test_word_operators_pt() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("8 vezes 9\n10 mais 5\n20 menos 3", true);
        assert_eq!(results[0].numeric_value, Some(72.0));
        assert_eq!(results[1].numeric_value, Some(15.0));
        assert_eq!(results[2].numeric_value, Some(17.0));
    }

    #[test]
    fn test_prev() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("100\nprev + 50\nprev * 2", true);
        assert_eq!(results[0].numeric_value, Some(100.0));
        assert_eq!(results[1].numeric_value, Some(150.0));
        assert_eq!(results[2].numeric_value, Some(300.0));
    }

    #[test]
    fn test_average() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("10\n20\n30\naverage", true);
        assert_eq!(results[3].numeric_value, Some(20.0));
    }

    #[test]
    fn test_currency_names() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("100 dollars in euros", true);
        assert!(results[0].numeric_value.is_some());
        assert!(results[0].output.contains("€"));
    }

    #[test]
    fn test_currency_names_pt() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("100 dólares em reais", true);
        assert!(results[0].numeric_value.is_some());
        assert!(results[0].output.contains("R$"));
    }

    #[test]
    fn test_pct_of() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("20% of 100\n10% de 50", true);
        assert_eq!(results[0].numeric_value, Some(20.0));
        assert_eq!(results[1].numeric_value, Some(5.0));
    }

    #[test]
    fn test_pct_on_off() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("10% on 100\n20% off 100", true);
        assert_eq!(results[0].numeric_value, Some(110.0));
        assert_eq!(results[1].numeric_value, Some(80.0));
    }

    #[test]
    fn test_discount() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("$100 - 10% discount", true);
        assert_eq!(results[0].numeric_value, Some(90.0));
    }

    #[test]
    fn test_hex_bin_oct() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("0xFF\n0b1010\n0o77", true);
        assert_eq!(results[0].numeric_value, Some(255.0));
        assert_eq!(results[1].numeric_value, Some(10.0));
        assert_eq!(results[2].numeric_value, Some(63.0));
    }

    #[test]
    fn test_labels() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("Price: 7 * 4\nFee: 10 + 5", true);
        assert_eq!(results[0].numeric_value, Some(28.0));
        assert_eq!(results[1].numeric_value, Some(15.0));
    }

    #[test]
    fn test_sum() {
        let mut engine = engine_with_rates();
        let results = engine.evaluate_document("10\n20\n30\nsum", true);
        assert_eq!(results[3].numeric_value, Some(60.0));
    }
}
