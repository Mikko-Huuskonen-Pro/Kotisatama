/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kotisatama whitelist: domain checks and blocked-navigation page generation.

use std::fs;
use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine};
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

/// Hosts that act as the avomeri gateway (Startpage). MVP: always allowed so the
/// blocked-page link works. Users can also navigate here directly via the URL bar.
const AVOMERI_GATEWAY_HOSTS: &[&str] = &["startpage.com", "www.startpage.com"];

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
    url.host_str()
        .map(|host| {
            let host = host.to_ascii_lowercase();
            AVOMERI_GATEWAY_HOSTS
                .iter()
                .any(|allowed| host == *allowed)
        })
        .unwrap_or(false)
}

/// Build a `data:` URL HTML page shown when navigation is blocked.
pub fn blocked_page_url(blocked_url: &Url) -> Url {
    let display = blocked_url.as_str();
    let search_term = blocked_url
        .host_str()
        .unwrap_or_else(|| blocked_url.as_str());
    let startpage_href = startpage_search_url(search_term);

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="fi">
<head>
  <meta charset="utf-8">
  <title>Ei löydy kotisatamasta</title>
  <style>
    body {{ font-family: system-ui, sans-serif; margin: 2.5rem; line-height: 1.5; color: #1a1a1a; }}
    h1 {{ font-size: 1.5rem; margin-bottom: 0.75rem; }}
    p {{ margin: 0.5rem 0; }}
    a {{ font-size: 1.05rem; }}
    .url {{ color: #555; font-size: 0.95rem; }}
  </style>
</head>
<body>
  <h1>Tätä sivua ei löydy kotisatamassa.</h1>
  <p class="url">{display}</p>
  <p><a href="{startpage_href}">Jatka avomerelle</a></p>
  <p style="margin-top: 1.5rem; color: #666; font-size: 0.9rem;">Voit ilmoittaa ongelmasta tai ehdottaa sivustoa selaimen <strong>Ilmoita</strong>-napilla.</p>
</body>
</html>"#,
    );

    let data_url = format!(
        "data:text/html;base64,{}",
        STANDARD.encode(html.as_bytes())
    );
    Url::parse(&data_url).expect("blocked page data URL must be valid")
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
    }
}
