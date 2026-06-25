/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Pulloposti subprocess client — sama malli kuin Meilisearch-haulla.
//!
//! Julkisessa repossa vain prosessinhallinta ja HTTP-rajapinta. Salaus, BLE ja
//! avainten hallinta bundlataan suljetusta reposta (`pulloposti-daemon`).

use std::process::Command;
use std::process::Stdio;

use kotisatama_subprocess_app::{
    HealthCheckConfig, ManagedSubprocess, SubprocessError, find_binary, is_healthy, wait_for_health,
};
use serde::Deserialize;
use serde_json::json;
use url::Url;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:7701";
const HEALTH_CONFIG: HealthCheckConfig = HealthCheckConfig {
    health_path: "/health",
    poll_ms: 100,
    timeout_secs: 15,
};

/// BLE-lähellä oleva laite Pullopostissa.
#[derive(Debug, Clone, Deserialize)]
pub struct PullopostiPeer {
    pub id: String,
    pub name: Option<String>,
    pub paired: Option<bool>,
}

/// Kirjeen metadata listauksessa.
#[derive(Debug, Clone, Deserialize)]
pub struct PullopostiLetter {
    pub id: String,
    pub from_peer_id: Option<String>,
    pub to_peer_id: Option<String>,
    pub preview: Option<String>,
    pub received_at: Option<String>,
}

/// Avattu kirje daemonista.
#[derive(Debug, Clone, Deserialize)]
pub struct PullopostiLetterBody {
    pub id: String,
    pub body: String,
    pub from_peer_id: Option<String>,
    pub to_peer_id: Option<String>,
}

/// Paikallinen Pulloposti-prosessi (valinnainen; voi olla jo käynnissä).
pub struct PullopostiClient {
    base_url: String,
    #[allow(dead_code)]
    process: ManagedSubprocess,
}

impl PullopostiClient {
    /// Käynnistä tai liitä olemassa olevaan Pulloposti-instanssiin.
    pub fn start() -> Result<Self, PullopostiError> {
        let base_url = std::env::var("KOTISATAMA_PULLOPOSTI_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        if is_healthy(&base_url, HEALTH_CONFIG.health_path)? {
            log::info!("Pulloposti: liitetty olemassa olevaan instanssiin ({base_url})");
            return Ok(Self {
                base_url,
                process: ManagedSubprocess::detached(),
            });
        }

        let binary = find_pulloposti_binary()?;
        log::info!(
            "Pulloposti: käynnistetään subprocess ({})",
            binary.display()
        );

        let child = Command::new(&binary)
            .env("KOTISATAMA_PULLOPOSTI_URL", &base_url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(PullopostiError::Io)?;

        wait_for_health(&base_url, HEALTH_CONFIG)?;

        Ok(Self {
            base_url,
            process: ManagedSubprocess::from_child(child),
        })
    }

    /// Onko Pulloposti-prosessi tavoitettavissa.
    pub fn is_available(&self) -> bool {
        is_healthy(&self.base_url, HEALTH_CONFIG.health_path).unwrap_or(false)
    }

    /// Paikallinen gateway-sivu selaimessa (`servo:pulloposti`).
    pub fn gateway_url() -> Url {
        Url::parse("servo:pulloposti").expect("pulloposti gateway URL must be valid")
    }

    /// Sovellusnäkymä selaimessa (`servo:pulloposti/app`).
    pub fn app_url() -> Url {
        Url::parse("servo:pulloposti/app").expect("pulloposti app URL must be valid")
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Lähistöllä olevat laitteet.
    pub fn list_peers(&self) -> Result<Vec<PullopostiPeer>, PullopostiError> {
        get_json(&format!("{}/peers", self.base_url.trim_end_matches('/')))
    }

    /// Aloita pariutuminen kuudella emojilla.
    pub fn pair(&self, emoji_code: &str) -> Result<(), PullopostiError> {
        post_json(
            &format!("{}/pair", self.base_url.trim_end_matches('/')),
            json!({ "emoji_code": emoji_code }),
        )
    }

    /// Listaa kirjeet (metadata).
    pub fn list_letters(&self) -> Result<Vec<PullopostiLetter>, PullopostiError> {
        get_json(&format!("{}/letters", self.base_url.trim_end_matches('/')))
    }

    /// Lue yksittäinen kirje.
    pub fn read_letter(&self, id: &str) -> Result<PullopostiLetterBody, PullopostiError> {
        get_json(&format!(
            "{}/letters/{}",
            self.base_url.trim_end_matches('/'),
            encode_path_segment(id)
        ))
    }

    /// Lähetä kirje.
    pub fn send_letter(&self, to_peer_id: &str, body: &str) -> Result<(), PullopostiError> {
        post_json(
            &format!("{}/letters", self.base_url.trim_end_matches('/')),
            json!({ "to_peer_id": to_peer_id, "body": body }),
        )
    }

    /// Poista kirje paikallisesti.
    pub fn delete_letter(&self, id: &str) -> Result<(), PullopostiError> {
        let url = format!(
            "{}/letters/{}",
            self.base_url.trim_end_matches('/'),
            encode_path_segment(id)
        );
        match ureq::delete(&url).call() {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(code, response)) => Err(PullopostiError::Http(format!(
                "delete letter failed (HTTP {code}): {}",
                response.into_string().unwrap_or_default()
            ))),
            Err(error) => Err(PullopostiError::Http(error.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum PullopostiError {
    Io(std::io::Error),
    BinaryNotFound,
    Timeout,
    Http(String),
    Json(String),
}

impl std::fmt::Display for PullopostiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "pulloposti IO: {error}"),
            Self::BinaryNotFound => write!(
                f,
                "pulloposti-daemon not found; set KOTISATAMA_PULLOPOSTI_BIN or build from Kotisataman-suljetut-osat"
            ),
            Self::Timeout => write!(f, "pulloposti health check timed out"),
            Self::Http(message) => write!(f, "pulloposti HTTP: {message}"),
            Self::Json(message) => write!(f, "pulloposti JSON: {message}"),
        }
    }
}

impl std::error::Error for PullopostiError {}

impl From<SubprocessError> for PullopostiError {
    fn from(error: SubprocessError) -> Self {
        match error {
            SubprocessError::Io(error) => Self::Io(error),
            SubprocessError::BinaryNotFound => Self::BinaryNotFound,
            SubprocessError::Timeout => Self::Timeout,
            SubprocessError::Http(message) => Self::Http(message),
        }
    }
}

fn find_pulloposti_binary() -> Result<std::path::PathBuf, PullopostiError> {
    find_binary(
        "KOTISATAMA_PULLOPOSTI_BIN",
        &[
            "pulloposti-daemon",
            "pulloposti-daemon.exe",
            "bin/pulloposti-daemon",
            "bin/pulloposti-daemon.exe",
        ],
    )
    .map_err(Into::into)
}

fn get_json<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T, PullopostiError> {
    let response = ureq::get(url)
        .call()
        .map_err(|error| PullopostiError::Http(error.to_string()))?;
    response
        .into_json()
        .map_err(|error| PullopostiError::Json(error.to_string()))
}

fn post_json(url: &str, body: serde_json::Value) -> Result<(), PullopostiError> {
    ureq::post(url)
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|error| PullopostiError::Http(error.to_string()))?;
    Ok(())
}

fn encode_path_segment(segment: &str) -> String {
    segment
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            },
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gateway_url_uses_servo_scheme() {
        assert_eq!(PullopostiClient::gateway_url().scheme(), "servo");
        assert_eq!(PullopostiClient::gateway_url().path(), "pulloposti");
    }

    #[test]
    fn app_url_uses_servo_scheme() {
        assert_eq!(PullopostiClient::app_url().scheme(), "servo");
        assert_eq!(PullopostiClient::app_url().path(), "pulloposti/app");
    }
}
