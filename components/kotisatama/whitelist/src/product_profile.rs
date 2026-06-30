/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Product profiles: Avomeri access, user whitelist edits, and whitelist tag selection.

use crate::document::WhitelistProfile;

/// Kotisatama product profile (who is using the browser).
///
/// Distinct from [`WhitelistProfile`], which filters curated `domains[]` by tag.
/// Resolved from `KOTISATAMA_PRODUCT_PROFILE`; defaults to [`Normaali`] (adult v1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductProfile {
    /// Adult / free — Avomeri on, user may extend whitelist locally.
    Normaali,
    /// Hopeakettu subscription — tagged curated list, otherwise like Normaali.
    Hopeakettu,
    /// Junior — Satama only, no Avomeri, no self-service whitelist edits.
    Lapsi,
    /// Senior — Satama by default; Avomeri off until enabled by carer (future setting).
    Seniori,
}

impl ProductProfile {
    /// Resolve from `KOTISATAMA_PRODUCT_PROFILE` (default: `normaali`).
    pub fn current() -> Self {
        match std::env::var("KOTISATAMA_PRODUCT_PROFILE")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .as_deref()
        {
            Some("lapsi") | Some("junior") => Self::Lapsi,
            Some("seniori") | Some("senior") => Self::Seniori,
            Some("hopeakettu") => Self::Hopeakettu,
            Some("normaali") | Some("free") | Some("adult") => Self::Normaali,
            None => Self::Normaali,
            Some(other) => {
                log::warn!(
                    "Kotisatama: tuntematon KOTISATAMA_PRODUCT_PROFILE={other:?}, käytetään normaali"
                );
                Self::Normaali
            },
        }
    }

    /// Whether the user may open Avomeri (open web via explicit confirmation).
    pub fn can_enter_avomeri(self) -> bool {
        match self {
            Self::Normaali | Self::Hopeakettu => true,
            Self::Seniori => seniori_avomeri_enabled(),
            Self::Lapsi => false,
        }
    }

    /// Whether the user may add domains to the local overlay (`servo:whitelist/add`).
    pub fn can_add_user_domain(self) -> bool {
        matches!(self, Self::Normaali | Self::Hopeakettu)
    }

    /// Curated whitelist tag filter for this product profile.
    pub fn whitelist_profile(self) -> WhitelistProfile {
        match self {
            Self::Normaali => WhitelistProfile::Free,
            Self::Hopeakettu => WhitelistProfile::Tagged("hopeakettu".into()),
            Self::Lapsi => WhitelistProfile::Tagged("lapsi".into()),
            Self::Seniori => WhitelistProfile::Tagged("seniori".into()),
        }
    }
}

/// Active curated whitelist filter (product profile overrides bare `KOTISATAMA_WHITELIST_PROFILE`).
pub fn effective_whitelist_profile() -> WhitelistProfile {
    if let Ok(tag) = std::env::var("KOTISATAMA_WHITELIST_PROFILE") {
        let tag = tag.trim().to_ascii_lowercase();
        if !tag.is_empty() && tag != "free" {
            return WhitelistProfile::Tagged(tag);
        }
        if tag == "free" {
            return WhitelistProfile::Free;
        }
    }
    ProductProfile::current().whitelist_profile()
}

fn seniori_avomeri_enabled() -> bool {
    std::env::var("KOTISATAMA_SENIORI_AVOMERI")
        .ok()
        .is_some_and(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normaali_allows_avomeri_and_user_domains() {
        let profile = ProductProfile::Normaali;
        assert!(profile.can_enter_avomeri());
        assert!(profile.can_add_user_domain());
        assert_eq!(profile.whitelist_profile(), WhitelistProfile::Free);
    }

    #[test]
    fn lapsi_blocks_avomeri_and_user_domains() {
        let profile = ProductProfile::Lapsi;
        assert!(!profile.can_enter_avomeri());
        assert!(!profile.can_add_user_domain());
        assert_eq!(
            profile.whitelist_profile(),
            WhitelistProfile::Tagged("lapsi".into())
        );
    }

    #[test]
    fn seniori_blocks_avomeri_by_default() {
        unsafe {
            std::env::remove_var("KOTISATAMA_SENIORI_AVOMERI");
        }
        let profile = ProductProfile::Seniori;
        assert!(!profile.can_enter_avomeri());
        assert!(!profile.can_add_user_domain());
    }

    #[test]
    fn seniori_allows_avomeri_when_carer_enables() {
        // SAFETY: test-only env mutation; single-threaded test runner.
        unsafe {
            std::env::set_var("KOTISATAMA_SENIORI_AVOMERI", "1");
        }
        assert!(ProductProfile::Seniori.can_enter_avomeri());
        unsafe {
            std::env::remove_var("KOTISATAMA_SENIORI_AVOMERI");
        }
    }
}
