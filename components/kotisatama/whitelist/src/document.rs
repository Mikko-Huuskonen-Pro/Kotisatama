/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Curated whitelist documents (v1 string list, v2 tagged entries, v2.1 categories/types).

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use url::Url;

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

/// Profile tags used for product filtering — never shown in search UI or index keywords.
pub const INTERNAL_PROFILE_TAGS: &[&str] = &["hopeakettu", "lapsi"];

/// Whether `tag` is an internal profile tag (not for public search or display).
pub fn is_internal_tag(tag: &str) -> bool {
    INTERNAL_PROFILE_TAGS
        .iter()
        .any(|internal| internal.eq_ignore_ascii_case(tag))
}

/// Tags safe to index for search and show in the UI (excludes internal profile tags).
pub fn public_tags(tags: &[String]) -> Vec<String> {
    tags.iter()
        .filter(|tag| !is_internal_tag(tag))
        .cloned()
        .collect()
}

/// Tags for search result chips — excludes internal profile tags and region slugs.
pub fn display_tags(tags: &[String], known_region_ids: &HashSet<&str>) -> Vec<String> {
    tags.iter()
        .filter(|tag| !is_internal_tag(tag) && !known_region_ids.contains(tag.as_str()))
        .cloned()
        .collect()
}

/// Maakunnan metatiedot (`regions[]` v2.2).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct RegionMeta {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

impl RegionMeta {
    /// Keywords indexed for regional search (id, label, aliases).
    pub fn search_keywords(&self) -> Vec<String> {
        let mut keywords = vec![self.id.clone(), self.label.clone()];
        keywords.extend(self.aliases.iter().cloned());
        keywords
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub entry_type: Option<String>,
    /// Kanoninen navigointi-URL (esim. `https://www.nordea.fi/` kun apex ei toimi).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_url: Option<String>,
}

impl WhitelistEntry {
    /// URL jolla sivu avataan alias-haussa tai kun apex-domain ei ole oikea lähtöosoite.
    pub fn navigation_url(&self) -> Option<Url> {
        if let Some(entry_url) = self.entry_url.as_deref() {
            if let Ok(url) = Url::parse(entry_url.trim()) {
                if matches!(url.scheme(), "http" | "https") {
                    if let Some(host) = url.host_str() {
                        if host_matches_domain(host, &self.domain) {
                            return Some(url);
                        }
                    }
                }
            }
            log::warn!(
                "Kotisatama whitelist: entry_url ei kelpaa domainille {:?}, käytetään oletusta",
                self.domain
            );
        }
        Url::parse(&format!("https://{}", self.domain)).ok()
    }
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<RegionMeta>,
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

    /// Region metadata by id (`regions[].id`).
    pub fn region_meta(&self, id: &str) -> Option<&RegionMeta> {
        self.regions
            .iter()
            .find(|region| region.id.eq_ignore_ascii_case(id))
    }

    /// Resolve region id from `region` field or legacy region slug in `tags`.
    pub fn resolve_entry_region(&self, entry: &WhitelistEntry) -> Option<String> {
        if let Some(region) = entry.region.as_deref() {
            let region = region.trim();
            if !region.is_empty() {
                return Some(region.to_owned());
            }
        }
        if self.regions.is_empty() {
            return None;
        }
        let region_ids: HashSet<&str> = self.regions.iter().map(|region| region.id.as_str()).collect();
        entry
            .tags
            .iter()
            .find(|tag| region_ids.contains(tag.as_str()))
            .cloned()
    }

    /// Tags shown on search result cards for `entry`.
    pub fn display_tags_for_entry(&self, entry: &WhitelistEntry) -> Vec<String> {
        let region_ids: HashSet<&str> = self.regions.iter().map(|region| region.id.as_str()).collect();
        display_tags(&entry.tags, &region_ids)
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
    #[serde(default)]
    regions: Vec<RegionMeta>,
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
        let region_ids: HashSet<&str> = self.regions.iter().map(|r| r.id.as_str()).collect();

        let mut domains = Vec::with_capacity(self.domains.len());
        for raw in self.domains {
            let entry = match raw {
                RawDomain::Plain(domain) => WhitelistEntry {
                    domain,
                    label: None,
                    category: None,
                    region: None,
                    tags: Vec::new(),
                    entry_type: None,
                    entry_url: None,
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
            if let Some(region) = &entry.region {
                if !region_ids.is_empty() && !region_ids.contains(region.as_str()) {
                    log::warn!(
                        "Kotisatama whitelist: tuntematon region {:?} domainille {:?}",
                        region,
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
            regions: self.regions,
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
    fn navigation_url_uses_entry_url_when_set() {
        let doc = WhitelistDocument::from_json_str(
            r#"{"domains":[{"domain":"nordea.fi","label":"Nordea","entry_url":"https://www.nordea.fi/"}]}"#,
        )
        .unwrap();
        let url = doc.domains[0].navigation_url().unwrap();
        assert_eq!(url.as_str(), "https://www.nordea.fi/");
    }

    #[test]
    fn navigation_url_falls_back_to_https_domain() {
        let doc = WhitelistDocument::from_json_str(r#"{"domains":[{"domain":"kela.fi"}]}"#).unwrap();
        let url = doc.domains[0].navigation_url().unwrap();
        assert_eq!(url.as_str(), "https://kela.fi/");
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

    #[test]
    fn public_tags_excludes_internal_profile_tags() {
        let tags = vec![
            "golf".into(),
            "hopeakettu".into(),
            "liikunta".into(),
            "lapsi".into(),
        ];
        assert_eq!(
            public_tags(&tags),
            vec!["golf".to_owned(), "liikunta".to_owned()]
        );
        assert!(is_internal_tag("hopeakettu"));
        assert!(is_internal_tag("Lapsi"));
        assert!(!is_internal_tag("golf"));
    }

    #[test]
    fn resolve_entry_region_prefers_field_over_tags() {
        let doc = WhitelistDocument::from_json_str(
            r#"{
              "regions":[{"id":"paijat-hame","label":"Päijät-Häme"}],
              "domains":[{"domain":"example.fi","region":"kanta-hame","tags":["paijat-hame"]}]
            }"#,
        )
        .unwrap();
        let entry = &doc.domains[0];
        assert_eq!(
            doc.resolve_entry_region(entry).as_deref(),
            Some("kanta-hame")
        );
    }

    #[test]
    fn resolve_entry_region_falls_back_to_legacy_tag() {
        let doc = WhitelistDocument::from_json_str(
            r#"{
              "regions":[{"id":"paijat-hame","label":"Päijät-Häme"}],
              "domains":[{"domain":"hartola.fi","tags":["kunta","paijat-hame"]}]
            }"#,
        )
        .unwrap();
        assert_eq!(
            doc.resolve_entry_region(&doc.domains[0]).as_deref(),
            Some("paijat-hame")
        );
        assert_eq!(
            doc.display_tags_for_entry(&doc.domains[0]),
            vec!["kunta".to_owned()]
        );
    }

    #[test]
    fn region_meta_search_keywords_include_aliases() {
        let region = RegionMeta {
            id: "paijat-hame".into(),
            label: "Päijät-Häme".into(),
            aliases: vec!["päijät häme".into()],
        };
        let keywords = region.search_keywords();
        assert!(keywords.contains(&"paijat-hame".to_owned()));
        assert!(keywords.contains(&"Päijät-Häme".to_owned()));
        assert!(keywords.contains(&"päijät häme".to_owned()));
    }
}
