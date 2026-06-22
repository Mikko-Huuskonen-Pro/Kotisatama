/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kotisatama integration for servoshell (whitelist navigation + local search).

use std::path::PathBuf;
use std::sync::OnceLock;

use kotisatama_pulloposti::PullopostiClient;
use kotisatama_report::{Report, ReportError, ReportKind, note_blocked_url};
pub use kotisatama_report::{domain_from_url, last_blocked_url};
use kotisatama_search::SearchClient;
pub use kotisatama_search::{SearchHit, SearchOutcome};
use kotisatama_whitelist::{
    blocked_page_url, init, init_empty, is_avomeri_gateway, is_navigation_allowed,
    note_avomeri_query, startpage_query, startpage_search_url, WhitelistProfile,
};
use log::{info, warn};
use servo::WebView;
use url::Url;

static SEARCH: OnceLock<Option<SearchClient>> = OnceLock::new();
static PULLOPOSTI: OnceLock<Option<PullopostiClient>> = OnceLock::new();

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

    let base_path = kotisatama_search::cached_whitelist_path()
        .or_else(|| {
            std::env::var("KOTISATAMA_WHITELIST_PATH")
                .ok()
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| PathBuf::from("config/whitelist.json"));
    let profile = WhitelistProfile::current();
    if let Err(error) = init(&base_path, profile.clone()) {
        warn!(
            "Kotisatama: could not load whitelist from {}: {error}. Using empty base list.",
            base_path.display()
        );
        let _ = init_empty(profile);
    }

    // Meilisearch and Pulloposti start lazily on first use (avoid blocking startup).
}

/// Whether navigation to `url` is allowed.
pub fn check_url(url: &Url) -> bool {
    is_navigation_allowed(url)
}

/// Track allowed navigations (Startpage query for blocked-page fallback).
pub fn on_allowed_navigation(url: &Url) {
    if let Some(query) = startpage_query(url) {
        note_avomeri_query(&query);
    }
}

/// Load `url` or show the blocked page if not whitelisted.
pub fn load_url_or_blocked(webview: &WebView, url: Url) {
    if check_url(&url) {
        on_allowed_navigation(&url);
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
    Url::parse(current_location)
        .map(|url| url.scheme() == "servo" && url.path() == "blocked")
        .unwrap_or(false)
        || current_location.starts_with("data:text/html")
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

/// Pulloposti gateway page (`servo:pulloposti`).
pub fn pulloposti_gateway_url() -> Url {
    PullopostiClient::gateway_url()
}

/// Whether Pulloposti subprocess responds to health checks.
pub fn pulloposti_available() -> bool {
    match PULLOPOSTI.get() {
        Some(Some(client)) => client.is_available(),
        _ => false,
    }
}

/// Startpage URL for avomeri search fallback (direct — no extra gateway page on desktop).
pub fn avomeri_search_url(query: &str) -> Url {
    note_avomeri_query(query);
    startpage_search_url(query)
}

fn ensure_pulloposti() {
    PULLOPOSTI.get_or_init(|| match PullopostiClient::start() {
        Ok(client) => {
            info!("Pulloposti subprocess valmiina");
            Some(client)
        },
        Err(error) => {
            warn!("Pulloposti unavailable: {error}");
            None
        },
    });
}

/// Open Pulloposti gateway (daemon starts in background).
pub fn open_pulloposti(webview: &WebView) {
    std::thread::spawn(|| ensure_pulloposti());
    webview.load(PullopostiClient::gateway_url());
}

/// Search the local Kotisatama index.
pub fn search(query: &str) -> KotisatamaSearchPanel {
    let query = query.trim().to_string();
    if query.is_empty() {
        return KotisatamaSearchPanel {
            query,
            outcome: SearchOutcome::Error("Hakusana puuttuu.".into()),
        };
    }
    note_avomeri_query(&query);
    let platform = if cfg!(target_os = "android") {
        "android"
    } else {
        "desktop"
    };
    let client = SEARCH.get_or_init(|| match SearchClient::start() {
        Ok(client) => Some(client),
        Err(error) => {
            warn!("Kotisatama search unavailable: {error}");
            None
        },
    });
    let outcome = match client {
        Some(client) => client.search(&query),
        None => SearchOutcome::Error(
            "Paikallinen haku ei kaytettavissa. Asenna Meilisearch tai aseta KOTISATAMA_MEILISEARCH_BIN."
                .into(),
        ),
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

// KOTISATAMA: UI-taustateema nykyisen selaustilan mukaan (ks. VAIHE7-TEEMAT.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KotisatamaTheme {
    Satama,
    Avomeri,
    Myrsky,
}

/// Resolve the active chrome theme from navigation and the latest search outcome.
pub fn current_theme(location: &str, last_search: Option<&SearchOutcome>) -> KotisatamaTheme {
    if matches!(last_search, Some(SearchOutcome::Error(_))) {
        return KotisatamaTheme::Myrsky;
    }
    if is_blocked_page(location) {
        return KotisatamaTheme::Avomeri;
    }
    if Url::parse(location)
        .map(|url| is_avomeri_gateway(&url))
        .unwrap_or(false)
    {
        return KotisatamaTheme::Avomeri;
    }
    KotisatamaTheme::Satama
}

fn theme_png_bytes(theme: KotisatamaTheme) -> &'static [u8] {
    match theme {
        KotisatamaTheme::Satama => {
            include_bytes!("../../assets/themes/Satama/Screenshot_20260614-114204.Kuvat.png")
        },
        KotisatamaTheme::Avomeri => {
            include_bytes!("../../assets/themes/Avomeri/Screenshot_20260613-231349.Kuvat.png")
        },
        KotisatamaTheme::Myrsky => {
            include_bytes!("../../assets/themes/Myrsky/Screenshot_20260614-114006.Kuvat.png")
        },
    }
}

/// Toolbar tint matching the active theme (desktop egui chrome).
#[cfg(not(any(target_os = "android", target_env = "ohos")))]
pub fn theme_toolbar_fill(theme: KotisatamaTheme) -> egui::Color32 {
    match theme {
        KotisatamaTheme::Satama => egui::Color32::from_rgb(210, 235, 255),
        KotisatamaTheme::Avomeri => egui::Color32::from_rgb(170, 215, 245),
        KotisatamaTheme::Myrsky => egui::Color32::from_rgb(120, 150, 175),
    }
}

#[cfg(not(any(target_os = "android", target_env = "ohos")))]
fn theme_color_image(theme: KotisatamaTheme) -> egui::ColorImage {
    let bytes = theme_png_bytes(theme);
    let image = image::load_from_memory(bytes).expect("Kotisatama theme PNG");
    let rgba = image.to_rgba8();
    let [width, height] = [rgba.width() as usize, rgba.height() as usize];
    egui::ColorImage::from_rgba_unmultiplied([width, height], rgba.as_raw())
}

/// Paint a full-bleed theme background behind egui and the webview viewport.
#[cfg(not(any(target_os = "android", target_env = "ohos")))]
pub fn paint_theme_background(
    ctx: &egui::Context,
    rect: egui::Rect,
    theme: KotisatamaTheme,
    cache: &mut std::collections::HashMap<KotisatamaTheme, egui::TextureHandle>,
) {
    let handle = cache.entry(theme).or_insert_with(|| {
        let image = theme_color_image(theme);
        ctx.load_texture(
            format!("kotisatama_theme_{theme:?}"),
            image,
            egui::TextureOptions {
                magnification: egui::TextureFilter::Linear,
                minification: egui::TextureFilter::Linear,
                ..Default::default()
            },
        )
    });

    let tex_size = handle.size_vec2();
    if tex_size.x <= 0.0 || tex_size.y <= 0.0 {
        return;
    }

    let scale = (rect.width() / tex_size.x).max(rect.height() / tex_size.y);
    let image_rect = egui::Rect::from_center_size(rect.center(), tex_size * scale);
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    ctx.layer_painter(egui::LayerId::background()).image(
        handle.id(),
        image_rect,
        uv,
        egui::Color32::WHITE,
    );
}
