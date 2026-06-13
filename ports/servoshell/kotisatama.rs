/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kotisatama integration for servoshell (whitelist navigation + local search).

use std::path::Path;
use std::sync::OnceLock;

use kotisatama_search::SearchClient;
pub use kotisatama_search::{SearchHit, SearchOutcome};
use kotisatama_whitelist::{blocked_page_url, is_allowed, startpage_search_url, Whitelist};
use log::warn;
use servo::WebView;
use url::Url;

static WHITELIST: OnceLock<Whitelist> = OnceLock::new();
static SEARCH: OnceLock<Option<SearchClient>> = OnceLock::new();

/// Active Kotisatama search panel state for the servoshell UI.
#[derive(Debug, Clone)]
pub struct KotisatamaSearchPanel {
    pub query: String,
    pub outcome: SearchOutcome,
}

/// Load whitelist and start local search (Meilisearch subprocess if needed).
pub fn init() {
    WHITELIST.get_or_init(|| {
        let path = std::env::var("KOTISATAMA_WHITELIST_PATH")
            .unwrap_or_else(|_| "config/whitelist.json".to_string());
        Whitelist::load_from_path(Path::new(&path)).unwrap_or_else(|error| {
            warn!(
                "Kotisatama: could not load whitelist from {path}: {error}. Using empty whitelist.",
            );
            Whitelist::empty()
        })
    });

    SEARCH.get_or_init(|| match SearchClient::start() {
        Ok(client) => Some(client),
        Err(error) => {
            warn!("Kotisatama search unavailable: {error}");
            None
        },
    });
}

fn whitelist() -> &'static Whitelist {
    WHITELIST.get().expect("kotisatama::init() not called")
}

/// Whether navigation to `url` is allowed.
pub fn check_url(url: &Url) -> bool {
    is_allowed(url, whitelist())
}

/// Load `url` or show the blocked page if not whitelisted.
pub fn load_url_or_blocked(webview: &WebView, url: Url) {
    if check_url(&url) {
        webview.load(url);
    } else {
        webview.load(blocked_page_url(&url));
    }
}

/// URL to load when `url` is not whitelisted.
pub fn blocked_url_for(url: &Url) -> Url {
    blocked_page_url(url)
}

/// Startpage URL for avomeri search fallback.
pub fn avomeri_search_url(query: &str) -> Url {
    startpage_search_url(query)
}

/// Search the local Kotisatama index.
pub fn search(query: &str) -> KotisatamaSearchPanel {
    let query = query.trim().to_string();
    let outcome = match SEARCH.get() {
        Some(Some(client)) => client.search(&query),
        Some(None) => SearchOutcome::Error(
            "Paikallinen haku ei käytettävissä. Asenna Meilisearch tai aseta KOTISATAMA_MEILISEARCH_BIN."
                .into(),
        ),
        None => SearchOutcome::Error("Kotisatama-haku ei alustettu.".into()),
    };
    KotisatamaSearchPanel { query, outcome }
}

/// Load a search hit URL in the webview (whitelist-checked).
pub fn open_search_hit(webview: &WebView, hit: &SearchHit) {
    if let Ok(url) = Url::parse(&hit.url) {
        load_url_or_blocked(webview, url);
    }
}
