/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Persistent user profile (Normi / Hopeakettu / Lapsi) with optional emoji lock.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::WhitelistError;
use crate::document::WhitelistProfile;

/// Runtime browser profile (task: Wikipedia / profiilit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    #[default]
    Normi,
    Hopeakettu,
    Lapsi,
}

impl Profile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normi => "normi",
            Self::Hopeakettu => "hopeakettu",
            Self::Lapsi => "lapsi",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "normi" | "normaali" | "free" | "adult" => Some(Self::Normi),
            "hopeakettu" => Some(Self::Hopeakettu),
            "lapsi" | "junior" => Some(Self::Lapsi),
            _ => None,
        }
    }

    /// Restrictions (emoji lock, filtered whitelist, optional Avomeri) apply when not Normi.
    pub fn restrictions_active(self) -> bool {
        !matches!(self, Self::Normi)
    }

    pub fn whitelist_profile(self) -> WhitelistProfile {
        match self {
            Self::Normi => WhitelistProfile::Free,
            Self::Hopeakettu => WhitelistProfile::Tagged("hopeakettu".into()),
            Self::Lapsi => WhitelistProfile::Tagged("lapsi".into()),
        }
    }

    /// Default Avomeri for this profile (before user toggle).
    pub fn default_avomeri(self) -> bool {
        match self {
            Self::Normi => true,
            Self::Hopeakettu => true,
            Self::Lapsi => false,
        }
    }

    /// Whether Avomeri can be enabled at all.
    pub fn can_enable_avomeri(self) -> bool {
        !matches!(self, Self::Lapsi)
    }
}

/// On-disk profile state (`profile.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileState {
    pub profile: Profile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji_hash: Option<String>,
    #[serde(default = "default_avomeri_true")]
    pub avomeri_enabled: bool,
    #[serde(default)]
    pub first_run_completed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lockout_until: Option<u64>,
    #[serde(default)]
    pub failed_attempts: u32,
}

fn default_avomeri_true() -> bool {
    true
}

impl Default for ProfileState {
    fn default() -> Self {
        Self {
            profile: Profile::Normi,
            emoji_hash: None,
            avomeri_enabled: true,
            first_run_completed: false,
            lockout_until: None,
            failed_attempts: 0,
        }
    }
}

impl ProfileState {
    pub fn load_from_path(path: &Path) -> Result<Self, WhitelistError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(path).map_err(WhitelistError::Io)?;
        serde_json::from_str(&contents).map_err(WhitelistError::Json)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), WhitelistError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(WhitelistError::Io)?;
        }
        let json = serde_json::to_string_pretty(self).map_err(WhitelistError::Json)?;
        fs::write(path, json).map_err(WhitelistError::Io)
    }

    pub fn is_locked_out(&self) -> bool {
        self.lockout_until
            .is_some_and(|until| now_unix_secs() < until)
    }

    pub fn lockout_remaining_secs(&self) -> u64 {
        self.lockout_until
            .map(|until| until.saturating_sub(now_unix_secs()))
            .unwrap_or(0)
    }
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

/// SHA-256 hex of the three-emoji sequence (UTF-8 concatenation).
pub fn hash_emoji_password(emojis: &[char]) -> String {
    let mut hasher = Sha256::new();
    for emoji in emojis {
        let mut buf = [0u8; 4];
        let encoded = emoji.encode_utf8(&mut buf);
        hasher.update(encoded.as_bytes());
    }
    hex_encode(&hasher.finalize())
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0xf) as usize] as char);
    }
    out
}

static PROFILE_PATH: OnceLock<PathBuf> = OnceLock::new();
static PROFILE_STATE: OnceLock<Mutex<ProfileState>> = OnceLock::new();

/// Default path: `%APPDATA%/kotisatama/profile.json` (or XDG config).
pub fn default_profile_path() -> PathBuf {
    if let Ok(path) = std::env::var("KOTISATAMA_PROFILE_PATH") {
        return PathBuf::from(path);
    }
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("kotisatama");
    dir.push("profile.json");
    dir
}

pub fn set_profile_path(path: PathBuf) {
    let _ = PROFILE_PATH.set(path);
}

fn profile_path() -> PathBuf {
    PROFILE_PATH
        .get()
        .cloned()
        .unwrap_or_else(default_profile_path)
}

fn state_lock() -> &'static Mutex<ProfileState> {
    PROFILE_STATE.get_or_init(|| {
        let path = profile_path();
        let state = ProfileState::load_from_path(&path).unwrap_or_default();
        Mutex::new(state)
    })
}

/// Load (or init) persisted profile state.
pub fn current_state() -> ProfileState {
    state_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

pub fn current_profile() -> Profile {
    // Env override still wins for CI / debugging.
    if let Ok(value) = std::env::var("KOTISATAMA_PRODUCT_PROFILE") {
        if let Some(profile) = Profile::parse(&value) {
            return profile;
        }
    }
    current_state().profile
}

pub fn profile_restrictions_active(profile: Profile) -> bool {
    profile.restrictions_active()
}

pub fn save_state(state: ProfileState) -> Result<(), WhitelistError> {
    let path = profile_path();
    state.save_to_path(&path)?;
    if let Ok(mut guard) = state_lock().lock() {
        *guard = state;
    }
    Ok(())
}

/// Set emoji password (exactly 3 emojis). Clears lockout.
pub fn set_emoji_password(emojis: &[char]) -> Result<(), WhitelistError> {
    if emojis.len() != 3 {
        return Err(WhitelistError::InvalidDomain(
            "emoji-salasana vaatii 3 emojia".into(),
        ));
    }
    let mut state = current_state();
    state.emoji_hash = Some(hash_emoji_password(emojis));
    state.failed_attempts = 0;
    state.lockout_until = None;
    save_state(state)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmojiAuthResult {
    Ok,
    Wrong,
    LockedOut,
    NotRequired,
}

/// Verify emoji password for restricted profiles. Updates failed_attempts / lockout.
pub fn verify_emoji_password(emojis: &[char]) -> Result<EmojiAuthResult, WhitelistError> {
    let mut state = current_state();
    if !state.profile.restrictions_active() {
        return Ok(EmojiAuthResult::NotRequired);
    }
    if state.emoji_hash.is_none() {
        return Ok(EmojiAuthResult::NotRequired);
    }
    if state.is_locked_out() {
        return Ok(EmojiAuthResult::LockedOut);
    }
    if emojis.len() != 3 {
        return Ok(EmojiAuthResult::Wrong);
    }
    let candidate = hash_emoji_password(emojis);
    if state.emoji_hash.as_deref() == Some(candidate.as_str()) {
        state.failed_attempts = 0;
        state.lockout_until = None;
        save_state(state)?;
        return Ok(EmojiAuthResult::Ok);
    }
    state.failed_attempts = state.failed_attempts.saturating_add(1);
    if state.failed_attempts >= 5 {
        state.lockout_until = Some(now_unix_secs() + 5 * 60);
        state.failed_attempts = 0;
    }
    save_state(state)?;
    Ok(EmojiAuthResult::Wrong)
}

/// Switch profile. Restricted targets require a valid emoji (or first-time set).
pub fn set_profile(profile: Profile, emojis: Option<&[char]>) -> Result<EmojiAuthResult, WhitelistError> {
    let mut state = current_state();
    if state.is_locked_out() {
        return Ok(EmojiAuthResult::LockedOut);
    }

    let from = state.profile;
    let to = profile;
    let needs_auth = from.restrictions_active() || to.restrictions_active();

    if needs_auth && from != to {
        if to.restrictions_active() && state.emoji_hash.is_none() {
            // First time entering a restricted profile: require setting 3 emojis.
            let Some(e) = emojis else {
                return Ok(EmojiAuthResult::Wrong);
            };
            if e.len() != 3 {
                return Ok(EmojiAuthResult::Wrong);
            }
            state.emoji_hash = Some(hash_emoji_password(e));
        } else if state.emoji_hash.is_some() {
            let Some(e) = emojis else {
                return Ok(EmojiAuthResult::Wrong);
            };
            if e.len() != 3 || state.emoji_hash.as_deref() != Some(hash_emoji_password(e).as_str()) {
                state.failed_attempts = state.failed_attempts.saturating_add(1);
                let locked = state.failed_attempts >= 5;
                if locked {
                    state.lockout_until = Some(now_unix_secs() + 5 * 60);
                    state.failed_attempts = 0;
                }
                save_state(state)?;
                return Ok(if locked {
                    EmojiAuthResult::LockedOut
                } else {
                    EmojiAuthResult::Wrong
                });
            }
            state.failed_attempts = 0;
            state.lockout_until = None;
        }
    }

    state.profile = to;
    state.avomeri_enabled = match to {
        Profile::Normi => true,
        Profile::Lapsi => false,
        Profile::Hopeakettu => state.avomeri_enabled,
    };
    if matches!(to, Profile::Normi) {
        state.emoji_hash = None;
        state.failed_attempts = 0;
        state.lockout_until = None;
    }
    state.first_run_completed = true;
    save_state(state)?;
    Ok(EmojiAuthResult::Ok)
}

pub fn set_avomeri_enabled(enabled: bool, emojis: Option<&[char]>) -> Result<EmojiAuthResult, WhitelistError> {
    let mut state = current_state();
    match state.profile {
        Profile::Normi => {
            state.avomeri_enabled = true;
            save_state(state)?;
            return Ok(EmojiAuthResult::NotRequired);
        },
        Profile::Lapsi => {
            state.avomeri_enabled = false;
            save_state(state)?;
            return Ok(EmojiAuthResult::Wrong);
        },
        Profile::Hopeakettu => {
            if state.emoji_hash.is_some() {
                match emojis {
                    Some(e) => match verify_emoji_password(e)? {
                        EmojiAuthResult::Ok => {},
                        other => return Ok(other),
                    },
                    None => return Ok(EmojiAuthResult::Wrong),
                }
            }
            state.avomeri_enabled = enabled;
            save_state(state)?;
            Ok(EmojiAuthResult::Ok)
        },
    }
}

pub fn mark_first_run_completed() -> Result<(), WhitelistError> {
    let mut state = current_state();
    state.first_run_completed = true;
    save_state(state)
}

/// Effective Avomeri for navigation (Normi always on, Lapsi always off).
pub fn avomeri_effectively_enabled() -> bool {
    let state = current_state();
    match state.profile {
        Profile::Normi => true,
        Profile::Lapsi => false,
        Profile::Hopeakettu => state.avomeri_enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normi_has_no_restrictions() {
        assert!(!Profile::Normi.restrictions_active());
        assert!(Profile::Hopeakettu.restrictions_active());
        assert!(Profile::Lapsi.restrictions_active());
    }

    #[test]
    fn emoji_hash_is_stable() {
        let a = hash_emoji_password(&['🔒', '🦊', '⚓']);
        let b = hash_emoji_password(&['🔒', '🦊', '⚓']);
        let c = hash_emoji_password(&['🔒', '⚓', '🦊']);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn profile_parse_aliases() {
        assert_eq!(Profile::parse("normaali"), Some(Profile::Normi));
        assert_eq!(Profile::parse("junior"), Some(Profile::Lapsi));
    }
}
