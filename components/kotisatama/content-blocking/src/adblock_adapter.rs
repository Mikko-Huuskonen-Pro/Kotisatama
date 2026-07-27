//! Ainoa paikka joka tuntee `adblock`-craten.

use std::sync::Mutex;

use adblock::{
    Engine,
    lists::{FilterSet, ParseOptions},
    request::Request,
};

use crate::decision::BlockingDecision;
use crate::request::BlockingRequest;

pub struct AdblockEngine {
    inner: Mutex<Engine>,
}

impl AdblockEngine {
    pub fn from_filter_list(rules: &str) -> Self {
        let mut filter_set = FilterSet::new(false);
        filter_set.add_filter_list(rules.to_string(), ParseOptions::default());
        let engine = Engine::new_with_filter_set(filter_set);
        Self {
            inner: Mutex::new(engine),
        }
    }

    pub fn check(&self, request: &BlockingRequest<'_>) -> BlockingDecision {
        let Ok(adblock_req) = Request::new(
            request.url,
            request.source_url,
            request.resource_type.as_adblock_cpt(),
            "get",
        ) else {
            // Virheellinen URL → fail-open
            log::debug!(
                "kotisatama-content-blocking: URL ei jäsenny, sallitaan: {}",
                request.url
            );
            return BlockingDecision::Allow;
        };

        let Ok(engine) = self.inner.lock() else {
            log::warn!("kotisatama-content-blocking: moottorilukko myrkyttynyt, fail-open");
            return BlockingDecision::Allow;
        };

        let result = engine.check_network_request(&adblock_req);
        if result.should_block() {
            BlockingDecision::Block
        } else {
            BlockingDecision::Allow
        }
    }
}
