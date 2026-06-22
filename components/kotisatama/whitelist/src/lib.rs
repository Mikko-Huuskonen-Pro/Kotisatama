/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kotisatama whitelist: curated base list, user overlay, and navigation checks.

mod document;
mod domain;
mod state;
mod user;

use std::sync::Mutex;

pub use document::{WhitelistDocument, WhitelistEntry, WhitelistProfile};
pub use domain::{host_matches_domain, normalize_domain};
pub use state::{
    add_user_domain, init, init_empty, is_navigation_allowed, remove_user_domain, user_entries,
    EffectiveWhitelist,
};
pub use user::{user_whitelist_path, UserWhitelist, UserWhitelistEntry};

use url::Url;

/// Legacy view for callers that only need domain hostnames.
#[derive(Debug, Clone)]
pub struct Whitelist {
    pub domains: Vec<String>,
}

impl Whitelist {
    pub fn load_from_path(path: &std::path::Path) -> Result<Self, WhitelistError> {
        let document = WhitelistDocument::load_from_path(path)?;
        Ok(Self {
            domains: document.domain_hosts_for_profile(&WhitelistProfile::Free),
        })
    }

    pub fn from_json_str(json: &str) -> Result<Self, WhitelistError> {
        let document = WhitelistDocument::from_json_str(json)?;
        Ok(Self {
            domains: document.domain_hosts_for_profile(&WhitelistProfile::Free),
        })
    }

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
    InvalidDomain(String),
    NotInitialized,
}

impl std::fmt::Display for WhitelistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "failed to read whitelist: {error}"),
            Self::Json(error) => write!(f, "failed to parse whitelist JSON: {error}"),
            Self::InvalidDomain(domain) => write!(f, "virheellinen domain: {domain}"),
            Self::NotInitialized => write!(f, "whitelist not initialized"),
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

/// Returns whether navigation to `url` is allowed under `whitelist`.
pub fn is_allowed(url: &Url, whitelist: &Whitelist) -> bool {
    if is_internal_navigation_url(url) || is_avomeri_gateway(url) {
        return true;
    }
    let host = match url.host_str() {
        Some(host) => host.to_ascii_lowercase(),
        None => return false,
    };
    whitelist.domains.iter().any(|domain| host_matches_domain(&host, domain))
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
    Url::parse(&format!("servo:avomeri?q={encoded}")).expect("avomeri gateway URL must be valid")
}

/// Startpage search URL for avomeri fallback.
pub fn startpage_search_url(query: &str) -> Url {
    let encoded = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
    Url::parse(&format!("https://www.startpage.com/search?q={encoded}"))
        .expect("startpage URL must be valid")
}

fn is_internal_navigation_url(url: &Url) -> bool {
    matches!(url.scheme(), "about" | "data" | "servo")
}

fn is_startpage_host(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    host == "startpage.com"
        || host == "www.startpage.com"
        || host.ends_with(".startpage.com")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::EffectiveWhitelist;

    fn whitelist_with(domains: &[&str]) -> Whitelist {
        Whitelist {
            domains: domains.iter().map(|domain| (*domain).to_string()).collect(),
        }
    }

    #[test]
    fn allows_whitelisted_domain_and_subdomain() {
        let whitelist = whitelist_with(&["kela.fi"]);
        let url = Url::parse("https://www.kela.fi/elake").unwrap();
        assert!(is_allowed(&url, &whitelist));
    }

    #[test]
    fn blocks_unknown_domain() {
        let whitelist = whitelist_with(&["kela.fi"]);
        let url = Url::parse("https://example.com/").unwrap();
        assert!(!is_allowed(&url, &whitelist));
    }

    #[test]
    fn allows_about_and_data() {
        let whitelist = whitelist_with(&[]);
        assert!(is_allowed(&Url::parse("about:blank").unwrap(), &whitelist));
        assert!(is_allowed(
            &blocked_page_url(&Url::parse("https://evil.com").unwrap()),
            &whitelist
        ));
    }

    #[test]
    fn allows_startpage_gateway() {
        let whitelist = whitelist_with(&[]);
        let url = Url::parse("https://www.startpage.com/search?q=test").unwrap();
        assert!(is_allowed(&url, &whitelist));
    }

    #[test]
    fn user_overlay_allows_extra_domain() {
        let base =
            WhitelistDocument::from_json_str(r#"{"domains":["kela.fi"]}"#).unwrap();
        let mut user = UserWhitelist::default();
        user.add_domain("example.com", None).unwrap();
        let effective = EffectiveWhitelist::new(base, user, WhitelistProfile::Free);
        assert!(effective.is_host_allowed("www.example.com"));
        assert!(!effective.is_host_allowed("blocked.test"));
    }
}
