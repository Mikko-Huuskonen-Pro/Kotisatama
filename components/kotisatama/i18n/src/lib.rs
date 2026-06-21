/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kotisatama desktop UI strings (Finnish default, Swedish supported).
//!
//! Locale detection mirrors `kotisatama-i18n.js`: `KOTISATAMA_LOCALE` override,
//! then `LANG`, then Finnish.

use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Fi,
    Sv,
}

static ACTIVE_LOCALE: OnceLock<Locale> = OnceLock::new();

/// Detect locale once per process (same priority as HTML i18n).
pub fn detect_locale() -> Locale {
    if let Ok(value) = std::env::var("KOTISATAMA_LOCALE") {
        if let Some(locale) = parse_locale_tag(&value) {
            return locale;
        }
    }

    if let Ok(lang) = std::env::var("LANG") {
        if let Some(locale) = parse_locale_tag(&lang) {
            return locale;
        }
    }

    Locale::Fi
}

fn parse_locale_tag(value: &str) -> Option<Locale> {
    let code = value.split(['_', '.', '-']).next()?.to_ascii_lowercase();
    match code.as_str() {
        "sv" => Some(Locale::Sv),
        "fi" => Some(Locale::Fi),
        _ => None,
    }
}

/// Active locale for this process.
pub fn locale() -> Locale {
    *ACTIVE_LOCALE.get_or_init(detect_locale)
}

/// Translate a desktop UI string key for the active locale.
pub fn t(key: &str) -> &str {
    t_for(locale(), key)
}

/// Translate for a specific locale (tests and callers that cache locale).
pub fn t_for(locale: Locale, key: &str) -> &str {
    match (locale, key) {
        (Locale::Fi, "report_button") => "Ilmoita",
        (Locale::Sv, "report_button") => "Anmäl",

        (Locale::Fi, "report_button_a11y") => "Ilmoita ongelmasta",
        (Locale::Sv, "report_button_a11y") => "Anmäl om ett problem",

        (Locale::Fi, "search_label") => "Hae:",
        (Locale::Sv, "search_label") => "Sök:",

        (Locale::Fi, "search_hint") => "Hae kotisatamasta…",
        (Locale::Sv, "search_hint") => "Sök i hemmahamnen…",

        (Locale::Fi, "pulloposti_button") => "Pulloposti",
        (Locale::Sv, "pulloposti_button") => "Flaskpost",

        (Locale::Fi, "search_window_title") => "Kotisatama-haku",
        (Locale::Sv, "search_window_title") => "Kotisatama-sökning",

        (Locale::Fi, "search_loading") => "Haetaan…",
        (Locale::Sv, "search_loading") => "Söker…",

        (Locale::Fi, "search_query_prefix") => "Haku:",
        (Locale::Sv, "search_query_prefix") => "Sök:",

        (Locale::Fi, "search_no_results") => {
            "Ei löydy kotisatamasta — haluatko hakea avomereltä?"
        },
        (Locale::Sv, "search_no_results") => {
            "Finns inte i hemmahamnen — vill du söka på öppet hav?"
        },

        (Locale::Fi, "search_avomeri") => "Hae avomereltä",
        (Locale::Sv, "search_avomeri") => "Sök på öppet hav",

        (Locale::Fi, "close") => "Sulje",
        (Locale::Sv, "close") => "Stäng",

        (Locale::Fi, "report_window_title") => "Ilmoita",
        (Locale::Sv, "report_window_title") => "Anmäl",

        (Locale::Fi, "report_intro") => {
            "Lähetä anonyymi raportti (ei käyttäjätunnistetta)."
        },
        (Locale::Sv, "report_intro") => {
            "Skicka en anonym rapport (ingen användaridentifiering)."
        },

        (Locale::Fi, "report_site_broken") => "Sivusto ei toimi",
        (Locale::Sv, "report_site_broken") => "Webbplatsen fungerar inte",

        (Locale::Fi, "report_suggest_site") => "Ehdota kotisatamaan",
        (Locale::Sv, "report_suggest_site") => "Föreslå till hemmahamnen",

        (Locale::Fi, "report_domain") => "Verkkotunnus:",
        (Locale::Sv, "report_domain") => "Domän:",

        (Locale::Fi, "report_description") => "Kuvaus (valinnainen):",
        (Locale::Sv, "report_description") => "Beskrivning (valfritt):",

        (Locale::Fi, "report_sent") => "Raportti lähetetty.",
        (Locale::Sv, "report_sent") => "Rapporten har skickats.",

        (Locale::Fi, "report_submit") => "Lähetä",
        (Locale::Sv, "report_submit") => "Skicka",

        (Locale::Fi, "report_submitting") => "Lähetetään…",
        (Locale::Sv, "report_submitting") => "Skickar…",

        _ => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swedish_report_button() {
        assert_eq!(t_for(Locale::Sv, "report_button"), "Anmäl");
    }

    #[test]
    fn finnish_defaults_when_key_unknown() {
        assert_eq!(t_for(Locale::Fi, "missing_key"), "missing_key");
    }

    #[test]
    fn parse_locale_tags() {
        assert_eq!(parse_locale_tag("sv_SE.UTF-8"), Some(Locale::Sv));
        assert_eq!(parse_locale_tag("fi"), Some(Locale::Fi));
        assert_eq!(parse_locale_tag("en_US"), None);
    }
}
