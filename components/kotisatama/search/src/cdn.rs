/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Fetch whitelist and index dump from Kotisatama CDN for OTA updates.

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use log::{info, warn};

use crate::SearchError;

/// Result of a CDN sync attempt.
#[derive(Debug, Clone, Default)]
pub struct CdnSyncReport {
    pub whitelist_updated: bool,
    pub index_dump_updated: bool,
}

/// Download `/free/whitelist.json` and `/free/index.dump` from CDN base URL.
pub fn sync_from_cdn(base_url: &str) -> Result<CdnSyncReport, SearchError> {
    let base = base_url.trim_end_matches('/');
    let cache_dir = PathBuf::from("index-data/cache");
    fs::create_dir_all(&cache_dir).map_err(SearchError::Io)?;

    let mut report = CdnSyncReport::default();

    let whitelist_url = format!("{base}/free/whitelist.json");
    let whitelist_dest = cache_dir.join("whitelist.json");
    match fetch_to_file(&whitelist_url, &whitelist_dest) {
        Ok(()) => {
            info!("Kotisatama CDN: updated whitelist from {whitelist_url}");
            report.whitelist_updated = true;
        },
        Err(error) => {
            warn!("Kotisatama CDN: whitelist fetch failed: {error}");
        },
    }

    let dump_url = format!("{base}/free/index.dump");
    let dump_dest = PathBuf::from("index-data/index.dump");
    if let Some(parent) = dump_dest.parent() {
        fs::create_dir_all(parent).map_err(SearchError::Io)?;
    }
    match fetch_to_file(&dump_url, &dump_dest) {
        Ok(()) => {
            info!("Kotisatama CDN: updated index dump from {dump_url}");
            report.index_dump_updated = true;
        },
        Err(error) => {
            warn!("Kotisatama CDN: index dump fetch failed: {error}");
        },
    }

    Ok(report)
}

/// Cached whitelist path after successful CDN sync.
pub fn cached_whitelist_path() -> Option<PathBuf> {
    let path = PathBuf::from("index-data/cache/whitelist.json");
    path.is_file().then_some(path)
}

fn fetch_to_file(url: &str, dest: &Path) -> Result<(), SearchError> {
    let response = ureq::get(url).call().map_err(|error| SearchError::Http(error.to_string()))?;
    if response.status() != 200 {
        return Err(SearchError::Http(format!(
            "GET {url} returned HTTP {}",
            response.status()
        )));
    }

    let mut reader = response.into_reader();
    let mut file = File::create(dest).map_err(SearchError::Io)?;
    io::copy(&mut reader, &mut file).map_err(SearchError::Io)?;
    Ok(())
}
