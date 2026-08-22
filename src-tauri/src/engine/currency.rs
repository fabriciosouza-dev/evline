#![allow(dead_code)]
use std::collections::HashMap;
use serde::Deserialize;

/// Fetch currency rates from a free API
/// Returns rates relative to USD
pub async fn fetch_rates() -> Result<HashMap<String, f64>, String> {
    // Use the free exchangerate API (no key required for basic use)
    let url = "https://open.er-api.com/v6/latest/USD";

    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let data: ApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    if data.result != "success" {
        return Err("API returned error".to_string());
    }

    Ok(data.rates)
}

/// Fallback rates in case API is unavailable
pub fn fallback_rates() -> HashMap<String, f64> {
    let mut rates = HashMap::new();
    rates.insert("EUR".to_string(), 0.92);
    rates.insert("GBP".to_string(), 0.79);
    rates.insert("BRL".to_string(), 5.45);
    rates.insert("JPY".to_string(), 149.50);
    rates.insert("CNY".to_string(), 7.24);
    rates.insert("CAD".to_string(), 1.36);
    rates.insert("AUD".to_string(), 1.53);
    rates.insert("CHF".to_string(), 0.88);
    rates.insert("ARS".to_string(), 875.0);
    rates.insert("CLP".to_string(), 935.0);
    rates.insert("COP".to_string(), 3950.0);
    rates.insert("MXN".to_string(), 17.15);
    rates.insert("PEN".to_string(), 3.72);
    rates.insert("UYU".to_string(), 38.5);
    rates.insert("INR".to_string(), 83.1);
    rates.insert("KRW".to_string(), 1320.0);
    rates.insert("SGD".to_string(), 1.34);
    rates.insert("HKD".to_string(), 7.82);
    rates.insert("NZD".to_string(), 1.63);
    rates.insert("SEK".to_string(), 10.45);
    rates.insert("NOK".to_string(), 10.55);
    rates.insert("DKK".to_string(), 6.87);
    rates.insert("PLN".to_string(), 3.98);
    rates.insert("CZK".to_string(), 22.5);
    rates.insert("HUF".to_string(), 355.0);
    rates.insert("TRY".to_string(), 30.2);
    rates.insert("ZAR".to_string(), 18.6);
    rates.insert("RUB".to_string(), 92.0);
    rates.insert("THB".to_string(), 34.8);
    rates.insert("TWD".to_string(), 31.5);
    rates
}

#[derive(Deserialize)]
struct ApiResponse {
    result: String,
    rates: HashMap<String, f64>,
}
