use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Locale {
    En,
    PtBr,
}

#[allow(dead_code)]
/// Keywords that vary by locale
pub struct Keywords {
    // Conversion keywords: "in", "to"
    pub conversion: &'static [&'static str],
    // "of what is" (inverse percentage)
    pub of_what_is: &'static [&'static str],
    // "of" (percentage of)
    pub percent_of: &'static [&'static str],
    // "today"
    pub today: &'static [&'static str],
    // Time units
    pub days: &'static [&'static str],
    pub weeks: &'static [&'static str],
    pub months: &'static [&'static str],
    pub years: &'static [&'static str],
    pub hours: &'static [&'static str],
    pub minutes: &'static [&'static str],
    pub seconds: &'static [&'static str],
    // Sum/total
    pub sum: &'static [&'static str],
}

pub fn get_keywords(locale: Locale) -> Keywords {
    match locale {
        Locale::En => Keywords {
            conversion: &["in", "to"],
            of_what_is: &["of what is"],
            percent_of: &["of"],
            today: &["today", "now"],
            days: &["days", "day"],
            weeks: &["weeks", "week"],
            months: &["months", "month"],
            years: &["years", "year"],
            hours: &["hours", "hour", "hrs", "hr"],
            minutes: &["minutes", "minute", "mins", "min"],
            seconds: &["seconds", "second", "secs", "sec"],
            sum: &["sum", "total", "subtotal"],
        },
        Locale::PtBr => Keywords {
            conversion: &["em", "para", "in", "to"],
            of_what_is: &["de quanto é", "de quanto e", "of what is"],
            percent_of: &["de", "of"],
            today: &["hoje", "agora", "today", "now"],
            days: &["dias", "dia", "days", "day"],
            weeks: &["semanas", "semana", "weeks", "week"],
            months: &["meses", "mês", "mes", "months", "month"],
            years: &["anos", "ano", "years", "year"],
            hours: &["horas", "hora", "hours", "hour", "hrs", "hr"],
            minutes: &["minutos", "minuto", "minutes", "minute", "mins", "min"],
            seconds: &["segundos", "segundo", "seconds", "second", "secs", "sec"],
            sum: &["soma", "total", "subtotal", "sum"],
        },
    }
}

/// Detect locale from text heuristic
pub fn detect_locale(text: &str) -> Locale {
    let lower = text.to_lowercase();
    let pt_indicators = [
        " em ", " para ", " hoje", " dias", " dia ", " semana", " mês", " mes ",
        " ano ", " anos ", " horas", " hora ", " minuto", " segundo", " soma",
        "de quanto", " de ",
    ];
    let en_indicators = [
        " in ", " to ", " today", " days", " day ", " week", " month",
        " year ", " years ", " hours", " hour ", " minute", " second", " sum",
        "of what is",
    ];

    let pt_score: usize = pt_indicators.iter().filter(|k| lower.contains(*k)).count();
    let en_score: usize = en_indicators.iter().filter(|k| lower.contains(*k)).count();

    if pt_score > en_score {
        Locale::PtBr
    } else {
        Locale::En
    }
}
