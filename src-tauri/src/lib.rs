mod engine;

use engine::{Engine, LineResult, ResultType, currency};
use engine::i18n::Locale;
use tauri::command;
use std::sync::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};

struct TabState {
    engine: Engine,
}

struct AppState {
    tabs: Mutex<HashMap<String, TabState>>,
    currency_rates: Mutex<HashMap<String, f64>>,
}

impl AppState {
    fn ensure_tab(&self, tab_id: &str) {
        let mut tabs = self.tabs.lock().unwrap();
        if !tabs.contains_key(tab_id) {
            let mut engine = Engine::new();
            let rates = self.currency_rates.lock().unwrap();
            if !rates.is_empty() {
                engine.set_currency_rates(rates.clone());
            }
            tabs.insert(tab_id.to_string(), TabState {
                engine,
            });
        }
    }
}

#[derive(Serialize)]
struct EvalResponse {
    results: Vec<LineResult>,
    total: Option<f64>,
    total_formatted: String,
    locale: String,
}

#[command]
fn evaluate(state: tauri::State<AppState>, tab_id: String, text: String, ui_locale: String) -> EvalResponse {
    state.ensure_tab(&tab_id);
    let mut tabs = state.tabs.lock().unwrap();
    let tab = tabs.get_mut(&tab_id).unwrap();

    // Always use PtBr keywords for parsing (superset that accepts both languages)
    tab.engine.set_locale(Locale::PtBr);
    let mut results = tab.engine.evaluate_document(&text, false);

    // Determine display locale for error messages
    let display_locale = match ui_locale.as_str() {
        "pt-br" | "pt" => Locale::PtBr,
        _ => Locale::En,
    };

    // If UI is English, revert translated errors back to English
    if display_locale == Locale::En {
        for r in results.iter_mut() {
            if r.result_type == ResultType::Error && !r.output.is_empty() {
                // Errors were translated to PT-BR by the engine, revert to original
                r.output = revert_error_to_en(&r.output);
            }
        }
    }
    let total = tab.engine.get_sum();
    let total_formatted = if total != 0.0 {
        format_total(total)
    } else {
        String::new()
    };

    let locale_str = match display_locale {
        Locale::En => "en",
        Locale::PtBr => "pt-br",
    };

    EvalResponse {
        results,
        total: if total != 0.0 { Some(total) } else { None },
        total_formatted,
        locale: locale_str.to_string(),
    }
}

#[command]
fn update_rates(state: tauri::State<AppState>, rates: HashMap<String, f64>) {
    // Update all existing tabs
    let mut tabs = state.tabs.lock().unwrap();
    for tab in tabs.values_mut() {
        tab.engine.set_currency_rates(rates.clone());
    }
    // Store rates for future tabs
    let mut stored_rates = state.currency_rates.lock().unwrap();
    *stored_rates = rates;
}

#[command]
fn close_tab(state: tauri::State<AppState>, tab_id: String) {
    let mut tabs = state.tabs.lock().unwrap();
    tabs.remove(&tab_id);
}

#[command]
fn export_file(default_name: String, content: String) -> Result<(), String> {
    use std::process::Command;

    // Use zenity (common on Linux) for native file dialog
    let output = Command::new("zenity")
        .args(["--file-selection", "--save", "--confirm-overwrite", "--filename", &default_name])
        .output()
        .map_err(|e| format!("Failed to open dialog: {}", e))?;

    if !output.status.success() {
        return Err("Cancelled".to_string());
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return Err("No path selected".to_string());
    }

    fs::write(&path, &content).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

// --- Persistence ---

#[derive(Serialize, Deserialize, Clone)]
struct SavedTab {
    id: String,
    title: String,
    text: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct SavedState {
    tabs: Vec<SavedTab>,
    active_tab_id: String,
    ui_locale: String,
    tab_counter: u32,
    #[serde(default)]
    theme: Option<String>,
}

fn state_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".evline")
}

fn state_file() -> PathBuf {
    state_dir().join("state.json")
}

#[command]
fn save_state(data: SavedState) -> Result<(), String> {
    let dir = state_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create dir: {}", e))?;
    let json = serde_json::to_string_pretty(&data).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(state_file(), json).map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

#[command]
fn load_state() -> Option<SavedState> {
    let path = state_file();
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

// --- Error translation ---

fn revert_error_to_en(err: &str) -> String {
    if err.starts_with("Variável desconhecida: ") {
        let var = &err["Variável desconhecida: ".len()..];
        return format!("Unknown variable: {}", var);
    }
    if err.contains("Número inválido") {
        return err.replace("Número inválido", "Invalid number");
    }
    if err == "Sem valor anterior" {
        return "No previous value".to_string();
    }
    if err == "Porcentagem inválida" {
        return "Invalid percentage".to_string();
    }
    if err == "Divisão por zero" {
        return "Division by zero".to_string();
    }
    if err == "Expressão inválida" {
        return "Invalid expression".to_string();
    }
    if err == "Expressão vazia" {
        return "Empty expression".to_string();
    }
    if err == "Parênteses não correspondem" {
        return "Mismatched parentheses".to_string();
    }
    if err.starts_with("Função desconhecida: ") {
        let f = &err["Função desconhecida: ".len()..];
        return format!("Unknown function: {}", f);
    }
    if err.starts_with("Caractere inesperado: ") {
        let c = &err["Caractere inesperado: ".len()..];
        return format!("Unexpected character: {}", c);
    }
    if err.contains("Não é possível converter") {
        return err.replace("Não é possível converter", "Cannot convert");
    }
    err.to_string()
}

fn format_total(val: f64) -> String {
    if val == val.floor() && val.abs() < 1e15 {
        let s = (val as i64).to_string();
        add_thousands_separator(&s)
    } else {
        format!("{:.2}", val)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            tabs: Mutex::new(HashMap::new()),
            currency_rates: Mutex::new(currency::fallback_rates()),
        })
        .invoke_handler(tauri::generate_handler![evaluate, update_rates, close_tab, export_file, save_state, load_state])
        .run(tauri::generate_context!())
        .expect("error while running Evline");
}
