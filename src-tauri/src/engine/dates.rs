use super::i18n::Keywords;
use chrono::{Local, TimeZone, Datelike, Timelike, Duration, NaiveDate};

/// Try to evaluate a date arithmetic expression
/// "today + 17 days" -> "21/08/2026"
/// "now" -> "21/08/2026 14:30:05"
/// "agora + 1 hora e 12 minutos" -> "24/08/2026 15:42:05"
/// "now + 1:30" -> "24/08/2026 16:00:05"
/// "08/21/2026 to BR" -> "21/08/2026"
pub fn try_date_arithmetic(expr: &str, keywords: &Keywords) -> Option<Result<String, String>> {
    let lower = expr.to_lowercase();
    let trimmed = lower.trim();

    // Check for "now"/"agora" with optional arithmetic
    if trimmed == "now" || trimmed == "agora" {
        let now = Local::now();
        return Some(Ok(format_datetime(&now, keywords)));
    }

    // Check for "now +/- ..." or "agora +/- ..."
    if let Some(result) = try_now_arithmetic(trimmed, keywords) {
        return Some(result);
    }

    // Check for "epoch"/"timestamp" -> raw unix timestamp
    if trimmed == "epoch" || trimmed == "timestamp" {
        let now = Local::now();
        return Some(Ok(now.timestamp().to_string()));
    }

    // Check for "fromunix(N)" or "fromepoch(N)" or "deunix(N)" or "deepoch(N)"
    if let Some(result) = try_from_unix(trimmed, keywords) {
        return Some(result);
    }

    // Check for "tounix(date)" or "toepoch(date)" or "paraunix(date)" or "paraepoch(date)"
    if let Some(result) = try_to_unix(trimmed, expr) {
        return Some(result);
    }

    // Check for date format conversion: "<date> to BR", "<date> to US", "<date> to ISO"
    if let Some(result) = try_date_format_conversion(expr, keywords) {
        return Some(result);
    }

    // Check if it starts with "today"/"hoje"
    let is_today = keywords.today.iter().any(|k| trimmed.starts_with(k));
    if !is_today {
        return None;
    }

    // Find the today keyword used
    let today_kw = keywords.today.iter()
        .find(|k| trimmed.starts_with(*k))
        .unwrap();

    let rest = trimmed[today_kw.len()..].trim();
    if rest.is_empty() {
        let now = get_today();
        return Some(Ok(format_date(now.0, now.1, now.2, keywords)));
    }

    // Parse operator: + or -
    let (op, after_op) = if rest.starts_with('+') {
        ('+', rest[1..].trim())
    } else if rest.starts_with('-') {
        ('-', rest[1..].trim())
    } else {
        let msg = if is_pt(keywords) {
            "Esperado + ou - após hoje"
        } else {
            "Expected + or - after today"
        };
        return Some(Err(msg.to_string()));
    };

    // Try compound time parse (e.g. "1:30" or "1 hora e 30 minutos")
    if let Some(total_secs) = parse_compound_duration(after_op, keywords) {
        let sign: i64 = if op == '+' { 1 } else { -1 };
        let now = Local::now() + Duration::seconds(sign * total_secs);
        return Some(Ok(format_datetime(&now, keywords)));
    }

    // Parse number and unit: "17 days"
    let parts: Vec<&str> = after_op.splitn(2, char::is_whitespace).collect();
    if parts.len() < 2 {
        if let Ok(n) = after_op.parse::<i64>() {
            let today = get_today();
            let result = add_days(today, if op == '+' { n } else { -n });
            return Some(Ok(format_date(result.0, result.1, result.2, keywords)));
        }
        let msg = if is_pt(keywords) {
            "+/- N dias/semanas/meses/anos"
        } else {
            "+/- N days/weeks/months/years"
        };
        return Some(Err(msg.to_string()));
    }

    let number: i64 = match parts[0].parse() {
        Ok(n) => if op == '+' { n } else { -n },
        Err(_) => {
            let msg = if is_pt(keywords) {
                format!("Número inválido: {}", parts[0])
            } else {
                format!("Invalid number: {}", parts[0])
            };
            return Some(Err(msg));
        }
    };

    let unit = parts[1].trim();
    let today = get_today();

    let result = if keywords.days.iter().any(|k| unit == *k) {
        add_days(today, number)
    } else if keywords.weeks.iter().any(|k| unit == *k) {
        add_days(today, number * 7)
    } else if keywords.months.iter().any(|k| unit == *k) {
        add_months(today, number)
    } else if keywords.years.iter().any(|k| unit == *k) {
        add_years(today, number)
    } else if keywords.hours.iter().any(|k| unit == *k) {
        let now = Local::now() + Duration::hours(number);
        if is_pt(keywords) {
            return Some(Ok(format!("{:02}/{:02}/{} {:02}:{:02}",
                now.day(), now.month(), now.year(), now.hour(), now.minute())));
        } else {
            return Some(Ok(format!("{}/{}/{} {:02}:{:02}",
                now.month(), now.day(), now.year(), now.hour(), now.minute())));
        }
    } else if keywords.minutes.iter().any(|k| unit == *k) {
        let now = Local::now() + Duration::minutes(number);
        if is_pt(keywords) {
            return Some(Ok(format!("{:02}/{:02}/{} {:02}:{:02}",
                now.day(), now.month(), now.year(), now.hour(), now.minute())));
        } else {
            return Some(Ok(format!("{}/{}/{} {:02}:{:02}",
                now.month(), now.day(), now.year(), now.hour(), now.minute())));
        }
    } else if keywords.seconds.iter().any(|k| unit == *k) {
        let now = Local::now() + Duration::seconds(number);
        return Some(Ok(format_datetime(&now, keywords)));
    } else {
        let msg = if is_pt(keywords) {
            format!("Unidade de tempo desconhecida: {}", unit)
        } else {
            format!("Unknown time unit: {}", unit)
        };
        return Some(Err(msg));
    };

    Some(Ok(format_date(result.0, result.1, result.2, keywords)))
}

/// Handle "now +/- duration" and "agora +/- duration"
fn try_now_arithmetic(trimmed: &str, keywords: &Keywords) -> Option<Result<String, String>> {
    let now_keywords = ["now", "agora"];

    for kw in &now_keywords {
        if !trimmed.starts_with(kw) {
            continue;
        }
        let rest = trimmed[kw.len()..].trim();
        if rest.is_empty() {
            continue; // handled elsewhere as bare "now"
        }

        // Must start with + or -
        let (op, after_op) = if rest.starts_with('+') {
            ('+', rest[1..].trim())
        } else if rest.starts_with('-') {
            ('-', rest[1..].trim())
        } else {
            continue;
        };

        let sign: i64 = if op == '+' { 1 } else { -1 };

        // Try compound duration: "1:30", "1 hora e 12 minutos", "2 hours and 30 minutes"
        if let Some(total_secs) = parse_compound_duration(after_op, keywords) {
            let result = Local::now() + Duration::seconds(sign * total_secs);
            return Some(Ok(format_datetime(&result, keywords)));
        }

        // Try single unit: "3 hours", "45 minutes"
        let parts: Vec<&str> = after_op.splitn(2, char::is_whitespace).collect();
        if parts.len() >= 2 {
            if let Ok(n) = parts[0].parse::<i64>() {
                let unit = parts[1].trim();
                let secs = unit_to_seconds(unit, keywords);
                if secs > 0 {
                    let result = Local::now() + Duration::seconds(sign * n * secs);
                    return Some(Ok(format_datetime(&result, keywords)));
                }
            }
        }

        // Try bare number (assume minutes)
        if let Ok(n) = after_op.parse::<i64>() {
            let result = Local::now() + Duration::minutes(sign * n);
            return Some(Ok(format_datetime(&result, keywords)));
        }

        return None;
    }

    None
}

/// Parse compound durations:
/// - "1:12" -> 1h12m = 4320s
/// - "1:30:15" -> 1h30m15s
/// - "1 hora e 12 minutos" -> 4320s
/// - "2 hours and 30 minutes" -> 9000s
/// - "1 hora 12 minutos e 30 segundos" -> compound
fn parse_compound_duration(s: &str, keywords: &Keywords) -> Option<i64> {
    // Try HH:MM or HH:MM:SS format first
    if let Some(secs) = parse_colon_duration(s) {
        return Some(secs);
    }

    // Try natural language: "1 hora e 12 minutos", "2 hours and 30 minutes"
    if let Some(secs) = parse_natural_compound(s, keywords) {
        return Some(secs);
    }

    None
}

/// Parse "1:12" -> 4320 seconds, "1:30:15" -> 5415 seconds
fn parse_colon_duration(s: &str) -> Option<i64> {
    let s = s.trim();
    let parts: Vec<&str> = s.split(':').collect();

    match parts.len() {
        2 => {
            let h: i64 = parts[0].trim().parse().ok()?;
            let m: i64 = parts[1].trim().parse().ok()?;
            if m < 0 || m >= 60 { return None; }
            Some(h * 3600 + m * 60)
        }
        3 => {
            let h: i64 = parts[0].trim().parse().ok()?;
            let m: i64 = parts[1].trim().parse().ok()?;
            let sec: i64 = parts[2].trim().parse().ok()?;
            if m < 0 || m >= 60 || sec < 0 || sec >= 60 { return None; }
            Some(h * 3600 + m * 60 + sec)
        }
        _ => None,
    }
}

/// Parse "1 hora e 12 minutos", "2 hours and 30 minutes and 10 seconds"
fn parse_natural_compound(s: &str, keywords: &Keywords) -> Option<i64> {
    // Split by "e" or "and" or just whitespace-separated segments
    let separators = [" e ", " and ", ", "];
    let segments = split_by_any(s, &separators);

    if segments.len() < 2 {
        return None; // Single unit handled elsewhere
    }

    let mut total_secs: i64 = 0;
    let mut found_any = false;

    for segment in &segments {
        let seg = segment.trim();
        if seg.is_empty() {
            continue;
        }

        let parts: Vec<&str> = seg.splitn(2, char::is_whitespace).collect();
        if parts.len() != 2 {
            return None;
        }

        let n: i64 = parts[0].parse().ok()?;
        let unit = parts[1].trim();
        let unit_secs = unit_to_seconds(unit, keywords);
        if unit_secs == 0 {
            return None;
        }

        total_secs += n * unit_secs;
        found_any = true;
    }

    if found_any { Some(total_secs) } else { None }
}

/// Split string by multiple separators
fn split_by_any<'a>(s: &'a str, separators: &[&str]) -> Vec<&'a str> {
    let mut result = vec![s];

    for sep in separators {
        let mut new_result = Vec::new();
        for part in &result {
            for sub in part.split(sep) {
                if !sub.trim().is_empty() {
                    new_result.push(sub);
                }
            }
        }
        result = new_result;
    }

    result
}

/// Convert a time unit keyword to seconds
fn unit_to_seconds(unit: &str, keywords: &Keywords) -> i64 {
    if keywords.hours.iter().any(|k| unit == *k) {
        3600
    } else if keywords.minutes.iter().any(|k| unit == *k) {
        60
    } else if keywords.seconds.iter().any(|k| unit == *k) {
        1
    } else {
        0
    }
}

/// Format a chrono DateTime as local date+time
fn format_datetime<Tz: chrono::TimeZone>(dt: &chrono::DateTime<Tz>, keywords: &Keywords) -> String
where
    Tz::Offset: std::fmt::Display,
{
    if is_pt(keywords) {
        format!("{:02}/{:02}/{} {:02}:{:02}:{:02}",
            dt.day(), dt.month(), dt.year(),
            dt.hour(), dt.minute(), dt.second())
    } else {
        format!("{}/{}/{} {:02}:{:02}:{:02}",
            dt.month(), dt.day(), dt.year(),
            dt.hour(), dt.minute(), dt.second())
    }
}

// === Date format conversion ===

fn try_date_format_conversion(expr: &str, keywords: &Keywords) -> Option<Result<String, String>> {
    let lower = expr.to_lowercase();

    for conv_kw in keywords.conversion.iter() {
        let patterns = [
            format!(" {} br", conv_kw),
            format!(" {} us", conv_kw),
            format!(" {} iso", conv_kw),
            format!(" {} dd/mm/yyyy", conv_kw),
            format!(" {} mm/dd/yyyy", conv_kw),
            format!(" {} yyyy-mm-dd", conv_kw),
        ];

        for pattern in &patterns {
            if let Some(pos) = lower.rfind(pattern.as_str()) {
                let date_part = expr[..pos].trim();
                let target = pattern.split_whitespace().last().unwrap();

                if let Some((year, month, day, time)) = parse_date_flexible(date_part) {
                    let formatted = match target {
                        "br" | "dd/mm/yyyy" => {
                            let base = format!("{:02}/{:02}/{}", day, month, year);
                            if let Some((h, m, s)) = time {
                                format!("{} {:02}:{:02}:{:02}", base, h, m, s)
                            } else {
                                base
                            }
                        }
                        "us" | "mm/dd/yyyy" => {
                            let base = format!("{:02}/{:02}/{}", month, day, year);
                            if let Some((h, m, s)) = time {
                                format!("{} {:02}:{:02}:{:02}", base, h, m, s)
                            } else {
                                base
                            }
                        }
                        "iso" | "yyyy-mm-dd" => {
                            let base = format!("{}-{:02}-{:02}", year, month, day);
                            if let Some((h, m, s)) = time {
                                format!("{}T{:02}:{:02}:{:02}", base, h, m, s)
                            } else {
                                base
                            }
                        }
                        _ => return None,
                    };
                    return Some(Ok(formatted));
                } else {
                    continue;
                }
            }
        }
    }
    None
}

/// Parse a date string flexibly, returning (year, month, day, Option<(hour, min, sec)>)
fn parse_date_flexible(s: &str) -> Option<(i32, u32, u32, Option<(u32, u32, u32)>)> {
    let s = s.trim();

    let (date_str, time_str) = if s.contains('T') {
        let parts: Vec<&str> = s.splitn(2, 'T').collect();
        (parts[0], Some(parts[1]))
    } else if let Some(space_pos) = s.rfind(' ') {
        let potential_time = &s[space_pos + 1..];
        if potential_time.contains(':') {
            (&s[..space_pos], Some(potential_time))
        } else {
            (s, None)
        }
    } else {
        (s, None)
    };

    let (year, month, day) = parse_date_part(date_str)?;

    let time = if let Some(t) = time_str {
        parse_time_part(t)
    } else {
        None
    };

    Some((year, month, day, time))
}

fn parse_date_part(s: &str) -> Option<(i32, u32, u32)> {
    let parts: Vec<&str> = if s.contains('-') {
        s.split('-').collect()
    } else if s.contains('/') {
        s.split('/').collect()
    } else {
        return None;
    };

    if parts.len() != 3 {
        return None;
    }

    if parts[0].len() == 4 {
        let y: i32 = parts[0].parse().ok()?;
        let m: u32 = parts[1].parse().ok()?;
        let d: u32 = parts[2].parse().ok()?;
        Some((y, m, d))
    } else if parts[2].len() == 4 {
        let first: u32 = parts[0].parse().ok()?;
        let second: u32 = parts[1].parse().ok()?;
        let y: i32 = parts[2].parse().ok()?;

        if first > 12 {
            Some((y, second, first))
        } else if second > 12 {
            Some((y, first, second))
        } else {
            Some((y, second, first))
        }
    } else {
        None
    }
}

fn parse_time_part(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim().trim_end_matches("UTC").trim_end_matches("utc").trim();
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        2 => {
            let h: u32 = parts[0].parse().ok()?;
            let m: u32 = parts[1].parse().ok()?;
            Some((h, m, 0))
        }
        3 => {
            let h: u32 = parts[0].parse().ok()?;
            let m: u32 = parts[1].parse().ok()?;
            let s: u32 = parts[2].parse::<f64>().ok()? as u32;
            Some((h, m, s))
        }
        _ => None,
    }
}

// Simple date tuple: (year, month, day)
type DateTuple = (i32, u32, u32);

fn get_today() -> DateTuple {
    let now = Local::now();
    (now.year(), now.month(), now.day())
}

fn unix_days_to_date(days: i64) -> DateTuple {
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn date_to_unix_days(year: i32, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year as i64 - 1 } else { year as i64 };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = (y - era * 400) as u32;
    let m = month;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

fn add_days(date: DateTuple, days: i64) -> DateTuple {
    let unix_days = date_to_unix_days(date.0, date.1, date.2);
    unix_days_to_date(unix_days + days)
}

fn add_months(date: DateTuple, months: i64) -> DateTuple {
    let total_months = date.0 as i64 * 12 + (date.1 as i64 - 1) + months;
    let new_year = (total_months / 12) as i32;
    let new_month = (total_months % 12 + 1) as u32;
    let max_day = days_in_month(new_year, new_month);
    let new_day = date.2.min(max_day);
    (new_year, new_month, new_day)
}

fn add_years(date: DateTuple, years: i64) -> DateTuple {
    add_months(date, years * 12)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn format_date(year: i32, month: u32, day: u32, keywords: &Keywords) -> String {
    if is_pt(keywords) {
        format!("{:02}/{:02}/{}", day, month, year)
    } else {
        format!("{}/{}/{}", month, day, year)
    }
}

fn is_pt(keywords: &Keywords) -> bool {
    keywords.today.contains(&"hoje")
}

/// Convert unix timestamp to human-readable date (local timezone)
fn try_from_unix(expr: &str, keywords: &Keywords) -> Option<Result<String, String>> {
    let prefixes = ["fromunix(", "fromepoch(", "deunix(", "deepoch(", "from_unix(", "from_epoch("];

    for prefix in &prefixes {
        if expr.starts_with(prefix) && expr.ends_with(')') {
            let inner = &expr[prefix.len()..expr.len()-1].trim();
            return Some(parse_and_convert_timestamp(inner, keywords));
        }
    }

    let kw_prefixes = ["fromunix ", "fromepoch ", "deunix ", "deepoch "];
    for prefix in &kw_prefixes {
        if expr.starts_with(prefix) {
            let inner = &expr[prefix.len()..].trim();
            return Some(parse_and_convert_timestamp(inner, keywords));
        }
    }

    None
}

fn parse_and_convert_timestamp(s: &str, keywords: &Keywords) -> Result<String, String> {
    let ts: i64 = s.parse().map_err(|_| {
        if is_pt(keywords) {
            format!("Timestamp inválido: {}", s)
        } else {
            format!("Invalid timestamp: {}", s)
        }
    })?;

    let secs = if ts > 1_000_000_000_000_000 {
        ts / 1_000_000
    } else if ts > 1_000_000_000_000 {
        ts / 1_000
    } else {
        ts
    };

    let dt = Local.timestamp_opt(secs, 0)
        .single()
        .ok_or_else(|| "Invalid timestamp".to_string())?;

    Ok(format_datetime(&dt, keywords))
}

/// Convert a date string to unix timestamp
fn try_to_unix(expr: &str, _original: &str) -> Option<Result<String, String>> {
    let prefixes = ["tounix(", "toepoch(", "paraunix(", "paraepoch(", "to_unix(", "to_epoch("];

    for prefix in &prefixes {
        if expr.starts_with(prefix) && expr.ends_with(')') {
            let inner = &expr[prefix.len()..expr.len()-1].trim();
            return Some(parse_date_to_unix(inner));
        }
    }

    let kw_prefixes = ["tounix ", "toepoch ", "paraunix ", "paraepoch "];
    for prefix in &kw_prefixes {
        if expr.starts_with(prefix) {
            let inner = &expr[prefix.len()..].trim();
            return Some(parse_date_to_unix(inner));
        }
    }

    None
}

fn parse_date_to_unix(s: &str) -> Result<String, String> {
    if let Some((year, month, day, time)) = parse_date_flexible(s) {
        let (h, m, sec) = time.unwrap_or((0, 0, 0));
        let naive = NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|d| d.and_hms_opt(h, m, sec))
            .ok_or_else(|| "Data inválida".to_string())?;
        let local_dt = Local.from_local_datetime(&naive)
            .single()
            .ok_or_else(|| "Data inválida para fuso horário local".to_string())?;
        Ok(local_dt.timestamp().to_string())
    } else {
        Err("Formato esperado: DD/MM/YYYY [HH:MM[:SS]] ou YYYY-MM-DD[THH:MM[:SS]]".to_string())
    }
}
