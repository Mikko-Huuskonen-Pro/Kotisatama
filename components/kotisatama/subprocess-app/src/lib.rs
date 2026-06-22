/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Shared helpers for Kotisatama bundled HTTP subprocess apps (Meilisearch, Pulloposti, …).

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for polling a subprocess `/health` endpoint.
#[derive(Debug, Clone, Copy)]
pub struct HealthCheckConfig {
    pub health_path: &'static str,
    pub poll_ms: u64,
    pub timeout_secs: u64,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            health_path: "/health",
            poll_ms: 100,
            timeout_secs: 30,
        }
    }
}

/// Errors shared by Kotisatama subprocess clients.
#[derive(Debug)]
pub enum SubprocessError {
    Io(std::io::Error),
    BinaryNotFound,
    Timeout,
    Http(String),
}

impl std::fmt::Display for SubprocessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::BinaryNotFound => write!(f, "subprocess binary not found"),
            Self::Timeout => write!(f, "subprocess health check timed out"),
            Self::Http(message) => write!(f, "subprocess HTTP: {message}"),
        }
    }
}

impl std::error::Error for SubprocessError {}

impl From<std::io::Error> for SubprocessError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// A subprocess started by Kotisatama; killed on drop when still owned.
pub struct ManagedSubprocess {
    child: Option<Child>,
}

impl ManagedSubprocess {
    pub fn from_child(child: Child) -> Self {
        Self {
            child: Some(child),
        }
    }

    pub fn detached() -> Self {
        Self { child: None }
    }
}

impl Drop for ManagedSubprocess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Resolve a binary from an environment variable pointing at a file path.
pub fn find_binary_from_env(env_var: &str) -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var(env_var).ok()?);
    path.is_file().then_some(path)
}

/// Resolve the first existing file from relative path candidates.
pub fn find_binary_candidates(candidates: &[&str]) -> Option<PathBuf> {
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

/// Resolve a binary from env override or known relative candidates.
pub fn find_binary(env_var: &str, candidates: &[&str]) -> Result<PathBuf, SubprocessError> {
    if let Some(path) = find_binary_from_env(env_var) {
        return Ok(path);
    }
    find_binary_candidates(candidates).ok_or(SubprocessError::BinaryNotFound)
}

/// Resolve a binary name from `PATH`.
pub fn find_on_path(binary_names: &[&str]) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        for name in binary_names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Spawn a subprocess with stdout/stderr discarded.
pub fn spawn_quiet(binary: &Path, args: &[String]) -> Result<ManagedSubprocess, SubprocessError> {
    let child = Command::new(binary)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(ManagedSubprocess::from_child(child))
}

/// Whether the HTTP health endpoint responds with status 200.
pub fn is_healthy(base_url: &str, health_path: &str) -> Result<bool, SubprocessError> {
    let url = format!(
        "{}{}",
        base_url.trim_end_matches('/'),
        normalize_health_path(health_path)
    );
    match ureq::get(&url).call() {
        Ok(response) => Ok(response.status() == 200),
        Err(ureq::Error::Status(404, _)) => Ok(false),
        Err(ureq::Error::Transport(_)) => Ok(false),
        Err(error) => Err(SubprocessError::Http(error.to_string())),
    }
}

/// Poll the health endpoint until ready or timeout.
pub fn wait_for_health(base_url: &str, config: HealthCheckConfig) -> Result<(), SubprocessError> {
    let deadline = Instant::now() + Duration::from_secs(config.timeout_secs);
    while Instant::now() < deadline {
        if is_healthy(base_url, config.health_path)? {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(config.poll_ms));
    }
    Err(SubprocessError::Timeout)
}

fn normalize_health_path(health_path: &str) -> &str {
    if health_path.starts_with('/') {
        health_path
    } else {
        "/health"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_health_path_adds_slash() {
        assert_eq!(normalize_health_path("health"), "/health");
        assert_eq!(normalize_health_path("/ready"), "/ready");
    }
}
