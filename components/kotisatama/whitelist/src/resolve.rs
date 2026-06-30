/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Curated whitelist path resolution and fail-safe initialization.

use std::path::{Path, PathBuf};

use crate::WhitelistError;
use crate::document::WhitelistProfile;
use crate::state::init;
use log::{info, warn};

/// Candidate paths for the curated base whitelist, highest priority first.
pub fn curated_whitelist_candidates(cdn_cache: Option<PathBuf>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut seen = Vec::new();

    let mut push = |path: PathBuf| {
        if path.as_os_str().is_empty() {
            return;
        }
        if seen.iter().any(|existing: &PathBuf| existing == &path) {
            return;
        }
        seen.push(path.clone());
        candidates.push(path);
    };

    if let Some(path) = cdn_cache {
        push(path);
    }

    if let Ok(path) = std::env::var("KOTISATAMA_WHITELIST_PATH") {
        push(PathBuf::from(path));
    }

    push(PathBuf::from("config/whitelist.json"));

    if let Some(path) = packaged_relative("config/whitelist.json") {
        push(path);
    }
    if let Some(path) = packaged_relative("whitelist.json") {
        push(path);
    }

    candidates
}

/// Load curated whitelist from the first readable candidate; never installs an empty base list.
pub fn init_with_fallback(
    cdn_cache: Option<PathBuf>,
    profile: WhitelistProfile,
) -> Result<PathBuf, WhitelistError> {
    for path in curated_whitelist_candidates(cdn_cache) {
        if !path.is_file() {
            continue;
        }
        match init(&path, profile.clone()) {
            Ok(()) => {
                info!(
                    "Kotisatama: whitelist loaded from {} ({} domains)",
                    path.display(),
                    domain_count_hint(&path, &profile)
                );
                return Ok(path);
            },
            Err(error) => {
                warn!(
                    "Kotisatama: could not load whitelist from {}: {error}",
                    path.display()
                );
            },
        }
    }

    Err(WhitelistError::NoBaseListFound)
}

fn domain_count_hint(path: &Path, profile: &WhitelistProfile) -> usize {
    crate::document::WhitelistDocument::load_from_path(path)
        .map(|doc| doc.domain_hosts_for_profile(profile).len())
        .unwrap_or(0)
}

fn packaged_relative(relative: &str) -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let path = exe_dir.join(relative);
    path.is_file().then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn init_with_fallback_loads_first_valid_candidate() {
        let good = std::env::temp_dir().join("kotisatama-resolve-good.json");
        let mut file = fs::File::create(&good).unwrap();
        writeln!(file, r#"{{"domains":["kela.fi"]}}"#).unwrap();

        let result = init_with_fallback(Some(good.clone()), WhitelistProfile::Free);
        assert_eq!(result.unwrap(), good);
        let _ = fs::remove_file(&good);
    }
}
