/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Missä olen subprocess client — käänteinen geokoodaus paikallisella daemonilla.
//!
//! Julkisessa repossa vain prosessinhallinta ja HTTP-rajapinta. Geokoodauslogiikka
//! bundlataan suljetusta reposta (`missa-olen-daemon`).

use std::process::{Command, Stdio};

use kotisatama_subprocess_app::{
    HealthCheckConfig, ManagedSubprocess, SubprocessError, find_binary, is_healthy, wait_for_health,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:7702";
const HEALTH_CONFIG: HealthCheckConfig = HealthCheckConfig {
    health_path: "/healthz",
    poll_ms: 100,
    timeout_secs: 15,
};

/// GPS-koordinaatit.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Fix {
    pub lat: f64,
    pub lon: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accuracy_m: Option<f64>,
}

/// Osoitetulos daemonista.
#[derive(Debug, Clone, Deserialize)]
pub struct AddressResult {
    pub address: String,
    pub lat: f64,
    pub lon: f64,
    pub source: String,
}

/// Paikallinen Missä olen -prosessi (valinnainen; voi olla jo käynnissä).
pub struct MissaOlenClient {
    base_url: String,
    #[allow(dead_code)]
    process: ManagedSubprocess,
}

impl MissaOlenClient {
    /// Käynnistä tai liitä olemassa olevaan Missä olen -instanssiin.
    pub fn start() -> Result<Self, MissaOlenError> {
        let base_url = std::env::var("KOTISATAMA_MISSA_OLEN_URL")
            .or_else(|_| std::env::var("VARUSTAMO_MISSA_OLEN_URL"))
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        if is_healthy(&base_url, HEALTH_CONFIG.health_path)? {
            log::info!("Missä olen: liitetty olemassa olevaan instanssiin ({base_url})");
            return Ok(Self {
                base_url,
                process: ManagedSubprocess::detached(),
            });
        }

        let binary = find_missa_olen_binary()?;
        log::info!(
            "Missä olen: käynnistetään subprocess ({})",
            binary.display()
        );

        let child = Command::new(&binary)
            .env("KOTISATAMA_MISSA_OLEN_URL", &base_url)
            .env("VARUSTAMO_MISSA_OLEN_URL", &base_url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(MissaOlenError::Io)?;

        wait_for_health(&base_url, HEALTH_CONFIG)?;

        Ok(Self {
            base_url,
            process: ManagedSubprocess::from_child(child),
        })
    }

    /// Onko daemon tavoitettavissa.
    pub fn is_available(&self) -> bool {
        is_healthy(&self.base_url, HEALTH_CONFIG.health_path).unwrap_or(false)
    }

    /// Gateway-sivu selaimessa (`servo:missa-olen`).
    pub fn gateway_url() -> Url {
        Url::parse("servo:missa-olen").expect("missa-olen gateway URL must be valid")
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Käänteinen geokoodaus koordinaateille.
    pub fn reverse(&self, fix: Fix, lang: Option<&str>) -> Result<AddressResult, MissaOlenError> {
        let mut body = json!({ "lat": fix.lat, "lon": fix.lon });
        if let Some(lang) = lang {
            body["lang"] = json!(lang);
        }
        post_json(
            &format!("{}/reverse", self.base_url.trim_end_matches('/')),
            body,
        )
    }
}

#[derive(Debug)]
pub enum MissaOlenError {
    Io(std::io::Error),
    BinaryNotFound,
    Timeout,
    Http(String),
    Json(String),
}

impl std::fmt::Display for MissaOlenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "missa-olen IO: {error}"),
            Self::BinaryNotFound => write!(
                f,
                "missa-olen-daemon not found; set KOTISATAMA_MISSA_OLEN_BIN or build from Kotisataman-suljetut-osat"
            ),
            Self::Timeout => write!(f, "missa-olen health check timed out"),
            Self::Http(message) => write!(f, "missa-olen HTTP: {message}"),
            Self::Json(message) => write!(f, "missa-olen JSON: {message}"),
        }
    }
}

impl std::error::Error for MissaOlenError {}

impl From<SubprocessError> for MissaOlenError {
    fn from(error: SubprocessError) -> Self {
        match error {
            SubprocessError::Io(error) => Self::Io(error),
            SubprocessError::BinaryNotFound => Self::BinaryNotFound,
            SubprocessError::Timeout => Self::Timeout,
            SubprocessError::Http(message) => Self::Http(message),
        }
    }
}

fn find_missa_olen_binary() -> Result<std::path::PathBuf, MissaOlenError> {
    find_binary(
        "KOTISATAMA_MISSA_OLEN_BIN",
        &[
            "missa-olen-daemon",
            "missa-olen-daemon.exe",
            "bin/missa-olen-daemon",
            "bin/missa-olen-daemon.exe",
        ],
    )
    .map_err(Into::into)
}

fn post_json<T: for<'de> Deserialize<'de>>(
    url: &str,
    body: serde_json::Value,
) -> Result<T, MissaOlenError> {
    let response = ureq::post(url)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|error| MissaOlenError::Http(error.to_string()))?;
    response
        .into_json()
        .map_err(|error| MissaOlenError::Json(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gateway_url_uses_servo_scheme() {
        assert_eq!(MissaOlenClient::gateway_url().scheme(), "servo");
        assert_eq!(MissaOlenClient::gateway_url().path(), "missa-olen");
    }
}
