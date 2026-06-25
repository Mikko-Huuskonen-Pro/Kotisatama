/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Runtime effective whitelist (curated base ∪ user overlay).

use std::path::Path;
use std::sync::{Mutex, OnceLock};

use url::Url;

use crate::WhitelistError;
use crate::document::{WhitelistDocument, WhitelistProfile};
use crate::domain::host_matches_domain;
use crate::user::{UserWhitelist, UserWhitelistEntry, user_whitelist_path};

/// Merged whitelist used for navigation checks.
#[derive(Debug, Clone)]
pub struct EffectiveWhitelist {
    profile: WhitelistProfile,
    base_hosts: Vec<String>,
    user: UserWhitelist,
}

impl EffectiveWhitelist {
    pub fn new(base: WhitelistDocument, user: UserWhitelist, profile: WhitelistProfile) -> Self {
        Self {
            profile: profile.clone(),
            base_hosts: base.domain_hosts_for_profile(&profile),
            user,
        }
    }

    pub fn profile(&self) -> &WhitelistProfile {
        &self.profile
    }

    pub fn base_domain_count(&self) -> usize {
        self.base_hosts.len()
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

/// Install an empty effective whitelist (fallback when base file is missing).
pub fn init_empty(profile: WhitelistProfile) -> Result<(), WhitelistError> {
    let base = WhitelistDocument {
        version: None,
        updated: None,
        description: None,
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
    if is_internal_navigation_url(url) || is_avomeri_gateway(url) {
        return true;
    }
    let host = match url.host_str() {
        Some(host) => host,
        None => return false,
    };
    with_effective(|effective| effective.is_host_allowed(host)).unwrap_or(false)
}

/// User-added whitelist entries for UI.
pub fn user_entries() -> Vec<UserWhitelistEntry> {
    with_effective(|effective| effective.user_entries().to_vec()).unwrap_or_default()
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

fn is_avomeri_gateway(url: &Url) -> bool {
    url.host_str().map(is_startpage_host).unwrap_or(false)
}

fn is_startpage_host(host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    host == "startpage.com" || host == "www.startpage.com" || host.ends_with(".startpage.com")
}
