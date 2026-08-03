//! Kotisatama content blocking — adapter adblock-Katselin-forkin ympärillä.
//!
//! Muu koodi käyttää vain [`RequestBlocker`] / [`ContentBlockingService`]-rajapintaa.
//! `adblock`-tyypit elävät vain [`adblock_adapter`]-moduulissa.

mod adblock_adapter;
mod decision;
mod exceptions;
mod filter_store;
mod request;
mod service;
mod statistics;

pub use decision::BlockingDecision;
pub use exceptions::SiteExceptionStore;
pub use filter_store::{FilterListStore, BUNDLED_FILTERS};
pub use request::{BlockingRequest, ResourceType};
pub use service::{ContentBlockingService, ContentBlockingStatus};
pub use statistics::BlockingStatistics;

/// Julkinen estorajapinta (Servo-hook ja testit).
pub trait RequestBlocker {
    fn check(&self, request: &BlockingRequest<'_>) -> BlockingDecision;
}
