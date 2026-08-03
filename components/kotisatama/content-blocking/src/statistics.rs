//! Lyhytikäiset estotilastot (muistissa).

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct BlockingStatistics {
    blocked_on_page: AtomicU64,
    blocked_total: AtomicU64,
}

impl BlockingStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_block(&self) {
        self.blocked_on_page.fetch_add(1, Ordering::Relaxed);
        self.blocked_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn blocked_count(&self) -> u64 {
        self.blocked_on_page.load(Ordering::Relaxed)
    }

    pub fn blocked_total(&self) -> u64 {
        self.blocked_total.load(Ordering::Relaxed)
    }

    pub fn reset_page(&self) {
        self.blocked_on_page.store(0, Ordering::Relaxed);
    }
}
