/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Curated whitelist documents (v1 string list or v2 tagged entries).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::domain::normalize_domain;
use crate::WhitelistError;

/// Product profile controlling which curated entries are active.
///
/// `Free` includes every entry in the base document (current default).
/// Tagged profiles (e.g. `hopeakettu`, `lapsi`) require a matching tag on the entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhitelistProfile {
    Free,
    Tagged(String),
}

impl WhitelistProfile {
    /// Resolve profile from `KOTISATAMA_WHITELIST_PROFILE` or default to free/all.
    pub fn current() -> Self {
        match std::env::var("KOTISATAMA_WHITELIST_PROFILE")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
        {
            Some(tag) if tag == "free" => Self::Free,
            Some(tag) => Self::Tagged(tag),
            None => Self::Free,
        }
    }

    fn matches_entry(&self, entry: &WhitelistEntry) -> bool {
        match self {
            Self::Free => true,
            Self::Tagged(required) => entry
                .tags
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(required)),
        }
    }
}

/// A single curated whitelist entry (v2).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WhitelistEntry {
    pub domain: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub entry_type: Option<String>,
}

/// Curated whitelist file (`config/whitelist.json`, CDN `/free/whitelist.json`).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WhitelistDocument {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub domains: Vec<WhitelistEntry>,
}

impl WhitelistDocument {
    /// Load curated whitelist JSON from disk.
    pub fn load_from_path(path: &Path) -> Result<Self, WhitelistError> {
        let contents = fs::read_to_string(path).map_err(WhitelistError::Io)?;
        Self::from_json_str(&contents)
    }

    /// Parse curated whitelist JSON (v1 string list or v2 entry objects).
    pub fn from_json_str(json: &str) -> Result<Self, WhitelistError> {
        let raw: WhitelistDocumentRaw = serde_json::from_str(json).map_err(WhitelistError::Json)?;
        raw.into_document()
    }

    /// Domain hostnames for the given product profile.
    pub fn domain_hosts_for_profile(&self, profile: &WhitelistProfile) -> Vec<String> {
        self.domains
            .iter()
            .filter(|entry| profile.matches_entry(entry))
            .filter_map(|entry| normalize_domain(&entry.domain).ok())
            .collect()
    }

    /// Entries visible for the given product profile (metadata for UI).
    pub fn entries_for_profile(&self, profile: &WhitelistProfile) -> Vec<WhitelistEntry> {
        self.domains
            .iter()
            .filter(|entry| profile.matches_entry(entry))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct WhitelistDocumentRaw {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    updated: Option<String>,
    #[serde(default)]
    description: Option<String>,
    domains: Vec<RawDomain>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawDomain {
    Plain(String),
    Entry(WhitelistEntry),
}

impl WhitelistDocumentRaw {
    fn into_document(self) -> Result<WhitelistDocument, WhitelistError> {
        let mut domains = Vec::with_capacity(self.domains.len());
        for raw in self.domains {
            let entry = match raw {
                RawDomain::Plain(domain) => WhitelistEntry {
                    domain,
                    label: None,
                    tags: Vec::new(),
                    entry_type: None,
                },
                RawDomain::Entry(entry) => entry,
            };
            let normalized = normalize_domain(&entry.domain)?;
            domains.push(WhitelistEntry {
                domain: normalized,
                ..entry
            });
        }
        Ok(WhitelistDocument {
            version: self.version,
            updated: self.updated,
            description: self.description,
            domains,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v1_string_domains() {
        let doc = WhitelistDocument::from_json_str(
            r#"{"domains":["kela.fi","yle.fi"]}"#,
        )
        .unwrap();
        assert_eq!(doc.domains.len(), 2);
        assert_eq!(doc.domains[0].domain, "kela.fi");
    }

    #[test]
    fn parses_v2_tagged_entries() {
        let doc = WhitelistDocument::from_json_str(
            r#"{
              "version":"2.0",
              "domains":[
                {"domain":"kela.fi","label":"Kela","tags":["hopeakettu"],"type":"white"}
              ]
            }"#,
        )
        .unwrap();
        assert_eq!(doc.domains[0].label.as_deref(), Some("Kela"));
        assert_eq!(doc.domains[0].tags, vec!["hopeakettu"]);
    }

    #[test]
    fn free_profile_includes_all_entries() {
        let doc = WhitelistDocument::from_json_str(
            r#"{"domains":[
                {"domain":"kela.fi","tags":["hopeakettu"]},
                {"domain":"pelit.fi","tags":["lapsi"]}
              ]}"#,
        )
        .unwrap();
        let hosts = doc.domain_hosts_for_profile(&WhitelistProfile::Free);
        assert_eq!(hosts, vec!["kela.fi", "pelit.fi"]);
    }

    #[test]
    fn tagged_profile_filters_entries() {
        let doc = WhitelistDocument::from_json_str(
            r#"{"domains":[
                {"domain":"kela.fi","tags":["hopeakettu"]},
                {"domain":"pelit.fi","tags":["lapsi"]}
              ]}"#,
        )
        .unwrap();
        let hosts = doc.domain_hosts_for_profile(&WhitelistProfile::Tagged("hopeakettu".into()));
        assert_eq!(hosts, vec!["kela.fi"]);
    }
}
