/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kotisatama whitelist: curated base list, user overlay, and navigation checks.

mod document;
mod domain;
mod product_profile;
mod resolve;
mod state;
mod user;

pub use document::{
    CategoryMeta, RegionMeta, TypeMeta, WhitelistDocument, WhitelistEntry, WhitelistProfile,
    INTERNAL_PROFILE_TAGS, display_tags, is_internal_tag, public_tags,
};
pub use domain::{host_matches_domain, normalize_domain};
pub use product_profile::{ProductProfile, effective_whitelist_profile};
pub use resolve::{curated_whitelist_candidates, init_with_fallback};
pub use state::{
    EffectiveWhitelist, add_user_domain, curated_document, init, init_empty,
    is_navigation_allowed, lookup_curated_entry, remove_user_domain, user_entries,
};
pub use user::{UserWhitelist, UserWhitelistEntry, user_whitelist_path};

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
    NoBaseListFound,
}

impl std::fmt::Display for WhitelistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "failed to read whitelist: {error}"),
            Self::Json(error) => write!(f, "failed to parse whitelist JSON: {error}"),
            Self::InvalidDomain(domain) => write!(f, "virheellinen domain: {domain}"),
            Self::NotInitialized => write!(f, "whitelist not initialized"),
            Self::NoBaseListFound => write!(f, "no curated whitelist could be loaded"),
        }
    }
}

impl std::error::Error for WhitelistError {}

/// Returns whether navigation to `url` is allowed under `whitelist`.
pub fn is_allowed(url: &Url, whitelist: &Whitelist) -> bool {
    if is_internal_navigation_url(url) {
        return true;
    }
    let host = match url.host_str() {
        Some(host) => host.to_ascii_lowercase(),
        None => return false,
    };
    whitelist
        .domains
        .iter()
        .any(|domain| host_matches_domain(&host, domain))
}

/// Whether `url` is the internal Avomeri port.
pub fn is_avomeri_gateway(url: &Url) -> bool {
    url.scheme() == "servo" && url.path() == "avomeri"
}

/// Build the internal blocked-page URL (`servo:blocked` + query params).
pub fn blocked_page_url(blocked_url: &Url) -> Url {
    let display = blocked_url.as_str();

    let mut url = Url::parse("servo:blocked").expect("servo:blocked URL must be valid");
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("u", display);
    }
    url
}

/// Internal Kotisatama avomeri gateway page (`servo:avomeri?q=...`).
pub fn avomeri_gateway_url(query: &str) -> Url {
    let encoded = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
    Url::parse(&format!("servo:avomeri?q={encoded}")).expect("avomeri gateway URL must be valid")
}

fn is_internal_navigation_url(url: &Url) -> bool {
    matches!(url.scheme(), "about" | "data" | "servo")
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
    fn kela_mvp_routes_stay_in_satama() {
        let whitelist = whitelist_with(&["kela.fi"]);
        for url in [
            "https://www.kela.fi/",
            "https://www.kela.fi/elake",
            "https://asiointi.kela.fi/",
        ] {
            assert!(is_allowed(&Url::parse(url).unwrap(), &whitelist), "{url}");
        }
    }

    #[test]
    fn kela_lookalike_domains_stay_blocked() {
        let whitelist = whitelist_with(&["kela.fi"]);
        for url in [
            "https://kela.fi.example.com/",
            "https://example-kela.fi/",
            "https://kelafi.example/",
        ] {
            assert!(!is_allowed(&Url::parse(url).unwrap(), &whitelist), "{url}");
        }
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
    fn blocks_startpage_without_explicit_whitelist_entry() {
        let whitelist = whitelist_with(&[]);
        let url = Url::parse("https://www.startpage.com/search?q=test").unwrap();
        assert!(!is_allowed(&url, &whitelist));
    }

    #[test]
    fn allows_internal_avomeri_port() {
        let whitelist = whitelist_with(&[]);
        let url = avomeri_gateway_url("test");
        assert!(is_allowed(&url, &whitelist));
    }

    #[test]
    fn user_overlay_allows_extra_domain() {
        let base = WhitelistDocument::from_json_str(r#"{"domains":["kela.fi"]}"#).unwrap();
        let mut user = UserWhitelist::default();
        user.add_domain("example.com", None).unwrap();
        let effective = EffectiveWhitelist::new(base, user, WhitelistProfile::Free);
        assert!(effective.is_host_allowed("www.example.com"));
        assert!(!effective.is_host_allowed("blocked.test"));
    }
}
