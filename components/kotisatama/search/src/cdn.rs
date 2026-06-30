/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Fetch whitelist and index dump from Kotisatama CDN for OTA updates.

use std::fs;
use std::path::PathBuf;

use log::{info, warn};

use crate::SearchError;
use crate::cdn_integrity::{fetch_manifest, fetch_verify_and_install, skip_integrity_check};

/// Result of a CDN sync attempt.
#[derive(Debug, Clone, Default)]
pub struct CdnSyncReport {
    pub whitelist_updated: bool,
    pub index_dump_updated: bool,
}

/// Download `/free/manifest.json`, then `/free/whitelist.json` and `/free/index.dump` with SHA-256 checks.
pub fn sync_from_cdn(base_url: &str) -> Result<CdnSyncReport, SearchError> {
    let base = base_url.trim_end_matches('/');
    let cache_dir = data_dir().join("cache");
    fs::create_dir_all(&cache_dir).map_err(SearchError::Io)?;

    let manifest = match fetch_manifest(base_url) {
        Ok(manifest) => manifest,
        Err(error) => {
            warn!("Kotisatama CDN: manifest fetch failed: {error}");
            return Ok(CdnSyncReport::default());
        },
    };

    let mut report = CdnSyncReport::default();

    let whitelist_url = format!("{base}/free/whitelist.json");
    let whitelist_dest = cache_dir.join("whitelist.json");
    match fetch_verify_and_install(&whitelist_url, &whitelist_dest, &manifest, "whitelist.json") {
        Ok(()) => {
            info!("Kotisatama CDN: updated whitelist from {whitelist_url}");
            report.whitelist_updated = true;
        },
        Err(error) => {
            warn!("Kotisatama CDN: whitelist update rejected: {error}");
        },
    }

    let dump_url = format!("{base}/free/index.dump");
    let dump_dest = data_dir().join("index.dump");
    if let Some(parent) = dump_dest.parent() {
        fs::create_dir_all(parent).map_err(SearchError::Io)?;
    }
    match fetch_verify_and_install(&dump_url, &dump_dest, &manifest, "index.dump") {
        Ok(()) => {
            info!("Kotisatama CDN: updated index dump from {dump_url}");
            report.index_dump_updated = true;
        },
        Err(error) => {
            warn!("Kotisatama CDN: index dump update rejected: {error}");
        },
    }

    if skip_integrity_check() {
        warn!("Kotisatama CDN: KOTISATAMA_CDN_SKIP_INTEGRITY is set — do not use in production");
    }

    Ok(report)
}

/// Cached whitelist path after successful CDN sync.
pub fn cached_whitelist_path() -> Option<PathBuf> {
    let path = data_dir().join("cache").join("whitelist.json");
    if !path.is_file() {
        return None;
    }
    if skip_integrity_check() {
        return Some(path);
    }
    let manifest_path = data_dir().join("cache").join("manifest.json");
    if !manifest_path.is_file() {
        return None;
    }
    let manifest = crate::cdn_integrity::load_manifest(&manifest_path).ok()?;
    crate::cdn_integrity::verify_manifest_signature(&manifest).ok()?;
    crate::cdn_integrity::verify_file(&manifest, "whitelist.json", &path).ok()?;
    Some(path)
}

fn data_dir() -> PathBuf {
    std::env::var("KOTISATAMA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("index-data"))
}
