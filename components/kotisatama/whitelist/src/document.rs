/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Curated whitelist documents (v1 string list, v2 tagged entries, v2.1 categories/types).

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::WhitelistError;
use crate::domain::{host_matches_domain, normalize_domain};

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

    pub(crate) fn matches_entry(&self, entry: &WhitelistEntry) -> bool {
        match self {
            Self::Free => true,
            Self::Tagged(required) => entry
                .tags
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(required)),
        }
    }
}

/// Toimialan metatiedot (`categories[]` v2.1).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CategoryMeta {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub icon: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
}

/// Väri-/tyypin metatiedot (`types[]` v2.1).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TypeMeta {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub icon: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
}

/// A single curated whitelist entry (v2+).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct WhitelistEntry {
    pub domain: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<CategoryMeta>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<TypeMeta>,
    pub domains: Vec<WhitelistEntry>,
}

impl WhitelistDocument {
    /// Load curated whitelist JSON from disk.
    pub fn load_from_path(path: &Path) -> Result<Self, WhitelistError> {
        let contents = fs::read_to_string(path).map_err(WhitelistError::Io)?;
        Self::from_json_str(&contents)
    }

    /// Parse curated whitelist JSON (v1 string list or v2+ entry objects).
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

    /// Find curated entry whose domain matches `host` (subdomains included).
    pub fn lookup_entry_for_host(
        &self,
        host: &str,
        profile: &WhitelistProfile,
    ) -> Option<&WhitelistEntry> {
        let host = host.to_ascii_lowercase();
        self.domains
            .iter()
            .filter(|entry| profile.matches_entry(entry))
            .find(|entry| host_matches_domain(&host, &entry.domain))
    }

    /// Category metadata by id (`categories[].id`).
    pub fn category_meta(&self, id: &str) -> Option<&CategoryMeta> {
        self.categories
            .iter()
            .find(|category| category.id.eq_ignore_ascii_case(id))
    }

    /// Type metadata by id (`types[].id`, e.g. `white` / `yellow`).
    pub fn type_meta(&self, id: &str) -> Option<&TypeMeta> {
        self.types
            .iter()
            .find(|entry_type| entry_type.id.eq_ignore_ascii_case(id))
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
    #[serde(default)]
    categories: Vec<CategoryMeta>,
    #[serde(default)]
    types: Vec<TypeMeta>,
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
        let category_ids: HashSet<&str> = self.categories.iter().map(|c| c.id.as_str()).collect();
        let type_ids: HashSet<&str> = self.types.iter().map(|t| t.id.as_str()).collect();

        let mut domains = Vec::with_capacity(self.domains.len());
        for raw in self.domains {
            let entry = match raw {
                RawDomain::Plain(domain) => WhitelistEntry {
                    domain,
                    label: None,
                    category: None,
                    tags: Vec::new(),
                    entry_type: None,
                },
                RawDomain::Entry(entry) => entry,
            };
            let normalized = match normalize_domain(&entry.domain) {
                Ok(domain) => domain,
                Err(error) => {
                    log::warn!(
                        "Kotisatama whitelist: ohitetaan domain {:?}: {error}",
                        entry.domain
                    );
                    continue;
                },
            };
            if let Some(category) = &entry.category {
                if !category_ids.is_empty() && !category_ids.contains(category.as_str()) {
                    log::warn!(
                        "Kotisatama whitelist: tuntematon category {:?} domainille {:?}",
                        category,
                        normalized
                    );
                }
            }
            if let Some(entry_type) = &entry.entry_type {
                if !type_ids.is_empty() && !type_ids.contains(entry_type.as_str()) {
                    log::warn!(
                        "Kotisatama whitelist: tuntematon type {:?} domainille {:?}",
                        entry_type,
                        normalized
                    );
                }
            }
            domains.push(WhitelistEntry {
                domain: normalized,
                ..entry
            });
        }
        Ok(WhitelistDocument {
            version: self.version,
            updated: self.updated,
            description: self.description,
            categories: self.categories,
            types: self.types,
            domains,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v1_string_domains() {
        let doc = WhitelistDocument::from_json_str(r#"{"domains":["kela.fi","yle.fi"]}"#).unwrap();
        assert_eq!(doc.domains.len(), 2);
        assert_eq!(doc.domains[0].domain, "kela.fi");
        assert!(doc.categories.is_empty());
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
    fn parses_v21_with_categories_types_and_yellow() {
        let doc = WhitelistDocument::from_json_str(
            r#"{
              "version":"2.1",
              "categories":[{"id":"health","label":"Terveys","icon":"health"}],
              "types":[
                {"id":"white","label":"Valkoinen","icon":"white"},
                {"id":"yellow","label":"Keltainen","icon":"yellow"}
              ],
              "domains":[
                {"domain":"kela.fi","label":"Kela","category":"health","type":"white"},
                {"domain":"247apteekkiin.fi","label":"Apteekki","category":"health","type":"yellow"}
              ]
            }"#,
        )
        .unwrap();
        assert_eq!(doc.version.as_deref(), Some("2.1"));
        assert_eq!(doc.categories.len(), 1);
        assert_eq!(doc.types.len(), 2);
        assert_eq!(doc.domains[1].entry_type.as_deref(), Some("yellow"));
        assert_eq!(doc.domains[0].category.as_deref(), Some("health"));
    }

    #[test]
    fn lookup_entry_for_host_matches_subdomain() {
        let doc = WhitelistDocument::from_json_str(
            r#"{"domains":[{"domain":"kela.fi","label":"Kela","type":"white"}]}"#,
        )
        .unwrap();
        let entry = doc
            .lookup_entry_for_host("www.kela.fi", &WhitelistProfile::Free)
            .unwrap();
        assert_eq!(entry.label.as_deref(), Some("Kela"));
    }

    #[test]
    fn category_and_type_meta_lookup() {
        let doc = WhitelistDocument::from_json_str(
            r#"{
              "categories":[{"id":"health","label":"Terveys","icon":"health"}],
              "types":[{"id":"white","label":"Valkoinen","icon":"white"}],
              "domains":[{"domain":"kela.fi"}]
            }"#,
        )
        .unwrap();
        assert_eq!(doc.category_meta("health").unwrap().icon, "health");
        assert_eq!(doc.type_meta("white").unwrap().label, "Valkoinen");
    }

    #[test]
    fn skips_invalid_curated_entries_without_dropping_whole_list() {
        let doc =
            WhitelistDocument::from_json_str(r#"{"domains":["kela.fi","not-a-domain","yle.fi"]}"#)
                .unwrap();
        let hosts = doc.domain_hosts_for_profile(&WhitelistProfile::Free);
        assert_eq!(hosts, vec!["kela.fi", "yle.fi"]);
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
