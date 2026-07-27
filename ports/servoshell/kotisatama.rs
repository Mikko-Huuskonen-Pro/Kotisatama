/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Kotisatama integration for servoshell (whitelist navigation + local search).

use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use kotisatama_missa_olen::MissaOlenClient;
use kotisatama_pulloposti::PullopostiClient;
use kotisatama_report::{Report, ReportError, ReportKind, note_blocked_url};
pub use kotisatama_report::{domain_from_url, last_blocked_url};
use kotisatama_search::SearchClient;
pub use kotisatama_search::{
    EnrichedSearchHit, EnrichedSearchOutcome, SearchHit, SearchOutcome, enrich_outcome,
};
use kotisatama_varustamo::gateway_url as varustamo_gateway_url;
pub use kotisatama_varustamo::{VarustamoRegistry, app_gateway_url, load_registry};
use kotisatama_whitelist::{
    CategoryMeta, TypeMeta, WhitelistDocument, avomeri_gateway_url, blocked_page_url,
    curated_document, effective_whitelist_profile, init_with_fallback, is_avomeri_gateway,
    is_navigation_allowed, ProductProfile,
};
use log::{info, warn};
use serde::Serialize;
use servo::WebView;
use url::Url;

const AVOMERI_DEFAULT_SEARCHPAGE: &str = "https://www.qwant.com/?q=%s";

static SEARCH: OnceLock<Option<SearchClient>> = OnceLock::new();
static PULLOPOSTI: OnceLock<Option<PullopostiClient>> = OnceLock::new();
static MISSA_OLEN: OnceLock<Option<MissaOlenClient>> = OnceLock::new();
static CONTENT_BLOCKING: OnceLock<kotisatama_content_blocking::ContentBlockingService> =
    OnceLock::new();
static AVOMERI_MODE: AtomicBool = AtomicBool::new(false);
static AVOMERI_SEARCHPAGE: OnceLock<String> = OnceLock::new();

fn whitelist_base_path() -> PathBuf {
    kotisatama_search::cached_whitelist_path()
        .or_else(|| {
            std::env::var("KOTISATAMA_WHITELIST_PATH")
                .ok()
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| PathBuf::from("config/whitelist.json"))
}

/// Active product profile (normaali / lapsi / seniori / hopeakettu).
pub fn product_profile() -> ProductProfile {
    ProductProfile::current()
}

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

    let cache = kotisatama_search::cached_whitelist_path();
    let profile = effective_whitelist_profile();
    match init_with_fallback(cache, profile.clone()) {
        Ok(path) => info!("Kotisatama: whitelist active from {}", path.display()),
        Err(error) => warn!(
            "Kotisatama: whitelist not loaded ({error}). Navigation restricted to internal pages."
        ),
    }

    // Meilisearch, Pulloposti and Varustamo apps start lazily on first use.
    // KOTISATAMA-PATCH: Varustamo parkkeerattu (ei ydintoimintoa).
    if varustamo_enabled() {
        match load_registry() {
            Ok(registry) => info!(
                "Varustamo: {} apps loaded from registry",
                registry.displayable_apps().len()
            ),
            Err(error) => warn!("Varustamo registry not loaded: {error}"),
        }
    } else {
        info!("Varustamo: pois käytöstä (parkkeerattu)");
    }

    // KOTISATAMA-PATCH: mainostenesto (adblock-Katselin) — fail-open jos lista puuttuu.
    let blocking = kotisatama_content_blocking::ContentBlockingService::from_bundled_filters();
    info!("Kotisatama content-blocking: {:?}", blocking.status());
    let _ = CONTENT_BLOCKING.set(blocking);
}

/// Whether navigation to `url` is allowed.
pub fn check_url(url: &Url) -> bool {
    is_navigation_allowed(url)
}

/// Content-blocking service (fail-open inactive if not initialized).
pub fn content_blocking() -> &'static kotisatama_content_blocking::ContentBlockingService {
    CONTENT_BLOCKING.get_or_init(kotisatama_content_blocking::ContentBlockingService::inactive)
}

/// Map Servo CSP Destination debug name → ResourceType.
pub fn resource_type_from_destination_name(name: &str) -> kotisatama_content_blocking::ResourceType {
    use kotisatama_content_blocking::ResourceType;
    match name {
        "Document" => ResourceType::Document,
        "IFrame" | "Frame" => ResourceType::Subdocument,
        "Script" | "ServiceWorker" | "SharedWorker" | "Worker" | "AudioWorklet" | "PaintWorklet" => {
            ResourceType::Script
        },
        "Style" => ResourceType::Stylesheet,
        "Image" => ResourceType::Image,
        "Font" => ResourceType::Font,
        "Json" | "Report" => ResourceType::XmlHttpRequest,
        "Audio" | "Video" | "Track" => ResourceType::Media,
        _ => ResourceType::Other,
    }
}

/// Returns true if the subresource should be blocked (never blocks main documents here —
/// navigations stay on the whitelist path).
pub fn should_block_web_resource(
    url: &str,
    source_url: &str,
    destination_name: &str,
    is_for_main_frame: bool,
) -> bool {
    use kotisatama_content_blocking::{BlockingDecision, BlockingRequest, RequestBlocker, ResourceType};

    if is_for_main_frame || destination_name == "Document" {
        return false;
    }

    let resource_type = resource_type_from_destination_name(destination_name);
    if resource_type == ResourceType::Document {
        return false;
    }

    let request = BlockingRequest {
        url,
        source_url,
        resource_type,
    };
    matches!(
        content_blocking().check(&request),
        BlockingDecision::Block
    )
}

/// Whether a navigation should be allowed in this webview.
///
/// Avomeri is an internal port; it does not grant open-web navigation by itself.
pub fn should_allow_navigation(webview: &WebView, target: &Url) -> bool {
    let _ = webview;
    check_url(target)
        || (avomeri_mode_enabled() && ProductProfile::current().can_enter_avomeri())
}

/// Track allowed navigations.
pub fn on_allowed_navigation(url: &Url) {
    let _ = url;
    // KOTISATAMA-PATCH: nollaa sivukohtainen estolaskuri uudella sivulla.
    content_blocking().reset_page_stats();
}

/// Estettyjen pyyntöjen määrä nykyisellä sivulla.
pub fn blocked_count_on_page() -> u64 {
    content_blocking().statistics().blocked_count()
}

/// Onko suodatusmoottori aktiivinen.
pub fn content_blocking_active() -> bool {
    use kotisatama_content_blocking::ContentBlockingStatus;
    content_blocking().status() == ContentBlockingStatus::Active
}

/// Onko sivustolle (URL tai domain) poikkeus.
pub fn site_protection_disabled(url_or_domain: &str) -> bool {
    content_blocking().exceptions().is_allowed(url_or_domain)
}

/// Salli sisältö nykyisellä sivustolla (poikkeus) ja palauta normalisoitu domain.
pub fn allow_site_protection_exception(page_url: &str) -> Option<String> {
    let domain = domain_from_url_str(page_url)?;
    content_blocking().exceptions().allow_site(&domain);
    Some(domain)
}

/// Poista sivustopoikkeus.
pub fn remove_site_protection_exception(page_url: &str) -> Option<String> {
    let domain = domain_from_url_str(page_url)?;
    content_blocking().exceptions().remove_site(&domain);
    Some(domain)
}

fn domain_from_url_str(page_url: &str) -> Option<String> {
    let url = Url::parse(page_url).ok()?;
    url.host_str().map(|h| h.to_ascii_lowercase())
}

/// Load `url` or show the blocked page if not whitelisted.
pub fn load_url_or_blocked(webview: &WebView, url: Url) {
    if should_allow_navigation(webview, &url) {
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

/// Submit an anonymous user report to Katselin.fi GitHub Issues (token or worker URL).
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

/// Internal Avomeri port URL. This does not grant open internet access by itself.
pub fn avomeri_search_url(query: &str) -> Url {
    avomeri_gateway_url(query)
}

/// Open web target after the user explicitly confirms Avomeri mode.
pub fn avomeri_open_url(query: &str) -> Url {
    avomeri_open_url_with_searchpage(query, AVOMERI_SEARCHPAGE.get().map(String::as_str))
}

pub fn set_avomeri_searchpage(searchpage: &str) {
    let _ = AVOMERI_SEARCHPAGE.set(searchpage.trim().to_owned());
}

fn avomeri_open_url_with_searchpage(query: &str, searchpage: Option<&str>) -> Url {
    let query = query.trim();
    let searchpage = searchpage
        .map(str::trim)
        .filter(|value| value.starts_with("https://") && value.contains("%s"))
        .unwrap_or(AVOMERI_DEFAULT_SEARCHPAGE);
    let target = if query.is_empty() {
        avomeri_home_url(searchpage)
    } else {
        let encoded = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
        searchpage.replace("%s", &encoded)
    };
    Url::parse(&target).expect("Avomeri target URL must be valid")
}

fn avomeri_home_url(searchpage: &str) -> String {
    match Url::parse(searchpage) {
        Ok(mut url) => {
            url.set_path("/");
            url.set_query(None);
            url.set_fragment(None);
            url.to_string()
        },
        Err(_) => "https://www.qwant.com/".to_owned(),
    }
}

pub fn enter_avomeri_mode() {
    if ProductProfile::current().can_enter_avomeri() {
        AVOMERI_MODE.store(true, Ordering::Relaxed);
    }
}

pub fn leave_avomeri_mode() {
    AVOMERI_MODE.store(false, Ordering::Relaxed);
}

pub fn avomeri_mode_enabled() -> bool {
    AVOMERI_MODE.load(Ordering::Relaxed)
}

/// Resolve a simple address-bar word such as "kela" to a curated domain.
pub fn resolve_address_alias(input: &str) -> Option<Url> {
    let query = input.trim().to_ascii_lowercase();
    if query.is_empty() || query.contains(char::is_whitespace) || query.contains('.') {
        return None;
    }

    let document = WhitelistDocument::load_from_path(&whitelist_base_path()).ok()?;
    let profile = effective_whitelist_profile();
    document
        .entries_for_profile(&profile)
        .into_iter()
        .find_map(|entry| {
            let label_matches = entry
                .label
                .as_deref()
                .map(|label| label.trim().eq_ignore_ascii_case(&query))
                .unwrap_or(false);
            let domain_alias = entry.domain.split('.').next().unwrap_or_default();
            if label_matches || domain_alias.eq_ignore_ascii_case(&query) {
                entry.navigation_url()
            } else {
                None
            }
        })
}

pub fn ensure_pulloposti() {
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

/// Varustamo hub page (`servo:varustamo`).
pub fn open_varustamo(webview: &WebView) {
    if !varustamo_enabled() {
        warn!("Varustamo: ohitettu (parkkeerattu)");
        return;
    }
    webview.load(varustamo_gateway_url());
}

/// Open a Varustamo app by registry id (starts daemon when needed).
pub fn open_varustamo_app(webview: &WebView, app_id: &str) {
    if !varustamo_enabled() {
        warn!("Varustamo: ohitettu (parkkeerattu), app={app_id}");
        return;
    }
    match app_id {
        "pulloposti" => open_pulloposti(webview),
        "missa-olen" => open_missa_olen(webview),
        _ => {
            if let Ok(url) = app_gateway_url(app_id) {
                webview.load(url);
            } else {
                warn!("Varustamo: unknown app id {app_id}");
            }
        },
    }
}

/// Loaded Varustamo registry, if available.
pub fn varustamo_registry() -> Option<VarustamoRegistry> {
    if !varustamo_enabled() {
        return None;
    }
    load_registry().ok()
}

/// KOTISATAMA-PATCH: Varustamo parkkeerattu oletuksena (ei ydintoimintoa).
/// Takaisin: `KOTISATAMA_VARUSTAMO=1` tai vaihda oletus `true`.
pub fn varustamo_enabled() -> bool {
    match std::env::var("KOTISATAMA_VARUSTAMO") {
        Ok(v) => matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"),
        Err(_) => false,
    }
}

/// Missä olen gateway (`servo:missa-olen`).
pub fn open_missa_olen(webview: &WebView) {
    std::thread::spawn(|| ensure_missa_olen());
    webview.load(MissaOlenClient::gateway_url());
}

/// Whether Missä olen daemon responds to health checks.
pub fn missa_olen_available() -> bool {
    match MISSA_OLEN.get() {
        Some(Some(client)) => client.is_available(),
        _ => false,
    }
}

pub fn ensure_missa_olen() {
    MISSA_OLEN.get_or_init(|| match MissaOlenClient::start() {
        Ok(client) => {
            info!("Missä olen subprocess valmiina");
            Some(client)
        },
        Err(error) => {
            warn!("Missä olen unavailable: {error}");
            None
        },
    });
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

/// Internal search results page (`servo:haku?q=...`).
pub fn search_results_url(query: &str) -> Url {
    let encoded = url::form_urlencoded::byte_serialize(query.trim().as_bytes()).collect::<String>();
    Url::parse(&format!("servo:haku?q={encoded}")).expect("haku URL must be valid")
}

/// Whether Enter should open the best hit directly instead of the results page.
pub fn should_open_best_hit_directly(hits: &[SearchHit]) -> bool {
    hits.len() == 1
}

/// Route address-bar search input (alias, direct hit, or results page).
pub fn open_search_or_results(webview: &WebView, query: &str, force_results_page: bool) {
    let query = query.trim();
    if query.is_empty() {
        return;
    }

    if let Some(url) = resolve_address_alias(query) {
        load_url_or_blocked(webview, url);
        return;
    }

    let panel = search(query);
    match panel.outcome {
        SearchOutcome::Hits(ref hits)
            if !force_results_page && should_open_best_hit_directly(hits) =>
        {
            if let Some(hit) = hits.first() {
                open_search_hit(webview, hit);
            }
        },
        SearchOutcome::Hits(_) | SearchOutcome::NoResults | SearchOutcome::Error(_) => {
            load_url_or_blocked(webview, search_results_url(&panel.query));
        },
    }
}

#[derive(Serialize)]
struct SearchResultsData {
    query: String,
    status: &'static str,
    message: Option<String>,
    hits: Vec<EnrichedSearchHit>,
    categories: Vec<CategoryMeta>,
    types: Vec<TypeMeta>,
}

/// JSON payload for `servo:haku/data?q=...`.
pub fn search_results_json(query: &str) -> String {
    let panel = search(query);
    let enriched = enrich_outcome(&panel.outcome);
    let document = curated_document();
    let categories = document
        .as_ref()
        .map(|doc| doc.categories.clone())
        .unwrap_or_default();
    let types = document
        .as_ref()
        .map(|doc| doc.types.clone())
        .unwrap_or_default();

    let (status, message, hits) = match enriched {
        EnrichedSearchOutcome::Hits(hits) => ("hits", None, hits),
        EnrichedSearchOutcome::NoResults => ("no_results", None, Vec::new()),
        EnrichedSearchOutcome::Error(message) => ("error", Some(message), Vec::new()),
    };

    serde_json::to_string(&SearchResultsData {
        query: panel.query,
        status,
        message,
        hits,
        categories,
        types,
    })
    .unwrap_or_else(|_| {
        r#"{"status":"error","message":"JSON serialisointi epäonnistui"}"#.to_owned()
    })
}

/// Load a search hit URL in the webview (whitelist-checked).
pub fn open_search_hit(webview: &WebView, hit: &SearchHit) {
    if let Ok(url) = Url::parse(&hit.url) {
        load_url_or_blocked(webview, url);
    }
}

// KOTISATAMA: UI-taustateema nykyisen selaustilan mukaan (ks. suljetun repon Docs/VAIHE7-TEEMAT.md).
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

#[cfg(test)]
mod tests {
    use super::avomeri_open_url_with_searchpage;

    #[test]
    fn avomeri_defaults_to_qwant() {
        let url = avomeri_open_url_with_searchpage("katsastus", None);
        assert_eq!(url.as_str(), "https://www.qwant.com/?q=katsastus");
    }

    #[test]
    fn avomeri_supports_startpage_template() {
        let url = avomeri_open_url_with_searchpage(
            "katsastus",
            Some("https://www.startpage.com/search?q=%s"),
        );
        assert_eq!(url.as_str(), "https://www.startpage.com/search?q=katsastus");
    }

    #[test]
    fn avomeri_supports_duckduckgo_template() {
        let url = avomeri_open_url_with_searchpage(
            "katsastus",
            Some("https://duckduckgo.com/html/?q=%s"),
        );
        assert_eq!(url.as_str(), "https://duckduckgo.com/html/?q=katsastus");
    }

    #[test]
    fn avomeri_empty_query_opens_search_engine_home() {
        let url = avomeri_open_url_with_searchpage("", None);
        assert_eq!(url.as_str(), "https://www.qwant.com/");
    }

    #[test]
    fn avomeri_empty_query_uses_selected_search_engine_root() {
        let url =
            avomeri_open_url_with_searchpage("", Some("https://www.startpage.com/search?q=%s"));
        assert_eq!(url.as_str(), "https://www.startpage.com/");
    }
}
