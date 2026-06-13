/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kotisatama integration for servoshell (whitelist navigation + local search).

use std::path::PathBuf;
use std::sync::OnceLock;

use kotisatama_report::{note_blocked_url, Report, ReportError, ReportKind};
use kotisatama_search::SearchClient;
pub use kotisatama_search::{SearchHit, SearchOutcome};
pub use kotisatama_report::{domain_from_url, last_blocked_url};
use kotisatama_whitelist::{
    blocked_page_url, is_allowed, is_avomeri_gateway, startpage_search_url, Whitelist,
};
use log::{info, warn};
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

/// Report form state for the servoshell UI.
#[derive(Debug, Clone)]
pub struct KotisatamaReportForm {
    pub kind: ReportKind,
    pub domain: String,
    pub message: String,
}

/// Load whitelist and start local search (Meilisearch subprocess if needed).
pub fn init() {
    if let Ok(cdn_base) = std::env::var("KOTISATAMA_CDN_BASE") {
        match kotisatama_search::sync_from_cdn(&cdn_base) {
            Ok(report) if report.whitelist_updated || report.index_dump_updated => {
                info!(
                    "Kotisatama CDN sync: whitelist={}, index={}",
                    report.whitelist_updated, report.index_dump_updated
                );
            },
            Ok(_) => {},
            Err(error) => warn!("Kotisatama CDN sync failed: {error}"),
        }
    }

    WHITELIST.get_or_init(|| {
        let path = kotisatama_search::cached_whitelist_path()
            .or_else(|| {
                std::env::var("KOTISATAMA_WHITELIST_PATH")
                    .ok()
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| PathBuf::from("config/whitelist.json"));
        Whitelist::load_from_path(&path).unwrap_or_else(|error| {
            warn!(
                "Kotisatama: could not load whitelist from {}: {error}. Using empty whitelist.",
                path.display()
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
        note_blocked_url(&url);
        webview.load(blocked_page_url(&url));
    }
}

/// URL to load when `url` is not whitelisted.
pub fn blocked_url_for(url: &Url) -> Url {
    note_blocked_url(url);
    blocked_page_url(url)
}

/// Whether the report button should be shown for the current location.
pub fn should_show_report_button(current_location: &str) -> bool {
    if is_blocked_page(current_location) {
        return true;
    }
    Url::parse(current_location)
        .map(|url| !is_avomeri_gateway(&url))
        .unwrap_or(true)
}

/// Whether the active page is the Kotisatama blocked error page.
pub fn is_blocked_page(current_location: &str) -> bool {
    current_location.starts_with("data:text/html")
}

/// Default report form values from the current browser location.
pub fn default_report_form(current_location: &str) -> KotisatamaReportForm {
    let on_blocked = is_blocked_page(current_location);
    let domain = last_blocked_url()
        .and_then(|url| domain_from_url(&url))
        .or_else(|| domain_from_url(current_location))
        .unwrap_or_default();

    KotisatamaReportForm {
        kind: if on_blocked {
            ReportKind::SuggestSite
        } else {
            ReportKind::SiteBroken
        },
        domain,
        message: String::new(),
    }
}

/// Submit an anonymous user report to the Cloudflare Worker endpoint.
pub fn submit_report(
    form: &KotisatamaReportForm,
    context_url: Option<String>,
) -> Result<(), ReportError> {
    let message = match form.kind {
        ReportKind::SiteBroken if !form.message.trim().is_empty() => {
            Some(form.message.trim().to_string())
        },
        _ => None,
    };

    kotisatama_report::submit(&Report {
        kind: form.kind,
        domain: form.domain.trim().to_string(),
        message,
        context_url,
    })
}

/// Startpage URL for avomeri search fallback.
pub fn avomeri_search_url(query: &str) -> Url {
    startpage_search_url(query)
}

/// Search the local Kotisatama index.
pub fn search(query: &str) -> KotisatamaSearchPanel {
    let query = query.trim().to_string();
    let platform = if cfg!(target_os = "android") {
        "android"
    } else {
        "desktop"
    };
    let outcome = match SEARCH.get() {
        Some(Some(client)) => client.search(&query),
        Some(None) => SearchOutcome::Error(
            "Paikallinen haku ei käytettävissä. Asenna Meilisearch tai aseta KOTISATAMA_MEILISEARCH_BIN."
                .into(),
        ),
        None => SearchOutcome::Error("Kotisatama-haku ei alustettu.".into()),
    };
    if matches!(outcome, SearchOutcome::NoResults) {
        kotisatama_report::log_fallback_search(&query, platform);
    }
    KotisatamaSearchPanel { query, outcome }
}

/// Load a search hit URL in the webview (whitelist-checked).
pub fn open_search_hit(webview: &WebView, hit: &SearchHit) {
    if let Ok(url) = Url::parse(&hit.url) {
        load_url_or_blocked(webview, url);
    }
}
