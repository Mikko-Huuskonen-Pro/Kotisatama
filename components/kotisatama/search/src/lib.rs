/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Local search against a Meilisearch instance (subprocess on `127.0.0.1:7700`).

mod cdn;
mod cdn_integrity;
mod enrich;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use kotisatama_subprocess_app::{
    HealthCheckConfig, ManagedSubprocess, SubprocessError, find_binary, find_on_path, is_healthy,
    wait_for_health,
};
use kotisatama_whitelist::{
    UserWhitelistEntry, WhitelistEntry, curated_document, user_entries,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use cdn::{CdnSyncReport, cached_whitelist_path, sync_from_cdn};
pub use cdn_integrity::{CdnManifest, sha256_file};
pub use enrich::{
    EnrichedSearchHit, EnrichedSearchOutcome, enrich_hit, enrich_hits, enrich_outcome,
};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:7700";
const INDEX_UID: &str = "documents";
const WIKI_INDEX_FULL: &str = "wiki_fi_full";
const WIKI_INDEX_LAPSI: &str = "wiki_fi_lapsi";
const HEALTH_CONFIG: HealthCheckConfig = HealthCheckConfig {
    health_path: "/health",
    poll_ms: 100,
    timeout_secs: 30,
};

/// Meilisearch wiki index for the active product profile.
pub fn wiki_index_for_current_profile() -> &'static str {
    use kotisatama_whitelist::{Profile, current_profile};
    match current_profile() {
        Profile::Lapsi => WIKI_INDEX_LAPSI,
        Profile::Normi | Profile::Hopeakettu => WIKI_INDEX_FULL,
    }
}

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

/// A single search hit from the local index (sites or Wikipedia).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchHit {
    /// Meilisearch primary key (numeric for sites, string for wiki paragraphs).
    #[serde(deserialize_with = "deserialize_flexible_id")]
    pub id: String,
    pub url: String,
    pub title: String,
    /// `"wikipedia"` for wiki hits; absent/other for Satama sites.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Snippet text (wiki paragraph).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Article slug for offline snapshot (`kotisatama://wiki/{slug}`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<u32>,
}

impl SearchHit {
    pub fn is_wikipedia(&self) -> bool {
        self.source
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("wikipedia"))
    }
}

fn deserialize_flexible_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        other => Ok(other.to_string()),
    }
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

        let dump_dir = std::env::var("KOTISATAMA_MEILISEARCH_DUMP_DIR").unwrap_or_else(|_| {
            data_dir()
                .join("meilisearch-dumps")
                .to_string_lossy()
                .into_owned()
        });
        let snapshot_dir = std::env::var("KOTISATAMA_MEILISEARCH_SNAPSHOT_DIR").unwrap_or_else(|_| {
            data_dir()
                .join("meilisearch-snapshots")
                .to_string_lossy()
                .into_owned()
        });
        let cwd = std::env::var("KOTISATAMA_MEILISEARCH_CWD").unwrap_or_else(|_| {
            data_dir().to_string_lossy().into_owned()
        });
        fs::create_dir_all(&dump_dir).map_err(SearchError::Io)?;
        fs::create_dir_all(&snapshot_dir).map_err(SearchError::Io)?;
        fs::create_dir_all(&cwd).map_err(SearchError::Io)?;

        let dump_path = std::env::var("KOTISATAMA_INDEX_DUMP")
            .unwrap_or_else(|_| data_dir().join("index.dump").to_string_lossy().into_owned());
        let import_dump = should_import_dump(&dump_path, &db_path);

        if import_dump && PathBuf::from(&db_path).exists() {
            fs::remove_dir_all(&db_path).map_err(SearchError::Io)?;
            fs::create_dir_all(&db_path).map_err(SearchError::Io)?;
        }

        // --dump-dir/--snapshot-dir/--no-analytics + writable cwd: Android bionic (M1).
        let mut args = vec![
            "--http-addr".to_string(),
            "127.0.0.1:7700".to_string(),
            "--db-path".to_string(),
            db_path.clone(),
            "--dump-dir".to_string(),
            dump_dir,
            "--snapshot-dir".to_string(),
            snapshot_dir,
            "--env".to_string(),
            "development".to_string(),
            "--no-analytics".to_string(),
        ];
        if import_dump {
            args.push("--import-dump".to_string());
            args.push(dump_path.clone());
            args.push("--ignore-missing-dump".to_string());
        }

        let child = Command::new(&binary)
            .args(&args)
            .current_dir(&cwd)
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
        if import_dump {
            record_imported_dump(&dump_path, &db_path);
        } else if PathBuf::from(&dump_path).is_file() && !wiki_indexes_ready(&client.base_url) {
            log::warn!(
                "Kotisatama search: wiki indexes missing; clear app data or reinstall to reimport dump"
            );
        }
        Ok(client)
    }

    /// Search sites + Wikipedia indexes (wiki hits first).
    pub fn search(&self, query: &str) -> SearchOutcome {
        let query = query.trim();
        if query.is_empty() {
            return SearchOutcome::NoResults;
        }

        let wiki_index = wiki_index_for_current_profile();
        let mut wiki_hits = self.search_index(wiki_index, query, 12);
        let site_hits = self.search_index(INDEX_UID, query, 25);

        // Mark wiki source if Meilisearch omitted it.
        for hit in &mut wiki_hits {
            if hit.source.is_none() {
                hit.source = Some("wikipedia".into());
            }
        }

        let mut seen = HashSet::new();
        let mut merged = Vec::with_capacity(wiki_hits.len() + site_hits.len());
        for hit in wiki_hits.into_iter().chain(site_hits) {
            let key = hit.url.to_ascii_lowercase();
            if seen.insert(key) {
                merged.push(hit);
            }
        }

        if merged.is_empty() {
            SearchOutcome::NoResults
        } else {
            SearchOutcome::Hits(merged)
        }
    }

    fn search_index(&self, index: &str, query: &str, limit: usize) -> Vec<SearchHit> {
        let url = format!("{}/indexes/{index}/search", self.base_url);
        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(json!({ "q": query, "limit": limit }));

        match response {
            Ok(resp) => {
                let body: SearchResponse = match resp.into_json() {
                    Ok(body) => body,
                    Err(error) => {
                        log::warn!("Kotisatama search: invalid response from {index}: {error}");
                        return Vec::new();
                    },
                };
                body.hits
            },
            Err(error) => {
                // Wiki indexes may be absent until import — treat as empty.
                log::warn!("Kotisatama search: index {index} unavailable: {error}");
                Vec::new()
            },
        }
    }

    /// KOTISATAMA-PATCH: dump-tuonnin jälkeen lataa seed-dokumentit tyhjästä (korvaa vanhat otsikot) — dump导入后重新加载种子文档以替换旧标题。
    fn ensure_index(&self) -> Result<(), SearchError> {
        let stats_url = format!("{}/indexes/{}/stats", self.base_url, INDEX_UID);
        if let Ok(resp) = ureq::get(&stats_url).call()
            && resp.status() == 200
        {
            return self.load_seed_documents_fresh();
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

        self.load_seed_documents_fresh()?;
        Ok(())
    }

    fn load_seed_documents(&self) -> Result<(), SearchError> {
        self.load_seed_documents_with_clear(false)
    }

    /// Clear the index and reload documents from scratch.
    ///
    /// KOTISATAMA-PATCH: upsert ei korvaa vanhoja otsikoita (esim. yle.fi) — 旧标题不会被upsert覆盖。
    fn load_seed_documents_fresh(&self) -> Result<(), SearchError> {
        self.load_seed_documents_with_clear(true)
    }

    fn load_seed_documents_with_clear(&self, clear_first: bool) -> Result<(), SearchError> {
        let documents = seed_documents()?;
        if documents.is_empty() {
            return Ok(());
        }

        let url = format!("{}/indexes/{}/documents", self.base_url, INDEX_UID);
        if clear_first {
            if let Ok(resp) = ureq::delete(&url).call() {
                if let Ok(task) = resp.into_json::<MeiliTask>() {
                    let _ = self.wait_for_task(task.task_uid);
                }
            }
        }
        let task: MeiliTask = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&documents)
            .map_err(|error| SearchError::Http(error.to_string()))?
            .into_json()
            .map_err(|error| SearchError::Http(error.to_string()))?;
        self.wait_for_task(task.task_uid)?;
        Ok(())
    }

    /// Re-POST seed + curated + user whitelist documents into Meilisearch.
    ///
    /// Call after local Satama add/remove so mid-session search stays in sync.
    /// Clears the index first so removals take effect (POST alone only upserts).
    pub fn reload_seed_documents(&self) -> Result<(), SearchError> {
        let url = format!("{}/indexes/{}/documents", self.base_url, INDEX_UID);
        match ureq::delete(&url).call() {
            Ok(resp) => {
                if let Ok(task) = resp.into_json::<MeiliTask>() {
                    let _ = self.wait_for_task(task.task_uid);
                }
            },
            Err(error) => {
                log::warn!("Kotisatama search: clear documents before reload: {error}");
            },
        }
        self.load_seed_documents()
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

fn seed_documents() -> Result<Vec<SeedDocument>, SearchError> {
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
    Ok(documents)
}

/// In-memory search over the curated seed documents.
///
/// Used when Meilisearch cannot run — e.g. on Android, where the prebuilt
/// Meilisearch binary is dynamically linked against glibc and will not
/// execute under bionic. Matching is intentionally simple (case-insensitive
/// substring over title, URL and keywords) since the curated set is small.
pub fn seed_search(query: &str) -> SearchOutcome {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return SearchOutcome::NoResults;
    }
    let documents = match seed_documents() {
        Ok(documents) => documents,
        Err(error) => {
            log::warn!("Kotisatama search: seed fallback failed: {error}");
            return SearchOutcome::Error(error.to_string());
        },
    };
    let mut scored: Vec<(i32, SearchHit)> = documents
        .iter()
        .filter_map(|doc| {
            let score = seed_match_score(doc, &query)?;
            Some((
                score,
                SearchHit {
                    id: doc.id.to_string(),
                    url: doc.url.clone(),
                    title: doc.title.clone(),
                    source: None,
                    text: None,
                    slug: None,
                    paragraph: None,
                },
            ))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.id.cmp(&b.1.id)));
    let hits: Vec<SearchHit> = scored.into_iter().take(25).map(|(_, hit)| hit).collect();
    if hits.is_empty() {
        SearchOutcome::NoResults
    } else {
        SearchOutcome::Hits(hits)
    }
}

fn seed_match_score(doc: &SeedDocument, query: &str) -> Option<i32> {
    let query = query.to_lowercase();
    let query = query.as_str();
    let title = doc.title.to_lowercase();
    let url = doc.url.to_lowercase();
    let keywords = doc.keywords.as_deref().unwrap_or_default().to_lowercase();
    if title == query {
        Some(400)
    } else if title.starts_with(query) {
        Some(300)
    } else if title.contains(query) {
        Some(200)
    } else if url.contains(query) {
        Some(150)
    } else if keywords.contains(query) {
        Some(100)
    } else {
        None
    }
}

fn append_whitelist_documents(documents: &mut Vec<SeedDocument>) {
    let mut seen_urls = documents
        .iter()
        .map(|document| document.url.to_ascii_lowercase())
        .collect::<HashSet<_>>();

    // KOTISATAMA-PATCH: curated CDN whitelist → hakuindeksi — 策划白名单进入搜索索引。
    if let Some(whitelist) = curated_document() {
        for entry in whitelist.domains {
            let Some(document) =
                whitelist_entry_document(1_000_000 + documents.len() as u64, &entry)
            else {
                continue;
            };
            if seen_urls.insert(document.url.to_ascii_lowercase()) {
                documents.push(document);
            }
        }
    }

    // KOTISATAMA-PATCH: käyttäjän omat Satama-lisäykset hakuun — 用户自有港口条目进入搜索。
    for (index, entry) in user_entries().into_iter().enumerate() {
        let Some(document) = user_entry_document(2_000_000 + index as u64, &entry) else {
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

fn user_entry_document(id: u64, entry: &UserWhitelistEntry) -> Option<SeedDocument> {
    let domain = entry.domain.trim();
    if domain.is_empty() {
        return None;
    }
    let title = entry
        .label
        .as_deref()
        .unwrap_or(domain)
        .trim()
        .to_owned();
    Some(SeedDocument {
        id,
        url: format!("https://{domain}/"),
        title,
        keywords: Some(domain.to_owned()),
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

fn imported_dump_marker_path(db_path: &str) -> PathBuf {
    PathBuf::from(db_path).join(".imported_dump_sha256")
}

fn record_imported_dump(dump_path: &str, db_path: &str) {
    if let Ok(hash) = sha256_file(Path::new(dump_path)) {
        let _ = fs::write(imported_dump_marker_path(db_path), hash);
    }
}

fn wiki_indexes_ready(base_url: &str) -> bool {
    for index in [WIKI_INDEX_FULL, WIKI_INDEX_LAPSI] {
        let url = format!("{base_url}/indexes/{index}/stats");
        let Ok(resp) = ureq::get(&url).call() else {
            return false;
        };
        if resp.status() != 200 {
            return false;
        }
        let Ok(body) = resp.into_json::<serde_json::Value>() else {
            return false;
        };
        if body
            .get("numberOfDocuments")
            .and_then(|value| value.as_u64())
            .unwrap_or(0)
            == 0
        {
            return false;
        }
    }
    true
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
    let current_hash = sha256_file(&dump).ok();
    let marker = imported_dump_marker_path(db_path);
    if !marker.is_file() {
        return true;
    }
    let stored = fs::read_to_string(marker).ok();
    match (current_hash, stored) {
        (Some(current), Some(stored)) => current.trim() != stored.trim(),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(title: &str, url: &str, keywords: Option<&str>) -> SeedDocument {
        SeedDocument {
            id: 1,
            url: url.to_owned(),
            title: title.to_owned(),
            keywords: keywords.map(str::to_owned),
        }
    }

    #[test]
    fn seed_match_score_ranks_title_over_keywords() {
        let by_title = doc("Kela", "https://kela.fi/", None);
        let by_keywords = doc("OmaKanta", "https://omakanta.fi/", Some("kela etuudet"));
        let title_score = seed_match_score(&by_title, "kela").unwrap();
        let keyword_score = seed_match_score(&by_keywords, "kela").unwrap();
        assert!(title_score > keyword_score);
    }

    #[test]
    fn seed_match_score_is_case_insensitive() {
        let document = doc("Kanta.fi", "https://kanta.fi/", None);
        assert!(seed_match_score(&document, "KANTA").is_some());
    }

    #[test]
    fn seed_match_score_misses_unrelated_document() {
        let document = doc("Kela", "https://kela.fi/", Some("eläke toimeentulotuki"));
        assert!(seed_match_score(&document, "golf").is_none());
    }

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

    #[test]
    fn user_entry_document_uses_label_and_stable_url() {
        let entry = UserWhitelistEntry {
            domain: "oma-esimerkki.fi".into(),
            label: Some("Oma esimerkki".into()),
            added: None,
        };
        let document = user_entry_document(2_000_000, &entry).unwrap();
        assert_eq!(document.id, 2_000_000);
        assert_eq!(document.url, "https://oma-esimerkki.fi/");
        assert_eq!(document.title, "Oma esimerkki");
        assert_eq!(document.keywords.as_deref(), Some("oma-esimerkki.fi"));
    }

    #[test]
    fn user_entry_document_falls_back_to_domain_title() {
        let entry = UserWhitelistEntry {
            domain: "pelkka-domain.fi".into(),
            label: None,
            added: None,
        };
        let document = user_entry_document(2_000_001, &entry).unwrap();
        assert_eq!(document.title, "pelkka-domain.fi");
    }
}
