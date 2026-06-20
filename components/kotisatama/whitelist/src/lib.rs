/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kotisatama whitelist: domain checks and blocked-navigation page generation.

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use serde::Deserialize;
use url::Url;

/// Whitelist loaded from JSON (`config/whitelist.json`).
#[derive(Debug, Clone, Deserialize)]
pub struct Whitelist {
    /// Allowed registrable domains (e.g. `kela.fi`). Subdomains match automatically.
    pub domains: Vec<String>,
}

impl Whitelist {
    /// Load whitelist from a JSON file.
    pub fn load_from_path(path: &Path) -> Result<Self, WhitelistError> {
        let contents = fs::read_to_string(path).map_err(WhitelistError::Io)?;
        Self::from_json_str(&contents)
    }

    /// Parse whitelist JSON.
    pub fn from_json_str(json: &str) -> Result<Self, WhitelistError> {
        let whitelist = serde_json::from_str(json).map_err(WhitelistError::Json)?;
        Ok(whitelist)
    }

    /// Empty whitelist (everything external is blocked except internal/avomeri URLs).
    pub fn empty() -> Self {
        Self {
            domains: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum WhitelistError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for WhitelistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "failed to read whitelist: {e}"),
            Self::Json(e) => write!(f, "failed to parse whitelist JSON: {e}"),
        }
    }
}

impl std::error::Error for WhitelistError {}

static LAST_AVOMERI_QUERY: Mutex<Option<String>> = Mutex::new(None);

/// Remember the last avomeri/Startpage search query (blocked-page fallback link).
pub fn note_avomeri_query(query: &str) {
    let query = query.trim();
    if query.is_empty() || query.chars().count() > 200 {
        return;
    }
    if let Ok(mut guard) = LAST_AVOMERI_QUERY.lock() {
        *guard = Some(query.to_string());
    }
}

/// Last avomeri search query, if any.
pub fn last_avomeri_query() -> Option<String> {
    LAST_AVOMERI_QUERY.lock().ok()?.clone()
}

/// Hosts that act as the avomeri gateway (Startpage). MVP: always allowed so the
/// blocked-page link works. Users can also navigate here directly via the URL bar.
fn is_startpage_host(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    host == "startpage.com"
        || host == "www.startpage.com"
        || host.ends_with(".startpage.com")
}

/// Returns whether navigation to `url` is allowed under `whitelist`.
pub fn is_allowed(url: &Url, whitelist: &Whitelist) -> bool {
    if is_internal_navigation_url(url) {
        return true;
    }

    if is_avomeri_gateway(url) {
        return true;
    }

    let host = match url.host_str() {
        Some(host) => host.to_ascii_lowercase(),
        None => return false,
    };

    whitelist.domains.iter().any(|domain| {
        let domain = domain.trim().to_ascii_lowercase();
        if domain.is_empty() {
            return false;
        }
        host == domain || host.ends_with(&format!(".{domain}"))
    })
}

fn is_internal_navigation_url(url: &Url) -> bool {
    match url.scheme() {
        "about" | "data" | "servo" => true,
        "file" => false,
        _ => false,
    }
}

/// Whether `url` is the avomeri (Startpage) gateway — report UI is hidden here.
pub fn is_avomeri_gateway(url: &Url) -> bool {
    url.host_str().map(is_startpage_host).unwrap_or(false)
}

/// Extract Startpage `q` query parameter if present.
pub fn startpage_query(url: &Url) -> Option<String> {
    if !is_avomeri_gateway(url) {
        return None;
    }
    url.query_pairs()
        .find(|(key, _)| key == "q")
        .map(|(_, value)| value.into_owned())
        .filter(|query| !query.trim().is_empty())
}

/// Build the internal blocked-page URL (`servo:blocked` + query params).
/// HTML lives in `resources/resource_protocol/blocked.html` with i18n via `kotisatama-i18n.js`.
pub fn blocked_page_url(blocked_url: &Url) -> Url {
    let display = blocked_url.as_str();
    let search_term = last_avomeri_query().unwrap_or_else(|| {
        blocked_url
            .host_str()
            .unwrap_or_else(|| blocked_url.as_str())
            .to_string()
    });
    let continue_href = startpage_search_url(&search_term);

    let mut url = Url::parse("servo:blocked").expect("servo:blocked URL must be valid");
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("u", display);
        pairs.append_pair("continue", continue_href.as_str());
    }
    url
}

/// Internal Kotisatama avomeri gateway page (`servo:avomeri?q=...`).
pub fn avomeri_gateway_url(query: &str) -> Url {
    let encoded = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
    Url::parse(&format!("servo:avomeri?q={encoded}"))
        .expect("avomeri gateway URL must be valid")
}

/// Startpage search URL for avomeri fallback.
pub fn startpage_search_url(query: &str) -> Url {
    let encoded = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
    Url::parse(&format!("https://www.startpage.com/search?q={encoded}"))
        .expect("startpage URL must be valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn whitelist_with(domains: &[&str]) -> Whitelist {
        Whitelist {
            domains: domains.iter().map(|d| (*d).to_string()).collect(),
        }
    }

    #[test]
    fn allows_whitelisted_domain_and_subdomain() {
        let wl = whitelist_with(&["kela.fi"]);
        let url = Url::parse("https://www.kela.fi/elake").unwrap();
        assert!(is_allowed(&url, &wl));
    }

    #[test]
    fn blocks_unknown_domain() {
        let wl = whitelist_with(&["kela.fi"]);
        let url = Url::parse("https://example.com/").unwrap();
        assert!(!is_allowed(&url, &wl));
    }

    #[test]
    fn allows_about_and_data() {
        let wl = whitelist_with(&[]);
        assert!(is_allowed(&Url::parse("about:blank").unwrap(), &wl));
        assert!(is_allowed(&blocked_page_url(&Url::parse("https://evil.com").unwrap()), &wl));
    }

    #[test]
    fn allows_startpage_gateway() {
        let wl = whitelist_with(&[]);
        let url = Url::parse("https://www.startpage.com/search?q=test").unwrap();
        assert!(is_allowed(&url, &wl));
        let eu = Url::parse("https://eu.startpage.com/sp/search?q=test").unwrap();
        assert!(is_allowed(&eu, &wl));
    }

    #[test]
    fn blocked_page_uses_servo_scheme() {
        let blocked = blocked_page_url(&Url::parse("https://example.com/path").unwrap());
        assert_eq!(blocked.scheme(), "servo");
        assert_eq!(blocked.path(), "blocked");
        assert!(blocked
            .query()
            .unwrap_or("")
            .contains("u=https%3A%2F%2Fexample.com%2Fpath"));
    }
}
