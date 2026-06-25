/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Varustamo app registry — luotettujen sovellusten luettelo Kotisatamassa.
//!
//! Rekisteri synkataan suljetusta reposta (`varustamo/registry.json`). Kotisatama
//! renderöi Varustamo-sivun ja ohjaa sovelluksiin `servo:`-sivuille.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use url::Url;

/// Varustamo registry file (schema v1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VarustamoRegistry {
    pub schema: u32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub apps: Vec<VarustamoApp>,
}

/// One installable / testable app in Varustamo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VarustamoApp {
    pub id: String,
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default, alias = "tauriConfig")]
    pub tauri_config: Option<String>,
    #[serde(default)]
    pub daemon: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default, rename = "noPermissions")]
    pub no_permissions: Vec<String>,
    #[serde(default, rename = "internetScope")]
    pub internet_scope: Vec<String>,
}

impl VarustamoApp {
    /// Resolved Tauri config path from registry JSON.
    pub fn tauri_config_path(&self) -> Option<&str> {
        self.tauri_config.as_deref()
    }
}

#[derive(Debug)]
pub enum VarustamoError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    NotFound,
}

impl std::fmt::Display for VarustamoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "varustamo IO: {error}"),
            Self::Parse(error) => write!(f, "varustamo JSON: {error}"),
            Self::NotFound => write!(
                f,
                "varustamo registry not found; set KOTISATAMA_VARUSTAMO_REGISTRY or sync from closed repo"
            ),
        }
    }
}

impl std::error::Error for VarustamoError {}

/// Varustamo hub page (`servo:varustamo`).
pub fn gateway_url() -> Url {
    Url::parse("servo:varustamo").expect("varustamo gateway URL must be valid")
}

/// JSON API for the hub page (`servo:varustamo/registry`).
pub fn registry_api_url() -> Url {
    Url::parse("servo:varustamo/registry").expect("varustamo registry API URL must be valid")
}

/// Map a registry app id to its Kotisatama gateway page.
pub fn app_gateway_url(app_id: &str) -> Result<Url, url::ParseError> {
    let path = match app_id {
        "pulloposti" => "pulloposti",
        "missa-olen" => "missa-olen",
        other => other,
    };
    Url::parse(&format!("servo:{path}"))
}

/// Default registry path candidates (first existing wins).
pub fn registry_path_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(override_path) = std::env::var("KOTISATAMA_VARUSTAMO_REGISTRY") {
        paths.push(PathBuf::from(override_path));
    }
    paths.push(PathBuf::from("config/varustamo/registry.json"));
    paths.push(PathBuf::from("config/varustamo/registry.example.json"));
    paths
}

/// Resolved registry file path, if any candidate exists.
pub fn resolve_registry_path() -> Option<PathBuf> {
    registry_path_candidates()
        .into_iter()
        .find(|path| path.is_file())
}

/// Load and parse the Varustamo registry.
pub fn load_registry() -> Result<VarustamoRegistry, VarustamoError> {
    let path = resolve_registry_path().ok_or(VarustamoError::NotFound)?;
    load_registry_from(&path)
}

/// Load registry from a specific path.
pub fn load_registry_from(path: &Path) -> Result<VarustamoRegistry, VarustamoError> {
    let contents = fs::read_to_string(path).map_err(VarustamoError::Io)?;
    serde_json::from_str(&contents).map_err(VarustamoError::Parse)
}

/// Raw registry JSON for `servo:varustamo/registry`.
pub fn load_registry_json() -> Result<String, VarustamoError> {
    let path = resolve_registry_path().ok_or(VarustamoError::NotFound)?;
    fs::read_to_string(path).map_err(VarustamoError::Io)
}

impl VarustamoRegistry {
    /// Apps shown in the Varustamo hub (testable or bundled).
    pub fn displayable_apps(&self) -> Vec<&VarustamoApp> {
        self.apps
            .iter()
            .filter(|app| matches!(app.status.as_str(), "testable" | "bundled" | "installed"))
            .collect()
    }

    /// Lookup app by id.
    pub fn find_app(&self, id: &str) -> Option<&VarustamoApp> {
        self.apps.iter().find(|app| app.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gateway_url_uses_servo_scheme() {
        assert_eq!(gateway_url().scheme(), "servo");
        assert_eq!(gateway_url().path(), "varustamo");
    }

    #[test]
    fn app_gateway_urls_for_known_apps() {
        assert_eq!(app_gateway_url("pulloposti").unwrap().path(), "pulloposti");
        assert_eq!(app_gateway_url("missa-olen").unwrap().path(), "missa-olen");
    }

    #[test]
    fn parses_example_registry() {
        let path = PathBuf::from("config/varustamo/registry.example.json");
        if !path.is_file() {
            return;
        }
        let registry = load_registry_from(&path).expect("example registry");
        assert_eq!(registry.schema, 1);
        assert!(!registry.displayable_apps().is_empty());
    }
}
