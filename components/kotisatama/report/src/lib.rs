/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Submit anonymous Kotisatama user reports and log fallback searches.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

use serde::Serialize;
use url::Url;

const MAX_FALLBACK_QUERY_LEN: usize = 200;

/// Report type sent to the worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportKind {
    /// Whitelisted site does not work.
    SiteBroken,
    /// Suggest a new site for the whitelist.
    SuggestSite,
}

/// Anonymous report payload (no user id).
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub kind: ReportKind,
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub     context_url: Option<String>,
}

/// Anonymous fallback search event (query only — no avomeri data, no user id).
#[derive(Debug, Clone, Serialize)]
pub struct FallbackSearchEvent {
    pub query: String,
    pub platform: String,
}

#[derive(Debug)]
pub enum ReportError {
    MissingEndpoint,
    InvalidDomain,
    InvalidQuery,
    Http(String),
}

impl std::fmt::Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEndpoint => write!(
                f,
                "KOTISATAMA_REPORT_URL not set (Cloudflare Worker endpoint)"
            ),
            Self::InvalidDomain => write!(f, "domain is required"),
            Self::InvalidQuery => write!(f, "query is empty or not loggable"),
            Self::Http(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ReportError {}

static LAST_BLOCKED: Mutex<Option<String>> = Mutex::new(None);

/// Remember the URL that was blocked (for report pre-fill).
pub fn note_blocked_url(url: &Url) {
    if let Ok(mut guard) = LAST_BLOCKED.lock() {
        *guard = Some(url.to_string());
    }
}

/// Last blocked URL, if any.
pub fn last_blocked_url() -> Option<String> {
    LAST_BLOCKED.lock().ok()?.clone()
}

/// Extract a domain/host string from a URL for the report form.
pub fn domain_from_url(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .or_else(|| {
            let trimmed = url.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
}

/// POST report JSON to `KOTISATAMA_REPORT_URL`.
pub fn submit(report: &Report) -> Result<(), ReportError> {
    let domain = report.domain.trim();
    if domain.is_empty() {
        return Err(ReportError::InvalidDomain);
    }

    let endpoint = std::env::var("KOTISATAMA_REPORT_URL").map_err(|_| ReportError::MissingEndpoint)?;

    let response = ureq::post(&endpoint)
        .set("Content-Type", "application/json")
        .send_json(report)
        .map_err(|error| ReportError::Http(error.to_string()))?;

    if response.status() >= 400 {
        return Err(ReportError::Http(format!(
            "report endpoint returned HTTP {}",
            response.status()
        )));
    }
    Ok(())
}

/// Sanitize a fallback search query before logging.
pub fn sanitize_fallback_query(query: &str) -> Option<String> {
    let query = query.trim();
    if query.is_empty() || query.chars().count() > MAX_FALLBACK_QUERY_LEN {
        return None;
    }
    let lower = query.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("data:") {
        return None;
    }
    if query.contains('@') {
        return None;
    }
    Some(query.to_string())
}

/// Log a local-index miss (fire-and-forget). Writes JSONL locally; POSTs if configured.
pub fn log_fallback_search(query: &str, platform: &str) {
    let Some(query) = sanitize_fallback_query(query) else {
        return;
    };

    let event = FallbackSearchEvent {
        query,
        platform: platform.to_string(),
    };

    if let Err(error) = append_local_fallback_log(&event) {
        log::warn!("Kotisatama: local fallback log failed: {error}");
    }

    if let Some(url) = fallback_log_endpoint() {
        let event = event.clone();
        thread::spawn(move || {
            if let Err(error) = post_fallback_event(&url, &event) {
                log::warn!("Kotisatama: remote fallback log failed: {error}");
            }
        });
    }
}

fn data_dir() -> PathBuf {
    std::env::var("KOTISATAMA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("index-data"))
}

fn fallback_log_endpoint() -> Option<String> {
    if let Ok(url) = std::env::var("KOTISATAMA_FALLBACK_LOG_URL") {
        if !url.trim().is_empty() {
            return Some(url);
        }
    }
    std::env::var("KOTISATAMA_ANALYTICS_URL")
        .ok()
        .filter(|url| !url.trim().is_empty())
}

fn append_local_fallback_log(event: &FallbackSearchEvent) -> Result<(), ReportError> {
    let path = data_dir().join("fallback-searches.jsonl");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| ReportError::Http(error.to_string()))?;
    }
    let line = serde_json::to_string(event).map_err(|error| ReportError::Http(error.to_string()))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| ReportError::Http(error.to_string()))?;
    writeln!(file, "{line}").map_err(|error| ReportError::Http(error.to_string()))?;
    Ok(())
}

fn post_fallback_event(url: &str, event: &FallbackSearchEvent) -> Result<(), ReportError> {
    let response = ureq::post(url)
        .set("Content-Type", "application/json")
        .send_json(event)
        .map_err(|error| ReportError::Http(error.to_string()))?;

    if response.status() >= 400 {
        return Err(ReportError::Http(format!(
            "fallback log endpoint returned HTTP {}",
            response.status()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_from_https_url() {
        assert_eq!(
            domain_from_url("https://www.kela.fi/elake"),
            Some("www.kela.fi".into())
        );
    }

    #[test]
    fn rejects_url_like_fallback_queries() {
        assert!(sanitize_fallback_query("https://example.com").is_none());
        assert!(sanitize_fallback_query("  kela eläke  ").is_some());
    }
}
