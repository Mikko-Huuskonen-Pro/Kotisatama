//! ContentBlockingService — julkinen palvelu.

use crate::RequestBlocker;
use crate::adblock_adapter::AdblockEngine;
use crate::decision::BlockingDecision;
use crate::exceptions::SiteExceptionStore;
use crate::filter_store::FilterListStore;
use crate::request::BlockingRequest;
use crate::statistics::BlockingStatistics;

/// Suodatuksen tila UI:lle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentBlockingStatus {
    /// Moottori valmis, suojaus käytössä (ellei sivustopoikkeus).
    Active,
    /// Alustus epäonnistui tai lista puuttuu — fail-open.
    Inactive,
}

/// Katselimen suodatuspalvelu.
pub struct ContentBlockingService {
    engine: Option<AdblockEngine>,
    exceptions: SiteExceptionStore,
    stats: BlockingStatistics,
    status: ContentBlockingStatus,
}

impl ContentBlockingService {
    /// Tyhjä palvelu: kaikki sallitaan (fail-open).
    pub fn inactive() -> Self {
        Self {
            engine: None,
            exceptions: SiteExceptionStore::new(),
            stats: BlockingStatistics::new(),
            status: ContentBlockingStatus::Inactive,
        }
    }

    /// Rakenna moottori säännöistä (testit / pieni lista).
    pub fn from_rules(rules: &str) -> Self {
        Self {
            engine: Some(AdblockEngine::from_filter_list(rules)),
            exceptions: SiteExceptionStore::new(),
            stats: BlockingStatistics::new(),
            status: ContentBlockingStatus::Active,
        }
    }

    /// Lataa suodatinlista: `KOTISATAMA_FILTER_LIST_PATH` → upotettu lista → fail-open.
    pub fn from_bundled_filters() -> Self {
        if let Ok(path) = std::env::var("KOTISATAMA_FILTER_LIST_PATH") {
            let store = FilterListStore::from_path(path);
            match store.load() {
                Ok(rules) => {
                    log::info!(
                        "kotisatama-content-blocking: lista env-polusta ({})",
                        store.path().display()
                    );
                    return Self::from_rules(&rules);
                },
                Err(err) => {
                    log::warn!(
                        "kotisatama-content-blocking: env-listaa ei voitu lukea ({}): {err}; kokeillaan upotettua",
                        store.path().display()
                    );
                },
            }
        }

        let rules = crate::filter_store::BUNDLED_FILTERS;
        if rules.trim().is_empty() {
            log::warn!("kotisatama-content-blocking: upotettu lista on tyhjä");
            Self::inactive()
        } else {
            Self::from_rules(rules)
        }
    }

    pub fn status(&self) -> ContentBlockingStatus {
        self.status
    }

    pub fn exceptions(&self) -> &SiteExceptionStore {
        &self.exceptions
    }

    pub fn statistics(&self) -> &BlockingStatistics {
        &self.stats
    }

    pub fn reset_page_stats(&self) {
        self.stats.reset_page();
    }
}

impl RequestBlocker for ContentBlockingService {
    fn check(&self, request: &BlockingRequest<'_>) -> BlockingDecision {
        let Some(engine) = &self.engine else {
            return BlockingDecision::Allow;
        };

        if self.exceptions.is_allowed(request.source_url) {
            return BlockingDecision::Allow;
        }

        let decision = engine.check(request);
        if decision == BlockingDecision::Block {
            self.stats.record_block();
        }
        decision
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::ResourceType;

    const TEST_RULES: &str = "\
-advertisement-icon.
-advertisement.
||doubleclick.net^
";

    #[test]
    fn blocks_known_ad_url() {
        let service = ContentBlockingService::from_rules(TEST_RULES);
        assert_eq!(service.status(), ContentBlockingStatus::Active);

        let req = BlockingRequest {
            url: "http://example.com/-advertisement-icon.png",
            source_url: "http://example.com/page",
            resource_type: ResourceType::Image,
        };
        assert_eq!(service.check(&req), BlockingDecision::Block);
        assert!(service.statistics().blocked_count() >= 1);
    }

    #[test]
    fn allows_first_party_resource() {
        let service = ContentBlockingService::from_rules(TEST_RULES);
        let req = BlockingRequest {
            url: "http://example.com/logo.png",
            source_url: "http://example.com/page",
            resource_type: ResourceType::Image,
        };
        assert_eq!(service.check(&req), BlockingDecision::Allow);
    }

    #[test]
    fn site_exception_allows_otherwise_blocked() {
        let service = ContentBlockingService::from_rules(TEST_RULES);
        service.exceptions().allow_site("example.com");
        let req = BlockingRequest {
            url: "http://cdn.example.com/-advertisement-icon.png",
            source_url: "http://www.example.com/page",
            resource_type: ResourceType::Image,
        };
        assert_eq!(service.check(&req), BlockingDecision::Allow);
    }

    #[test]
    fn invalid_url_fail_open() {
        let service = ContentBlockingService::from_rules(TEST_RULES);
        let req = BlockingRequest {
            url: "not a url",
            source_url: "http://example.com/",
            resource_type: ResourceType::Other,
        };
        assert_eq!(service.check(&req), BlockingDecision::Allow);
    }

    #[test]
    fn inactive_service_allows_all() {
        let service = ContentBlockingService::inactive();
        let req = BlockingRequest {
            url: "http://example.com/-advertisement-icon.png",
            source_url: "http://example.com/",
            resource_type: ResourceType::Image,
        };
        assert_eq!(service.check(&req), BlockingDecision::Allow);
        assert_eq!(service.status(), ContentBlockingStatus::Inactive);
    }

    #[test]
    fn bundled_filters_load() {
        let service = ContentBlockingService::from_bundled_filters();
        assert_eq!(service.status(), ContentBlockingStatus::Active);
        let req = BlockingRequest {
            url: "https://page.test/-advertisement-icon.",
            source_url: "https://page.test/",
            resource_type: ResourceType::Image,
        };
        assert_eq!(service.check(&req), BlockingDecision::Block);
    }

    #[test]
    fn bundled_filters_block_eff_simulators() {
        let service = ContentBlockingService::from_bundled_filters();
        assert_eq!(service.status(), ContentBlockingStatus::Active);
        for url in [
            "https://trackersimulator.org/?action=tracking_tally&ad_url=1",
            "https://eviltracker.net/?action=tracking_tally&trackingserver=1",
        ] {
            let req = BlockingRequest {
                url,
                source_url: "https://coveryourtracks.eff.org/",
                resource_type: ResourceType::Image,
            };
            assert_eq!(
                service.check(&req),
                BlockingDecision::Block,
                "expected block for {url}"
            );
        }
    }

    #[test]
    fn bundled_filters_allow_dnt_domain() {
        let service = ContentBlockingService::from_bundled_filters();
        let req = BlockingRequest {
            url: "https://do-not-tracker.org/?action=tracking_tally&random=1",
            source_url: "https://coveryourtracks.eff.org/",
            resource_type: ResourceType::Script,
        };
        assert_eq!(service.check(&req), BlockingDecision::Allow);
    }
}
