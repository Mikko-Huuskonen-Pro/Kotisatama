/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Local search against a Meilisearch instance (subprocess on `127.0.0.1:7700`).

mod cdn;
mod cdn_integrity;
mod enrich;

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use kotisatama_subprocess_app::{
    HealthCheckConfig, ManagedSubprocess, SubprocessError, find_binary, find_on_path, is_healthy,
    wait_for_health,
};
use kotisatama_whitelist::{WhitelistEntry, curated_document};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use cdn::{CdnSyncReport, cached_whitelist_path, sync_from_cdn};
pub use cdn_integrity::{CdnManifest, sha256_file};
pub use enrich::{
    EnrichedSearchHit, EnrichedSearchOutcome, enrich_hit, enrich_hits, enrich_outcome,
};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:7700";
const INDEX_UID: &str = "documents";
const HEALTH_CONFIG: HealthCheckConfig = HealthCheckConfig {
    health_path: "/health",
    poll_ms: 100,
    timeout_secs: 30,
};

fn data_dir() -> PathBuf {
    std::env::var("KOTISATAMA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            packaged_path("index-data").unwrap_or_else(|| PathBuf::from("index-data"))
        })
}

fn packaged_path(relative: &str) -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    Some(exe_dir.join(relative))
}

/// A single search hit from the local index.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchHit {
    pub id: u64,
    pub url: String,
    pub title: String,
}

/// Outcome of a Kotisatama search query.
#[derive(Debug, Clone)]
pub enum SearchOutcome {
    Hits(Vec<SearchHit>),
    NoResults,
    Error(String),
}

/// Meilisearch HTTP client; optionally owns a spawned subprocess.
pub struct SearchClient {
    base_url: String,
    #[allow(dead_code)]
    process: ManagedSubprocess,
}

impl SearchClient {
    /// Start or attach to Meilisearch and ensure the test index exists.
    pub fn start() -> Result<Self, SearchError> {
        let base_url = std::env::var("KOTISATAMA_MEILISEARCH_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        if is_healthy(&base_url, HEALTH_CONFIG.health_path).map_err(map_subprocess_error)? {
            let client = Self {
                base_url,
                process: ManagedSubprocess::detached(),
            };
            client.ensure_index()?;
            return Ok(client);
        }

        let binary = find_meilisearch_binary()?;
        let db_path = std::env::var("KOTISATAMA_MEILISEARCH_DB").unwrap_or_else(|_| {
            data_dir()
                .join("meilisearch")
                .to_string_lossy()
                .into_owned()
        });
        fs::create_dir_all(&db_path).map_err(SearchError::Io)?;

        let dump_path = std::env::var("KOTISATAMA_INDEX_DUMP")
            .unwrap_or_else(|_| data_dir().join("index.dump").to_string_lossy().into_owned());
        let import_dump = should_import_dump(&dump_path, &db_path);

        if import_dump && PathBuf::from(&db_path).exists() {
            fs::remove_dir_all(&db_path).map_err(SearchError::Io)?;
            fs::create_dir_all(&db_path).map_err(SearchError::Io)?;
        }

        let mut args = vec![
            "--http-addr".to_string(),
            "127.0.0.1:7700".to_string(),
            "--db-path".to_string(),
            db_path.clone(),
            "--env".to_string(),
            "development".to_string(),
        ];
        if import_dump {
            args.push("--import-dump".to_string());
            args.push(dump_path);
            args.push("--ignore-missing-dump".to_string());
        }

        let child = Command::new(&binary)
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(SearchError::Spawn)?;

        wait_for_health(&base_url, HEALTH_CONFIG).map_err(map_subprocess_error)?;

        let client = Self {
            base_url,
            process: ManagedSubprocess::from_child(child),
        };
        client.ensure_index()?;
        Ok(client)
    }

    /// Search the local index.
    pub fn search(&self, query: &str) -> SearchOutcome {
        let query = query.trim();
        if query.is_empty() {
            return SearchOutcome::NoResults;
        }

        let url = format!("{}/indexes/{}/search", self.base_url, INDEX_UID);
        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(json!({ "q": query, "limit": 25 }));

        match response {
            Ok(resp) => {
                let body: SearchResponse = match resp.into_json() {
                    Ok(body) => body,
                    Err(error) => {
                        return SearchOutcome::Error(format!("invalid search response: {error}"));
                    },
                };
                if body.hits.is_empty() {
                    SearchOutcome::NoResults
                } else {
                    SearchOutcome::Hits(body.hits)
                }
            },
            Err(ureq::Error::Status(code, resp)) => SearchOutcome::Error(format!(
                "search failed (HTTP {code}): {}",
                resp.into_string().unwrap_or_default()
            )),
            Err(error) => SearchOutcome::Error(format!("search request failed: {error}")),
        }
    }

    fn ensure_index(&self) -> Result<(), SearchError> {
        let stats_url = format!("{}/indexes/{}/stats", self.base_url, INDEX_UID);
        if let Ok(resp) = ureq::get(&stats_url).call()
            && resp.status() == 200
        {
            return self.load_seed_documents();
        }

        let create_url = format!("{}/indexes", self.base_url);
        if let Err(error) = ureq::post(&create_url)
            .set("Content-Type", "application/json")
            .send_json(json!({
                "uid": INDEX_UID,
                "primaryKey": "id"
            }))
        {
            // Index may already exist if stats endpoint failed transiently.
            log::warn!("Kotisatama search: create index: {error}");
        }

        self.load_seed_documents()?;
        Ok(())
    }

    fn load_seed_documents(&self) -> Result<(), SearchError> {
        let path = std::env::var("KOTISATAMA_SEARCH_DOCUMENTS")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                packaged_path("config/search-index/documents.json")
                    .unwrap_or_else(|| PathBuf::from("config/search-index/documents.json"))
            });
        let contents = fs::read_to_string(&path).unwrap_or_else(|error| {
            log::warn!("Kotisatama search: seed documents not found at {path:?}: {error}");
            "[]".to_owned()
        });
        let mut documents: Vec<SeedDocument> =
            serde_json::from_str(&contents).map_err(SearchError::Json)?;
        append_whitelist_documents(&mut documents);
        if documents.is_empty() {
            return Ok(());
        }

        let url = format!("{}/indexes/{}/documents", self.base_url, INDEX_UID);
        let task: MeiliTask = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&documents)
            .map_err(|error| SearchError::Http(error.to_string()))?
            .into_json()
            .map_err(|error| SearchError::Http(error.to_string()))?;
        self.wait_for_task(task.task_uid)?;
        Ok(())
    }

    fn wait_for_task(&self, task_uid: u64) -> Result<(), SearchError> {
        let deadline = Instant::now() + Duration::from_secs(30);
        let url = format!("{}/tasks/{task_uid}", self.base_url);
        loop {
            let task: MeiliTask = ureq::get(&url)
                .call()
                .map_err(|error| SearchError::Http(error.to_string()))?
                .into_json()
                .map_err(|error| SearchError::Http(error.to_string()))?;
            match task.status.as_str() {
                "succeeded" => return Ok(()),
                "failed" | "canceled" => {
                    return Err(SearchError::Http(format!(
                        "meilisearch indexing task {task_uid} {}",
                        task.status
                    )));
                },
                _ if Instant::now() >= deadline => return Err(SearchError::Timeout),
                _ => thread::sleep(Duration::from_millis(100)),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<SearchHit>,
}

#[derive(Debug, Deserialize)]
struct MeiliTask {
    #[serde(alias = "taskUid", alias = "uid")]
    task_uid: u64,
    status: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SeedDocument {
    id: u64,
    url: String,
    title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    keywords: Option<String>,
}

fn append_whitelist_documents(documents: &mut Vec<SeedDocument>) {
    let Some(whitelist) = curated_document() else {
        return;
    };
    let mut seen_urls = documents
        .iter()
        .map(|document| document.url.to_ascii_lowercase())
        .collect::<HashSet<_>>();

    for entry in whitelist.domains {
        let Some(document) = whitelist_entry_document(1_000_000 + documents.len() as u64, &entry)
        else {
            continue;
        };
        if seen_urls.insert(document.url.to_ascii_lowercase()) {
            documents.push(document);
        }
    }
}

fn whitelist_entry_document(id: u64, entry: &WhitelistEntry) -> Option<SeedDocument> {
    let domain = entry.domain.trim();
    if domain.is_empty() {
        return None;
    }

    let title = entry.label.as_deref().unwrap_or(domain).trim().to_owned();
    let mut keywords = vec![domain.to_owned()];
    keywords.extend(entry.tags.iter().cloned());
    if entry
        .tags
        .iter()
        .any(|tag| tag.eq_ignore_ascii_case("golf"))
    {
        keywords.extend(
            [
                "golfkenttä",
                "golfkentät",
                "golfkenttään",
                "golfkentän",
                "golfkenttia",
                "golfkenttiä",
            ]
            .into_iter()
            .map(str::to_owned),
        );
    }

    Some(SeedDocument {
        id,
        url: format!("https://{domain}/"),
        title,
        keywords: Some(keywords.join(" ")),
    })
}

#[derive(Debug)]
pub enum SearchError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Spawn(std::io::Error),
    Http(String),
    Timeout,
    BinaryNotFound,
    Integrity(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::Spawn(e) => write!(f, "failed to start Meilisearch: {e}"),
            Self::Http(msg) => write!(f, "{msg}"),
            Self::Timeout => write!(f, "Meilisearch did not become ready"),
            Self::BinaryNotFound => write!(
                f,
                "meilisearch binary not found (set KOTISATAMA_MEILISEARCH_BIN or install meilisearch)"
            ),
            Self::Integrity(msg) => write!(f, "CDN integrity check failed: {msg}"),
        }
    }
}

impl std::error::Error for SearchError {}

fn find_meilisearch_binary() -> Result<PathBuf, SearchError> {
    if let Ok(path) = find_binary(
        "KOTISATAMA_MEILISEARCH_BIN",
        &["bin/meilisearch.exe", "bin/meilisearch"],
    ) {
        return Ok(path);
    }

    for relative in ["bin/meilisearch.exe", "bin/meilisearch"] {
        if let Some(path) = packaged_path(relative).filter(|path| path.is_file()) {
            return Ok(path);
        }
    }

    find_on_path(&["meilisearch.exe", "meilisearch"]).ok_or(SearchError::BinaryNotFound)
}

fn map_subprocess_error(error: SubprocessError) -> SearchError {
    match error {
        SubprocessError::Io(error) => SearchError::Io(error),
        SubprocessError::BinaryNotFound => SearchError::BinaryNotFound,
        SubprocessError::Timeout => SearchError::Timeout,
        SubprocessError::Http(message) => SearchError::Http(message),
    }
}

fn should_import_dump(dump_path: &str, db_path: &str) -> bool {
    let dump = PathBuf::from(dump_path);
    if !dump.is_file() {
        return false;
    }
    let db = PathBuf::from(db_path);
    if !db.exists() {
        return true;
    }
    let dump_modified = fs::metadata(&dump).and_then(|m| m.modified()).ok();
    let db_modified = fs::metadata(&db).and_then(|m| m.modified()).ok();
    match (dump_modified, db_modified) {
        (Some(d), Some(b)) => d > b,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_document_json_parses() {
        let json = r#"[{"id":1,"url":"https://kela.fi/elake","title":"Eläke"}]"#;
        let docs: Vec<SeedDocument> = serde_json::from_str(json).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].title, "Eläke");
    }

    #[test]
    fn whitelist_entry_document_adds_golf_keywords() {
        let entry = WhitelistEntry {
            domain: "example-golf.fi".into(),
            label: Some("Example Golf".into()),
            category: Some("sports".into()),
            tags: vec!["golf".into()],
            entry_type: Some("yellow".into()),
            entry_url: None,
        };
        let document = whitelist_entry_document(1, &entry).unwrap();
        assert_eq!(document.url, "https://example-golf.fi/");
        assert_eq!(document.title, "Example Golf");
        assert!(document.keywords.unwrap().contains("golfkenttään"));
    }
}
