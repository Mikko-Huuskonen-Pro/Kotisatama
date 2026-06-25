/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! User-added domains (local overlay, never synced to CDN).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::WhitelistError;
use crate::domain::normalize_domain;

/// A domain added locally by the user.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct UserWhitelistEntry {
    pub domain: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added: Option<String>,
}

/// Local user overlay (`user-whitelist.json` in app data dir).
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct UserWhitelist {
    pub domains: Vec<UserWhitelistEntry>,
}

impl UserWhitelist {
    pub fn load_from_path(path: &Path) -> Result<Self, WhitelistError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(path).map_err(WhitelistError::Io)?;
        let mut whitelist: UserWhitelist =
            serde_json::from_str(&contents).map_err(WhitelistError::Json)?;
        for entry in &mut whitelist.domains {
            entry.domain = normalize_domain(&entry.domain)?;
        }
        Ok(whitelist)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), WhitelistError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(WhitelistError::Io)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(WhitelistError::Json)?;
        fs::write(path, json).map_err(WhitelistError::Io)
    }

    pub fn domain_hosts(&self) -> Vec<String> {
        self.domains
            .iter()
            .map(|entry| entry.domain.clone())
            .collect()
    }

    pub fn contains_domain(&self, domain: &str) -> bool {
        self.domains
            .iter()
            .any(|entry| entry.domain.eq_ignore_ascii_case(domain))
    }

    pub fn add_domain(
        &mut self,
        domain: &str,
        label: Option<String>,
    ) -> Result<bool, WhitelistError> {
        let domain = normalize_domain(domain)?;
        if self.contains_domain(&domain) {
            return Ok(false);
        }
        self.domains.push(UserWhitelistEntry {
            domain,
            label,
            added: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
        });
        Ok(true)
    }

    pub fn remove_domain(&mut self, domain: &str) -> Result<bool, WhitelistError> {
        let domain = normalize_domain(domain)?;
        let before = self.domains.len();
        self.domains
            .retain(|entry| !entry.domain.eq_ignore_ascii_case(&domain));
        Ok(self.domains.len() < before)
    }
}

/// Path for the user overlay file.
pub fn user_whitelist_path() -> PathBuf {
    data_dir().join("user-whitelist.json")
}

fn data_dir() -> PathBuf {
    std::env::var("KOTISATAMA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("index-data"))
}
