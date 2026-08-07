/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Runtime effective whitelist (curated base ∪ user overlay).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use url::Url;

use crate::WhitelistError;
use crate::document::{WhitelistDocument, WhitelistEntry, WhitelistProfile};
use crate::domain::host_matches_domain;
use crate::user::{UserWhitelist, UserWhitelistEntry, user_whitelist_path};

/// Merged whitelist used for navigation checks.
#[derive(Debug, Clone)]
pub struct EffectiveWhitelist {
    profile: WhitelistProfile,
    base: WhitelistDocument,
    base_hosts: Vec<String>,
    user: UserWhitelist,
}

impl EffectiveWhitelist {
    pub fn new(base: WhitelistDocument, user: UserWhitelist, profile: WhitelistProfile) -> Self {
        let base_hosts = base.domain_hosts_for_profile(&profile);
        Self {
            profile: profile.clone(),
            base,
            base_hosts,
            user,
        }
    }

    pub fn profile(&self) -> &WhitelistProfile {
        &self.profile
    }

    pub fn base_document(&self) -> &WhitelistDocument {
        &self.base
    }

    pub fn base_domain_count(&self) -> usize {
        self.base_hosts.len()
    }

    /// Curated whitelist entry for `host`, if any (for search result enrichment).
    pub fn lookup_curated_entry(&self, host: &str) -> Option<WhitelistEntry> {
        self.base
            .lookup_entry_for_host(host, &self.profile)
            .cloned()
    }

    pub fn user_entries(&self) -> &[UserWhitelistEntry] {
        &self.user.domains
    }

    pub fn is_host_allowed(&self, host: &str) -> bool {
        let host = host.to_ascii_lowercase();
        self.base_hosts
            .iter()
            .chain(self.user.domain_hosts().iter())
            .any(|domain| host_matches_domain(&host, domain))
    }

    pub fn add_user_domain(
        &mut self,
        domain: &str,
        label: Option<String>,
    ) -> Result<bool, WhitelistError> {
        let added = self.user.add_domain(domain, label)?;
        if added {
            self.user.save_to_path(&user_whitelist_path())?;
        }
        Ok(added)
    }

    pub fn remove_user_domain(&mut self, domain: &str) -> Result<bool, WhitelistError> {
        let removed = self.user.remove_domain(domain)?;
        if removed {
            self.user.save_to_path(&user_whitelist_path())?;
        }
        Ok(removed)
    }
}

static EFFECTIVE: OnceLock<Mutex<EffectiveWhitelist>> = OnceLock::new();

    /// Initialize runtime whitelist from curated base path and local user overlay.
    pub fn init(base_path: &Path, profile: WhitelistProfile) -> Result<(), WhitelistError> {
        let base = WhitelistDocument::load_from_path(base_path)?;
        let user = UserWhitelist::load_from_path(&user_whitelist_path())?;
        let effective = EffectiveWhitelist::new(base, user, profile);
        let _ = EFFECTIVE.set(Mutex::new(effective));
        Ok(())
    }

    /// Reinitialize whitelist for a new profile (called after profile switch).
    ///
    /// Reads profile-aware candidates again and swaps the effective whitelist so
    /// navigation checks reflect the new profile without app restart.
    pub fn reload_for_profile(
        cdn_cache: Option<PathBuf>,
        profile: WhitelistProfile,
    ) -> Result<(), WhitelistError> {
        // Do not call init_with_fallback here: EFFECTIVE is already set (OnceLock).
        let base_path = crate::resolve::curated_whitelist_candidates_for_profile(
            cdn_cache,
            &profile,
        )
        .into_iter()
        .find(|path| path.is_file())
        .ok_or(WhitelistError::NoBaseListFound)?;

        let base = WhitelistDocument::load_from_path(&base_path)?;
        let user = UserWhitelist::load_from_path(&user_whitelist_path())?;
        let effective = EffectiveWhitelist::new(base, user, profile);
        if let Some(guard) = EFFECTIVE.get() {
            if let Ok(mut existing) = guard.lock() {
                *existing = effective;
                log::info!(
                    "Kotisatama: whitelist reloaded from {} ({} domains)",
                    base_path.display(),
                    existing.base_domain_count()
                );
                return Ok(());
            }
        }
        let _ = EFFECTIVE.set(Mutex::new(effective));
        Ok(())
    }

/// Install an empty effective whitelist (fallback when base file is missing).
pub fn init_empty(profile: WhitelistProfile) -> Result<(), WhitelistError> {
    let base = WhitelistDocument {
        version: None,
        updated: None,
        description: None,
        categories: Vec::new(),
        types: Vec::new(),
        domains: Vec::new(),
    };
    let user = UserWhitelist::load_from_path(&user_whitelist_path())?;
    let effective = EffectiveWhitelist::new(base, user, profile);
    let _ = EFFECTIVE.set(Mutex::new(effective));
    Ok(())
}

fn with_effective<R>(f: impl FnOnce(&EffectiveWhitelist) -> R) -> Option<R> {
    EFFECTIVE
        .get()
        .and_then(|guard| guard.lock().ok().map(|effective| f(&effective)))
}

fn with_effective_mut<R>(f: impl FnOnce(&mut EffectiveWhitelist) -> R) -> Option<R> {
    EFFECTIVE
        .get()
        .and_then(|guard| guard.lock().ok().map(|mut effective| f(&mut effective)))
}

/// Whether navigation to `url` is allowed under the effective whitelist.
pub fn is_navigation_allowed(url: &Url) -> bool {
    if is_internal_navigation_url(url) {
        return true;
    }
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };
    // KOTISATAMA-PATCH: Lapsi ei koskaan pääse online-Wikipediaan — 儿童配置文件永不允许在线Wikipedia。
    if matches!(crate::profile::current_profile(), crate::profile::Profile::Lapsi)
        && is_wikipedia_host(host)
    {
        return false;
    }
    with_effective(|effective| effective.is_host_allowed(host)).unwrap_or(false)
}

fn is_wikipedia_host(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    host == "wikipedia.org"
        || host.ends_with(".wikipedia.org")
        || host == "wikimedia.org"
        || host.ends_with(".wikimedia.org")
}

/// User-added whitelist entries for UI.
pub fn user_entries() -> Vec<UserWhitelistEntry> {
    with_effective(|effective| effective.user_entries().to_vec()).unwrap_or_default()
}

/// Curated whitelist entry for `host` (search UI metadata lookup).
pub fn lookup_curated_entry(host: &str) -> Option<WhitelistEntry> {
    with_effective(|effective| effective.lookup_curated_entry(host))
        .unwrap_or(None)
}

/// Active curated whitelist document (categories, types, domains).
pub fn curated_document() -> Option<WhitelistDocument> {
    with_effective(|effective| effective.base_document().clone())
}

/// Add a user domain to the local overlay.
pub fn add_user_domain(domain: &str, label: Option<String>) -> Result<bool, WhitelistError> {
    with_effective_mut(|effective| effective.add_user_domain(domain, label))
        .ok_or_else(|| WhitelistError::NotInitialized)?
}

/// Remove a user domain from the local overlay.
pub fn remove_user_domain(domain: &str) -> Result<bool, WhitelistError> {
    with_effective_mut(|effective| effective.remove_user_domain(domain))
        .ok_or_else(|| WhitelistError::NotInitialized)?
}

fn is_internal_navigation_url(url: &Url) -> bool {
    matches!(url.scheme(), "about" | "data" | "servo")
}
