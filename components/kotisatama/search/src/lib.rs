/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Local search against a Meilisearch instance (subprocess on `127.0.0.1:7700`).

mod cdn;

use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::json;

pub use cdn::{cached_whitelist_path, sync_from_cdn, CdnSyncReport};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:7700";
const INDEX_UID: &str = "documents";
const HEALTH_POLL_MS: u64 = 100;
const HEALTH_TIMEOUT_SECS: u64 = 30;

fn data_dir() -> PathBuf {
    std::env::var("KOTISATAMA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("index-data"))
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
    process: Option<Child>,
}

impl SearchClient {
    /// Start or attach to Meilisearch and ensure the test index exists.
    pub fn start() -> Result<Self, SearchError> {
        let base_url = std::env::var("KOTISATAMA_MEILISEARCH_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        if is_healthy(&base_url)? {
            let client = Self {
                base_url,
                process: None,
            };
            client.ensure_index()?;
            return Ok(client);
        }

        let binary = find_meilisearch_binary()?;
        let db_path = std::env::var("KOTISATAMA_MEILISEARCH_DB")
            .unwrap_or_else(|_| data_dir().join("meilisearch").to_string_lossy().into_owned());
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

        let process = Command::new(&binary)
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(SearchError::Spawn)?;

        wait_for_health(&base_url)?;

        let client = Self {
            base_url,
            process: Some(process),
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
            Err(ureq::Error::Status(code, resp)) => {
                SearchOutcome::Error(format!(
                    "search failed (HTTP {code}): {}",
                    resp.into_string().unwrap_or_default()
                ))
            },
            Err(error) => SearchOutcome::Error(format!("search request failed: {error}")),
        }
    }

    fn ensure_index(&self) -> Result<(), SearchError> {
        let stats_url = format!("{}/indexes/{}/stats", self.base_url, INDEX_UID);
        if let Ok(resp) = ureq::get(&stats_url).call() && resp.status() == 200 {
            let stats: IndexStats = resp
                .into_json()
                .map_err(|error| SearchError::Http(error.to_string()))?;
            if stats.number_of_documents > 0 {
                return Ok(());
            }
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
            .unwrap_or_else(|_| "config/search-index/documents.json".to_string());
        let contents = fs::read_to_string(&path).map_err(SearchError::Io)?;
        let documents: Vec<SeedDocument> = serde_json::from_str(&contents).map_err(SearchError::Json)?;

        let url = format!("{}/indexes/{}/documents", self.base_url, INDEX_UID);
        ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&documents)
            .map_err(|error| SearchError::Http(error.to_string()))?;
        Ok(())
    }
}

impl Drop for SearchClient {
    fn drop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<SearchHit>,
}

#[derive(Debug, Deserialize)]
struct IndexStats {
    number_of_documents: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct SeedDocument {
    id: u64,
    url: String,
    title: String,
}

#[derive(Debug)]
pub enum SearchError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Spawn(std::io::Error),
    Http(String),
    Timeout,
    BinaryNotFound,
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
        }
    }
}

impl std::error::Error for SearchError {}

fn is_healthy(base_url: &str) -> Result<bool, SearchError> {
    let url = format!("{}/health", base_url);
    match ureq::get(&url).call() {
        Ok(resp) => Ok(resp.status() == 200),
        Err(ureq::Error::Status(404, _)) => Ok(false),
        Err(_) => Ok(false),
    }
}

fn wait_for_health(base_url: &str) -> Result<(), SearchError> {
    let deadline = Instant::now() + Duration::from_secs(HEALTH_TIMEOUT_SECS);
    while Instant::now() < deadline {
        if is_healthy(base_url)? {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(HEALTH_POLL_MS));
    }
    Err(SearchError::Timeout)
}

fn find_meilisearch_binary() -> Result<PathBuf, SearchError> {
    if let Ok(path) = std::env::var("KOTISATAMA_MEILISEARCH_BIN") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }

    if let Ok(path) = which_meilisearch_on_path() {
        return Ok(path);
    }

    Err(SearchError::BinaryNotFound)
}

fn which_meilisearch_on_path() -> Result<PathBuf, SearchError> {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(if cfg!(windows) {
            "meilisearch.exe"
        } else {
            "meilisearch"
        });
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(SearchError::BinaryNotFound)
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
}
