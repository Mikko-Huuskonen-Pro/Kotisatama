/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kotisatama integration for servoshell (whitelist navigation).

use std::path::Path;
use std::sync::OnceLock;

use kotisatama_whitelist::{blocked_page_url, is_allowed, Whitelist};
use log::warn;
use servo::WebView;
use url::Url;

static WHITELIST: OnceLock<Whitelist> = OnceLock::new();

/// Load whitelist from `KOTISATAMA_WHITELIST_PATH` or `config/whitelist.json`.
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
