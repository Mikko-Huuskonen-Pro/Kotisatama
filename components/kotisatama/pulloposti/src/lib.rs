/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Pulloposti subprocess client — sama malli kuin Meilisearch-haulla.
//!
//! Julkisessa repossa vain prosessinhallinta ja health-check. Salaus, BLE ja
//! avainten hallinta bundlataan suljetusta reposta (`pulloposti-daemon`).

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use url::Url;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:7701";
const HEALTH_PATH: &str = "/health";
const HEALTH_POLL_MS: u64 = 100;
const HEALTH_TIMEOUT_SECS: u64 = 15;

/// Paikallinen Pulloposti-prosessi (valinnainen; voi olla jo käynnissä).
pub struct PullopostiClient {
    base_url: String,
    process: Option<Child>,
}

impl PullopostiClient {
    /// Käynnistä tai liitä olemassa olevaan Pulloposti-instanssiin.
    pub fn start() -> Result<Self, PullopostiError> {
        let base_url = std::env::var("KOTISATAMA_PULLOPOSTI_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        if is_healthy(&base_url)? {
            log::info!("Pulloposti: liitetty olemassa olevaan instanssiin ({base_url})");
            return Ok(Self {
                base_url,
                process: None,
            });
        }

        let binary = find_pulloposti_binary()?;
        log::info!("Pulloposti: käynnistetään subprocess ({})", binary.display());

        let process = Command::new(&binary)
            .env("KOTISATAMA_PULLOPOSTI_URL", &base_url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(PullopostiError::Spawn)?;

        wait_for_health(&base_url)?;

        Ok(Self {
            base_url,
            process: Some(process),
        })
    }

    /// Onko Pulloposti-prosessi tavoitettavissa.
    pub fn is_available(&self) -> bool {
        is_healthy(&self.base_url).unwrap_or(false)
    }

    /// Paikallinen gateway-sivu selaimessa (`servo:pulloposti`).
    pub fn gateway_url() -> Url {
        Url::parse("servo:pulloposti")
            .expect("pulloposti gateway URL must be valid")
    }
}

impl Drop for PullopostiClient {
    fn drop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[derive(Debug)]
pub enum PullopostiError {
    Io(std::io::Error),
    Spawn(std::io::Error),
    BinaryNotFound,
    Timeout,
    Http(String),
}

impl std::fmt::Display for PullopostiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "pulloposti IO: {e}"),
            Self::Spawn(e) => write!(f, "pulloposti spawn failed: {e}"),
            Self::BinaryNotFound => write!(
                f,
                "pulloposti-daemon not found; set KOTISATAMA_PULLOPOSTI_BIN or build from Kotisataman-suljetut-osat"
            ),
            Self::Timeout => write!(f, "pulloposti health check timed out"),
            Self::Http(msg) => write!(f, "pulloposti HTTP: {msg}"),
        }
    }
}

impl std::error::Error for PullopostiError {}

fn find_pulloposti_binary() -> Result<PathBuf, PullopostiError> {
    if let Ok(path) = std::env::var("KOTISATAMA_PULLOPOSTI_BIN") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }

    for candidate in [
        "pulloposti-daemon",
        "pulloposti-daemon.exe",
        "bin/pulloposti-daemon",
        "bin/pulloposti-daemon.exe",
    ] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Ok(path);
        }
    }

    Err(PullopostiError::BinaryNotFound)
}

fn is_healthy(base_url: &str) -> Result<bool, PullopostiError> {
    let url = format!("{}{HEALTH_PATH}", base_url.trim_end_matches('/'));
    match ureq::get(&url).call() {
        Ok(response) => Ok(response.status() == 200),
        Err(ureq::Error::Status(404, _)) => Ok(false),
        Err(ureq::Error::Transport(_)) => Ok(false),
        Err(e) => Err(PullopostiError::Http(e.to_string())),
    }
}

fn wait_for_health(base_url: &str) -> Result<(), PullopostiError> {
    let deadline = Instant::now() + Duration::from_secs(HEALTH_TIMEOUT_SECS);
    while Instant::now() < deadline {
        if is_healthy(base_url)? {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(HEALTH_POLL_MS));
    }
    Err(PullopostiError::Timeout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gateway_url_uses_servo_scheme() {
        assert_eq!(PullopostiClient::gateway_url().scheme(), "servo");
        assert_eq!(PullopostiClient::gateway_url().path(), "pulloposti");
    }
}
