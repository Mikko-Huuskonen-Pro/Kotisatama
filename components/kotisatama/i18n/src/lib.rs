/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kotisatama desktop UI strings (Finnish default, Swedish supported).
//!
//! Locale selection mirrors `kotisatama-i18n.js`: `KOTISATAMA_LOCALE` override,
//! then saved choice (`auto` / `fi` / `sv`) in the user config dir, then `LANG`,
//! then Finnish.

use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Fi,
    Sv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleChoice {
    Auto,
    Fi,
    Sv,
}

static RESOLVED_LOCALE: RwLock<Option<Locale>> = RwLock::new(None);

/// Path to the persisted locale choice file.
pub fn locale_config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("Kotisatama").join("locale"))
}

/// Parse a stored locale choice tag.
pub fn parse_locale_choice(value: &str) -> Option<LocaleChoice> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(LocaleChoice::Auto),
        "fi" => Some(LocaleChoice::Fi),
        "sv" => Some(LocaleChoice::Sv),
        _ => None,
    }
}

/// Load the saved locale choice, if any.
pub fn load_locale_choice() -> Option<LocaleChoice> {
    let path = locale_config_path()?;
    let contents = fs::read_to_string(path).ok()?;
    parse_locale_choice(&contents)
}

/// Persist locale choice for the next servoshell launch.
pub fn save_locale_choice(choice: LocaleChoice) -> std::io::Result<()> {
    let path = locale_config_path()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no config dir"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let value = match choice {
        LocaleChoice::Auto => "auto",
        LocaleChoice::Fi => "fi",
        LocaleChoice::Sv => "sv",
    };
    fs::write(path, value)
}

/// Save choice and refresh the in-process locale cache.
pub fn set_locale_choice(choice: LocaleChoice) -> std::io::Result<()> {
    save_locale_choice(choice)?;
    if let Ok(mut guard) = RESOLVED_LOCALE.write() {
        *guard = Some(resolve_locale(choice));
    }
    Ok(())
}

fn parse_locale_tag(value: &str) -> Option<Locale> {
    let code = value.split(['_', '.', '-']).next()?.to_ascii_lowercase();
    match code.as_str() {
        "sv" => Some(Locale::Sv),
        "fi" => Some(Locale::Fi),
        _ => None,
    }
}

fn system_locale() -> Option<Locale> {
    std::env::var("LANG")
        .ok()
        .and_then(|lang| parse_locale_tag(&lang))
}

fn resolve_locale(choice: LocaleChoice) -> Locale {
    match choice {
        LocaleChoice::Fi => Locale::Fi,
        LocaleChoice::Sv => Locale::Sv,
        LocaleChoice::Auto => system_locale().unwrap_or(Locale::Fi),
    }
}

/// Detect locale choice and resolved language for first use.
pub fn detect_locale() -> Locale {
    if let Ok(value) = std::env::var("KOTISATAMA_LOCALE") {
        if let Some(locale) = parse_locale_tag(&value) {
            return locale;
        }
    }

    if let Some(choice) = load_locale_choice() {
        return resolve_locale(choice);
    }

    system_locale().unwrap_or(Locale::Fi)
}

/// Active locale for this process.
pub fn locale() -> Locale {
    if let Ok(guard) = RESOLVED_LOCALE.read() {
        if let Some(locale) = *guard {
            return locale;
        }
    }

    let locale = detect_locale();
    if let Ok(mut guard) = RESOLVED_LOCALE.write() {
        *guard = Some(locale);
    }
    locale
}

/// Translate a desktop UI string key for the active locale.
pub fn t(key: &str) -> &str {
    t_for(locale(), key)
}

/// Translate for a specific locale (tests and callers that cache locale).
pub fn t_for(locale: Locale, key: &str) -> &str {
    match (locale, key) {
        (Locale::Fi, "report_button") => "Lokikirja",
        (Locale::Sv, "report_button") => "Loggbok",

        (Locale::Fi, "report_button_a11y") => "Lokikirja — ilmoita ongelmasta",
        (Locale::Sv, "report_button_a11y") => "Loggbok — anmäl om ett problem",

        (Locale::Fi, "search_label") => "Hae:",
        (Locale::Sv, "search_label") => "Sök:",

        (Locale::Fi, "search_hint") => "Hae satamasta…",
        (Locale::Sv, "search_hint") => "Sök i hamnen…",

        (Locale::Fi, "pulloposti_button") => "Pulloposti",
        (Locale::Sv, "pulloposti_button") => "Flaskpost",

        (Locale::Fi, "varustamo_button") => "Varustamo",
        (Locale::Sv, "varustamo_button") => "Varustamo",

        (Locale::Fi, "search_button") => "🔍",
        (Locale::Sv, "search_button") => "🔍",

        (Locale::Fi, "search_button_a11y") => "Avaa hakutulokset",
        (Locale::Sv, "search_button_a11y") => "Öppna sökresultat",

        (Locale::Fi, "search_window_title") => "Satama-haku",
        (Locale::Sv, "search_window_title") => "Hamn-sökning",

        (Locale::Fi, "search_loading") => "Haetaan…",
        (Locale::Sv, "search_loading") => "Söker…",

        (Locale::Fi, "search_query_prefix") => "Haku:",
        (Locale::Sv, "search_query_prefix") => "Sök:",

        (Locale::Fi, "search_no_results") => {
            "Ei löydy satamasta. Voit kirjata tarpeen tai ehdottaa sivustoa Satamaan."
        },
        (Locale::Sv, "search_no_results") => {
            "Finns inte i hamnen. Du kan anteckna behovet eller föreslå webbplatsen till hamnen."
        },

        (Locale::Fi, "search_avomeri") => "Avaa Avomeri-portti",
        (Locale::Sv, "search_avomeri") => "Öppna porten till öppet hav",

        (Locale::Fi, "close") => "Sulje",
        (Locale::Sv, "close") => "Stäng",

        (Locale::Fi, "report_window_title") => "Lokikirja",
        (Locale::Sv, "report_window_title") => "Loggbok",

        (Locale::Fi, "report_intro") => {
            "Lähetä anonyymi ilmoitus GitHub-issueen (Katselin.fi-repo). Ei käyttäjätunnistetta."
        },
        (Locale::Sv, "report_intro") => {
            "Skicka en anonym rapport som GitHub-issue (Katselin.fi-repot). Ingen inloggning."
        },

        (Locale::Fi, "report_site_broken") => "Sivusto ei toimi",
        (Locale::Sv, "report_site_broken") => "Webbplatsen fungerar inte",

        (Locale::Fi, "report_suggest_site") => "Ehdota satamaan",
        (Locale::Sv, "report_suggest_site") => "Föreslå till hamnen",

        (Locale::Fi, "report_domain") => "Verkkotunnus:",
        (Locale::Sv, "report_domain") => "Domän:",

        (Locale::Fi, "report_description") => "Kuvaus (valinnainen):",
        (Locale::Sv, "report_description") => "Beskrivning (valfritt):",

        (Locale::Fi, "report_sent") => "Ilmoitus tallennettu (GitHub-issue).",
        (Locale::Sv, "report_sent") => "Rapporten sparades (GitHub-issue).",

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
        assert_eq!(t_for(Locale::Sv, "report_button"), "Loggbok");
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

    #[test]
    fn parse_locale_choices() {
        assert_eq!(parse_locale_choice("auto"), Some(LocaleChoice::Auto));
        assert_eq!(parse_locale_choice("sv"), Some(LocaleChoice::Sv));
    }
}
