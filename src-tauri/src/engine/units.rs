use std::collections::HashMap;

/// Try to convert from a parsed left/right pair
/// left: "10 km", right: "miles"
pub fn try_convert_parts(left: &str, target_unit: &str) -> Option<Result<(f64, String), String>> {
    let (value, source_unit) = parse_value_unit(left)?;

    let conversions = get_conversions();
    let src_norm = normalize_unit(source_unit);
    let tgt_norm = normalize_unit(target_unit);

    // Handle temperature separately
    if is_temperature(&src_norm) || is_temperature(&tgt_norm) {
        return Some(convert_temperature(value, &src_norm, &tgt_norm));
    }

    let source_info = conversions.get(src_norm.as_str())?;
    let target_info = match conversions.get(tgt_norm.as_str()) {
        Some(info) => info,
        None => return Some(Err(format!("Unknown unit: {}", target_unit))),
    };

    if source_info.category != target_info.category {
        return Some(Err(format!(
            "Cannot convert {} to {} (incompatible types)",
            source_unit, target_unit
        )));
    }

    let base_value = value * source_info.to_base;
    let result = base_value / target_info.to_base;

    Some(Ok((result, display_unit(target_unit))))
}

/// Check if a string is a known unit
pub fn is_known_unit(s: &str) -> bool {
    let norm = normalize_unit(s);
    get_conversions().contains_key(norm.as_str()) || is_temperature(&norm)
}

#[allow(dead_code)]
/// Legacy interface for backward compatibility
pub fn try_convert(expr: &str) -> Option<Result<f64, String>> {
    let parts: Vec<&str> = expr.splitn(2, " in ").collect();
    let (left, target_unit) = if parts.len() == 2 {
        (parts[0].trim(), parts[1].trim())
    } else {
        let parts: Vec<&str> = expr.splitn(2, " to ").collect();
        if parts.len() == 2 {
            (parts[0].trim(), parts[1].trim())
        } else {
            return None;
        }
    };

    match try_convert_parts(left, target_unit) {
        Some(Ok((val, _))) => Some(Ok(val)),
        Some(Err(e)) => Some(Err(e)),
        None => None,
    }
}

fn parse_value_unit(s: &str) -> Option<(f64, &str)> {
    let s = s.trim();
    // Handle expressions in parens: "(25 cm * 6 + 5%)" -- for now just simple values
    let mut i = 0;
    let chars: Vec<char> = s.chars().collect();

    if i < chars.len() && chars[i] == '-' {
        i += 1;
    }

    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_') {
        i += 1;
    }

    if i == 0 {
        return None;
    }

    let num_str: String = chars[..i].iter().filter(|c| **c != '_').collect();
    let value: f64 = num_str.parse().ok()?;
    let rest = s[i..].trim();

    if rest.is_empty() {
        return None;
    }

    Some((value, rest))
}

fn normalize_unit(u: &str) -> String {
    u.to_lowercase().trim().to_string()
}

fn display_unit(u: &str) -> String {
    let norm = normalize_unit(u);
    // Return canonical short form
    match norm.as_str() {
        "teaspoon" | "teaspoons" | "tea spoons" | "tea spoon" | "tsp" => "tsp".to_string(),
        "tablespoon" | "tablespoons" | "tbsp" => "tbsp".to_string(),
        "cup" | "cups" => "cup".to_string(),
        "milliliter" | "milliliters" | "ml" => "ml".to_string(),
        "liter" | "liters" | "l" | "litro" | "litros" => "L".to_string(),
        "gallon" | "gallons" | "gal" => "gal".to_string(),
        "kilometer" | "kilometers" | "km" => "km".to_string(),
        "meter" | "meters" => "m".to_string(),
        "centimeter" | "centimeters" | "cm" => "cm".to_string(),
        "millimeter" | "millimeters" | "mm" => "mm".to_string(),
        "mile" | "miles" | "mi" => "mi".to_string(),
        "foot" | "feet" | "ft" => "ft".to_string(),
        "inch" | "inches" => "in".to_string(),
        "yard" | "yards" | "yd" => "yd".to_string(),
        "kilogram" | "kilograms" | "kg" | "quilo" | "quilos" => "kg".to_string(),
        "gram" | "grams" | "g" | "grama" | "gramas" => "g".to_string(),
        "pound" | "pounds" | "lb" | "lbs" | "libra" | "libras" => "lb".to_string(),
        "ounce" | "ounces" | "oz" => "oz".to_string(),
        "celsius" | "°c" | "c" => "°C".to_string(),
        "fahrenheit" | "°f" | "f" => "°F".to_string(),
        "kelvin" | "k" => "K".to_string(),
        _ => u.to_string(),
    }
}

// Temperature conversion (non-linear)
fn is_temperature(unit: &str) -> bool {
    matches!(unit, "celsius" | "°c" | "fahrenheit" | "°f" | "kelvin")
}

fn convert_temperature(value: f64, from: &str, to: &str) -> Result<(f64, String), String> {
    let celsius = match from {
        "celsius" | "°c" => value,
        "fahrenheit" | "°f" => (value - 32.0) * 5.0 / 9.0,
        "kelvin" => value - 273.15,
        _ => return Err(format!("Unknown temperature unit: {}", from)),
    };

    let result = match to {
        "celsius" | "°c" => celsius,
        "fahrenheit" | "°f" => celsius * 9.0 / 5.0 + 32.0,
        "kelvin" => celsius + 273.15,
        _ => return Err(format!("Unknown temperature unit: {}", to)),
    };

    Ok((result, display_unit(to)))
}

#[derive(Clone)]
struct UnitInfo {
    to_base: f64,
    category: &'static str,
}

fn get_conversions() -> HashMap<&'static str, UnitInfo> {
    let mut m = HashMap::new();

    // Length (base: meters)
    let length = "length";
    m.insert("mm", UnitInfo { to_base: 0.001, category: length });
    m.insert("millimeter", UnitInfo { to_base: 0.001, category: length });
    m.insert("millimeters", UnitInfo { to_base: 0.001, category: length });
    m.insert("milímetro", UnitInfo { to_base: 0.001, category: length });
    m.insert("milímetros", UnitInfo { to_base: 0.001, category: length });
    m.insert("cm", UnitInfo { to_base: 0.01, category: length });
    m.insert("centimeter", UnitInfo { to_base: 0.01, category: length });
    m.insert("centimeters", UnitInfo { to_base: 0.01, category: length });
    m.insert("centímetro", UnitInfo { to_base: 0.01, category: length });
    m.insert("centímetros", UnitInfo { to_base: 0.01, category: length });
    m.insert("m", UnitInfo { to_base: 1.0, category: length });
    m.insert("meter", UnitInfo { to_base: 1.0, category: length });
    m.insert("meters", UnitInfo { to_base: 1.0, category: length });
    m.insert("metro", UnitInfo { to_base: 1.0, category: length });
    m.insert("metros", UnitInfo { to_base: 1.0, category: length });
    m.insert("km", UnitInfo { to_base: 1000.0, category: length });
    m.insert("kilometer", UnitInfo { to_base: 1000.0, category: length });
    m.insert("kilometers", UnitInfo { to_base: 1000.0, category: length });
    m.insert("quilômetro", UnitInfo { to_base: 1000.0, category: length });
    m.insert("quilômetros", UnitInfo { to_base: 1000.0, category: length });
    m.insert("in", UnitInfo { to_base: 0.0254, category: length });
    m.insert("inch", UnitInfo { to_base: 0.0254, category: length });
    m.insert("inches", UnitInfo { to_base: 0.0254, category: length });
    m.insert("polegada", UnitInfo { to_base: 0.0254, category: length });
    m.insert("polegadas", UnitInfo { to_base: 0.0254, category: length });
    m.insert("ft", UnitInfo { to_base: 0.3048, category: length });
    m.insert("foot", UnitInfo { to_base: 0.3048, category: length });
    m.insert("feet", UnitInfo { to_base: 0.3048, category: length });
    m.insert("pé", UnitInfo { to_base: 0.3048, category: length });
    m.insert("pés", UnitInfo { to_base: 0.3048, category: length });
    m.insert("yd", UnitInfo { to_base: 0.9144, category: length });
    m.insert("yard", UnitInfo { to_base: 0.9144, category: length });
    m.insert("yards", UnitInfo { to_base: 0.9144, category: length });
    m.insert("mi", UnitInfo { to_base: 1609.344, category: length });
    m.insert("mile", UnitInfo { to_base: 1609.344, category: length });
    m.insert("miles", UnitInfo { to_base: 1609.344, category: length });
    m.insert("milha", UnitInfo { to_base: 1609.344, category: length });
    m.insert("milhas", UnitInfo { to_base: 1609.344, category: length });

    // Weight (base: grams)
    let weight = "weight";
    m.insert("mg", UnitInfo { to_base: 0.001, category: weight });
    m.insert("g", UnitInfo { to_base: 1.0, category: weight });
    m.insert("gram", UnitInfo { to_base: 1.0, category: weight });
    m.insert("grams", UnitInfo { to_base: 1.0, category: weight });
    m.insert("grama", UnitInfo { to_base: 1.0, category: weight });
    m.insert("gramas", UnitInfo { to_base: 1.0, category: weight });
    m.insert("kg", UnitInfo { to_base: 1000.0, category: weight });
    m.insert("kilogram", UnitInfo { to_base: 1000.0, category: weight });
    m.insert("kilograms", UnitInfo { to_base: 1000.0, category: weight });
    m.insert("quilo", UnitInfo { to_base: 1000.0, category: weight });
    m.insert("quilos", UnitInfo { to_base: 1000.0, category: weight });
    m.insert("lb", UnitInfo { to_base: 453.592, category: weight });
    m.insert("lbs", UnitInfo { to_base: 453.592, category: weight });
    m.insert("pound", UnitInfo { to_base: 453.592, category: weight });
    m.insert("pounds", UnitInfo { to_base: 453.592, category: weight });
    m.insert("libra", UnitInfo { to_base: 453.592, category: weight });
    m.insert("libras", UnitInfo { to_base: 453.592, category: weight });
    m.insert("oz", UnitInfo { to_base: 28.3495, category: weight });
    m.insert("ounce", UnitInfo { to_base: 28.3495, category: weight });
    m.insert("ounces", UnitInfo { to_base: 28.3495, category: weight });
    m.insert("ton", UnitInfo { to_base: 1_000_000.0, category: weight });
    m.insert("tons", UnitInfo { to_base: 1_000_000.0, category: weight });
    m.insert("tonelada", UnitInfo { to_base: 1_000_000.0, category: weight });
    m.insert("toneladas", UnitInfo { to_base: 1_000_000.0, category: weight });

    // Volume (base: milliliters)
    let volume = "volume";
    m.insert("ml", UnitInfo { to_base: 1.0, category: volume });
    m.insert("milliliter", UnitInfo { to_base: 1.0, category: volume });
    m.insert("milliliters", UnitInfo { to_base: 1.0, category: volume });
    m.insert("mililitro", UnitInfo { to_base: 1.0, category: volume });
    m.insert("mililitros", UnitInfo { to_base: 1.0, category: volume });
    m.insert("l", UnitInfo { to_base: 1000.0, category: volume });
    m.insert("liter", UnitInfo { to_base: 1000.0, category: volume });
    m.insert("liters", UnitInfo { to_base: 1000.0, category: volume });
    m.insert("litro", UnitInfo { to_base: 1000.0, category: volume });
    m.insert("litros", UnitInfo { to_base: 1000.0, category: volume });
    m.insert("gal", UnitInfo { to_base: 3785.41, category: volume });
    m.insert("gallon", UnitInfo { to_base: 3785.41, category: volume });
    m.insert("gallons", UnitInfo { to_base: 3785.41, category: volume });
    m.insert("galão", UnitInfo { to_base: 3785.41, category: volume });
    m.insert("galões", UnitInfo { to_base: 3785.41, category: volume });
    // Cooking
    m.insert("tsp", UnitInfo { to_base: 4.929, category: volume });
    m.insert("teaspoon", UnitInfo { to_base: 4.929, category: volume });
    m.insert("teaspoons", UnitInfo { to_base: 4.929, category: volume });
    m.insert("tea spoon", UnitInfo { to_base: 4.929, category: volume });
    m.insert("tea spoons", UnitInfo { to_base: 4.929, category: volume });
    m.insert("colher de chá", UnitInfo { to_base: 4.929, category: volume });
    m.insert("colheres de chá", UnitInfo { to_base: 4.929, category: volume });
    m.insert("tbsp", UnitInfo { to_base: 14.787, category: volume });
    m.insert("tablespoon", UnitInfo { to_base: 14.787, category: volume });
    m.insert("tablespoons", UnitInfo { to_base: 14.787, category: volume });
    m.insert("colher de sopa", UnitInfo { to_base: 14.787, category: volume });
    m.insert("colheres de sopa", UnitInfo { to_base: 14.787, category: volume });
    m.insert("cup", UnitInfo { to_base: 236.588, category: volume });
    m.insert("cups", UnitInfo { to_base: 236.588, category: volume });
    m.insert("xícara", UnitInfo { to_base: 236.588, category: volume });
    m.insert("xícaras", UnitInfo { to_base: 236.588, category: volume });
    m.insert("fl oz", UnitInfo { to_base: 29.574, category: volume });
    m.insert("fluid ounce", UnitInfo { to_base: 29.574, category: volume });
    m.insert("fluid ounces", UnitInfo { to_base: 29.574, category: volume });

    // Time (base: seconds)
    let time = "time";
    m.insert("ms", UnitInfo { to_base: 0.001, category: time });
    m.insert("millisecond", UnitInfo { to_base: 0.001, category: time });
    m.insert("milliseconds", UnitInfo { to_base: 0.001, category: time });
    m.insert("s", UnitInfo { to_base: 1.0, category: time });
    m.insert("sec", UnitInfo { to_base: 1.0, category: time });
    m.insert("second", UnitInfo { to_base: 1.0, category: time });
    m.insert("seconds", UnitInfo { to_base: 1.0, category: time });
    m.insert("segundo", UnitInfo { to_base: 1.0, category: time });
    m.insert("segundos", UnitInfo { to_base: 1.0, category: time });
    m.insert("min", UnitInfo { to_base: 60.0, category: time });
    m.insert("minute", UnitInfo { to_base: 60.0, category: time });
    m.insert("minutes", UnitInfo { to_base: 60.0, category: time });
    m.insert("minuto", UnitInfo { to_base: 60.0, category: time });
    m.insert("minutos", UnitInfo { to_base: 60.0, category: time });
    m.insert("h", UnitInfo { to_base: 3600.0, category: time });
    m.insert("hr", UnitInfo { to_base: 3600.0, category: time });
    m.insert("hour", UnitInfo { to_base: 3600.0, category: time });
    m.insert("hours", UnitInfo { to_base: 3600.0, category: time });
    m.insert("hora", UnitInfo { to_base: 3600.0, category: time });
    m.insert("horas", UnitInfo { to_base: 3600.0, category: time });
    m.insert("day", UnitInfo { to_base: 86400.0, category: time });
    m.insert("days", UnitInfo { to_base: 86400.0, category: time });
    m.insert("dia", UnitInfo { to_base: 86400.0, category: time });
    m.insert("dias", UnitInfo { to_base: 86400.0, category: time });
    m.insert("week", UnitInfo { to_base: 604800.0, category: time });
    m.insert("weeks", UnitInfo { to_base: 604800.0, category: time });
    m.insert("semana", UnitInfo { to_base: 604800.0, category: time });
    m.insert("semanas", UnitInfo { to_base: 604800.0, category: time });

    // Data (base: bytes)
    let data = "data";
    m.insert("b", UnitInfo { to_base: 1.0, category: data });
    m.insert("byte", UnitInfo { to_base: 1.0, category: data });
    m.insert("bytes", UnitInfo { to_base: 1.0, category: data });
    m.insert("kb", UnitInfo { to_base: 1024.0, category: data });
    m.insert("kilobyte", UnitInfo { to_base: 1024.0, category: data });
    m.insert("kilobytes", UnitInfo { to_base: 1024.0, category: data });
    m.insert("mb", UnitInfo { to_base: 1_048_576.0, category: data });
    m.insert("megabyte", UnitInfo { to_base: 1_048_576.0, category: data });
    m.insert("megabytes", UnitInfo { to_base: 1_048_576.0, category: data });
    m.insert("gb", UnitInfo { to_base: 1_073_741_824.0, category: data });
    m.insert("gigabyte", UnitInfo { to_base: 1_073_741_824.0, category: data });
    m.insert("gigabytes", UnitInfo { to_base: 1_073_741_824.0, category: data });
    m.insert("tb", UnitInfo { to_base: 1_099_511_627_776.0, category: data });
    m.insert("terabyte", UnitInfo { to_base: 1_099_511_627_776.0, category: data });
    m.insert("terabytes", UnitInfo { to_base: 1_099_511_627_776.0, category: data });

    m
}
