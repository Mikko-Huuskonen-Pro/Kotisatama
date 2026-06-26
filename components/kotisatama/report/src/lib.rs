/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Submit anonymous Kotisatama user reports and log fallback searches.
//!
//! Lokikirja-ilmoitukset tallentuvat Katselin.fi-repoon GitHub-issueina, kun joko
//! `KOTISATAMA_GITHUB_TOKEN` (kehitys) tai `KOTISATAMA_REPORT_URL` (tuotanto-worker)
//! on asetettu. Ilman näitä kirjoitus menee paikalliseen `reports.jsonl`-jonoon.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;

use serde::Serialize;
use url::Url;

const MAX_FALLBACK_QUERY_LEN: usize = 200;
const DEFAULT_GITHUB_REPO: &str = "Mikko-Huuskonen-Pro/Katselin.fi";
const GITHUB_API_VERSION: &str = "2022-11-28";

/// Report type sent to GitHub Issues or the report worker.
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
    pub context_url: Option<String>,
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
                "Ilmoitusta ei voitu lähettää: aseta KOTISATAMA_GITHUB_TOKEN tai KOTISATAMA_REPORT_URL (katso katselin.fi/kehittajille)"
            ),
            Self::InvalidDomain => write!(f, "Verkkotunnus puuttuu"),
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

/// Build a public GitHub “new issue” URL (for manual follow-up; no token required).
pub fn github_issue_new_url(report: &Report) -> Result<Url, ReportError> {
    let (title, body) = github_issue_content(report);
    let mut url = Url::parse(&format!(
        "https://github.com/{}/issues/new",
        github_repo()
    ))
    .map_err(|error| ReportError::Http(error.to_string()))?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("title", &title);
        pairs.append_pair("body", &body);
    }
    Ok(url)
}

/// POST report to GitHub Issues API, report worker, or local queue.
pub fn submit(report: &Report) -> Result<(), ReportError> {
    let domain = report.domain.trim();
    if domain.is_empty() {
        return Err(ReportError::InvalidDomain);
    }

    let _ = append_local_report_log(report);

    if let Some(token) = github_token() {
        if submit_via_github_api(report, &token).is_ok() {
            return Ok(());
        }
    }

    if let Some(endpoint) = report_endpoint() {
        if submit_via_http_endpoint(report, &endpoint).is_ok() {
            return Ok(());
        }
    }

    if github_token().is_some() || report_endpoint().is_some() {
        return Err(ReportError::Http(
            "Etäpalveluun lähetys epäonnistui; merkintä on tallennettu paikallisesti.".into(),
        ));
    }

    Err(ReportError::MissingEndpoint)
}

fn github_repo() -> String {
    std::env::var("KOTISATAMA_GITHUB_REPO")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_GITHUB_REPO.to_string())
}

fn github_token() -> Option<String> {
    std::env::var("KOTISATAMA_GITHUB_TOKEN")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn report_endpoint() -> Option<String> {
    std::env::var("KOTISATAMA_REPORT_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn github_issue_content(report: &Report) -> (String, String) {
    let title = match report.kind {
        ReportKind::SiteBroken => format!("[Selain] {}", report.domain.trim()),
        ReportKind::SuggestSite => format!("[Ehdotus] {}", report.domain.trim()),
    };

    let mut body = format!("## Verkkotunnus\n{}\n", report.domain.trim());
    if let Some(message) = report.message.as_deref().filter(|m| !m.trim().is_empty()) {
        body.push_str(&format!("\n## Kuvaus\n{message}\n"));
    }
    if let Some(context_url) = report
        .context_url
        .as_deref()
        .filter(|url| !url.trim().is_empty())
    {
        body.push_str(&format!("\n## Konteksti\n`{context_url}`\n"));
    }
    body.push_str("\n---\n*Anonyymi ilmoitus Katselimen Lokikirjasta.*\n");
    (title, body)
}

fn github_labels(report: &Report) -> Vec<&'static str> {
    match report.kind {
        ReportKind::SiteBroken => vec!["bug", "selain", "lokikirja"],
        ReportKind::SuggestSite => vec!["palaute", "whitelist", "lokikirja"],
    }
}

fn submit_via_github_api(report: &Report, token: &str) -> Result<(), ReportError> {
    let (title, body) = github_issue_content(report);
    let payload = serde_json::json!({
        "title": title,
        "body": body,
        "labels": github_labels(report),
    });

    let url = format!("https://api.github.com/repos/{}/issues", github_repo());
    let response = ureq::post(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", GITHUB_API_VERSION)
        .set("Content-Type", "application/json")
        .send_json(payload)
        .map_err(|error| ReportError::Http(error.to_string()))?;

    if response.status() >= 400 {
        return Err(ReportError::Http(format!(
            "GitHub API returned HTTP {}",
            response.status()
        )));
    }
    Ok(())
}

fn submit_via_http_endpoint(report: &Report, endpoint: &str) -> Result<(), ReportError> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(10))
        .build();

    let response = agent
        .post(endpoint)
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

fn append_local_report_log(report: &Report) -> Result<(), ReportError> {
    let path = data_dir().join("reports.jsonl");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| ReportError::Http(error.to_string()))?;
    }
    let line =
        serde_json::to_string(report).map_err(|error| ReportError::Http(error.to_string()))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| ReportError::Http(error.to_string()))?;
    writeln!(file, "{line}").map_err(|error| ReportError::Http(error.to_string()))?;
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
    let line =
        serde_json::to_string(event).map_err(|error| ReportError::Http(error.to_string()))?;
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

    #[test]
    fn github_issue_url_contains_title_and_body() {
        let report = Report {
            kind: ReportKind::SiteBroken,
            domain: "kela.fi".into(),
            message: Some("Sivu jäätyy".into()),
            context_url: Some("https://www.kela.fi/".into()),
        };
        let url = github_issue_new_url(&report).expect("valid url");
        let query = url.query().unwrap_or_default();
        assert!(query.contains("title="));
        assert!(query.contains("body="));
    }

    #[test]
    fn suggest_site_title_prefix() {
        let report = Report {
            kind: ReportKind::SuggestSite,
            domain: "example.fi".into(),
            message: None,
            context_url: None,
        };
        let (title, _) = github_issue_content(&report);
        assert!(title.starts_with("[Ehdotus]"));
    }
}
