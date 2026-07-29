/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Rikastaa Meilisearch-osumat whitelist 2.1 -metadatalla.

use kotisatama_whitelist::lookup_curated_entry;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{SearchHit, SearchOutcome};

/// Hakutulos whitelist-metadatalla (hakusivun kortti).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnrichedSearchHit {
    pub url: String,
    pub title: String,
    pub label: Option<String>,
    pub domain: Option<String>,
    pub category: Option<String>,
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
    pub tags: Vec<String>,
}

/// Rikastaa yksi Meilisearch-osuma whitelist-lookupilla.
pub fn enrich_hit(hit: &SearchHit) -> EnrichedSearchHit {
    let host = Url::parse(&hit.url)
        .ok()
        .and_then(|url| url.host_str().map(str::to_ascii_lowercase));
    let curated = host.as_deref().and_then(lookup_curated_entry);

    EnrichedSearchHit {
        url: hit.url.clone(),
        title: hit.title.clone(),
        label: curated.as_ref().and_then(|entry| entry.label.clone()),
        domain: host.or_else(|| curated.as_ref().map(|entry| entry.domain.clone())),
        category: curated.as_ref().and_then(|entry| entry.category.clone()),
        entry_type: curated.as_ref().and_then(|entry| entry.entry_type.clone()),
        tags: curated.map(|entry| entry.tags).unwrap_or_default(),
    }
}

/// Rikastaa kaikki osumat.
pub fn enrich_hits(hits: &[SearchHit]) -> Vec<EnrichedSearchHit> {
    hits.iter().map(enrich_hit).collect()
}

/// Rikastaa hakutuloksen.
pub fn enrich_outcome(outcome: &SearchOutcome) -> EnrichedSearchOutcome {
    match outcome {
        SearchOutcome::Hits(hits) => EnrichedSearchOutcome::Hits(enrich_hits(hits)),
        SearchOutcome::NoResults => EnrichedSearchOutcome::NoResults,
        SearchOutcome::Error(message) => EnrichedSearchOutcome::Error(message.clone()),
    }
}

/// Rikastettu hakutulos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnrichedSearchOutcome {
    Hits(Vec<EnrichedSearchHit>),
    NoResults,
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use kotisatama_whitelist::WhitelistProfile;

    #[test]
    fn enrich_hit_without_whitelist_uses_title_and_url() {
        let hit = SearchHit {
            id: 1,
            url: "https://www.example.com/elake".into(),
            title: "Eläke".into(),
        };
        let enriched = enrich_hit(&hit);
        assert_eq!(enriched.title, "Eläke");
        assert_eq!(enriched.url, "https://www.example.com/elake");
        assert!(enriched.label.is_none());
        assert!(enriched.category.is_none());
        assert!(enriched.entry_type.is_none());
        assert!(enriched.tags.is_empty());
    }

    #[test]
    fn enrich_hit_uses_whitelist_metadata_when_initialized() {
        let json = r#"{
          "categories":[{"id":"health","label":"Terveys","icon":"health"}],
          "types":[{"id":"white","label":"Valkoinen","icon":"white"}],
          "domains":[{"domain":"kela.fi","label":"Kela","category":"health","type":"white","tags":["eläke"]}]
        }"#;
        let temp = std::env::temp_dir().join("kotisatama-whitelist-enrich-test.json");
        std::fs::write(&temp, json).unwrap();
        kotisatama_whitelist::init(&temp, WhitelistProfile::Free).unwrap();

        let hit = SearchHit {
            id: 1,
            url: "https://www.kela.fi/elake".into(),
            title: "Eläke - Kela".into(),
        };
        let enriched = enrich_hit(&hit);
        assert_eq!(enriched.label.as_deref(), Some("Kela"));
        assert_eq!(enriched.category.as_deref(), Some("health"));
        assert_eq!(enriched.entry_type.as_deref(), Some("white"));
        assert_eq!(enriched.tags, vec!["eläke"]);
        let _ = std::fs::remove_file(temp);
    }
}
