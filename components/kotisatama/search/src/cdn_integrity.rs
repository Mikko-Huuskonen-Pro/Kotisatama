/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CDN manifest parsing, SHA-256 integrity checks, and Ed25519 signatures for OTA artifacts.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::SearchError;

/// CDN bundle manifest (`/free/manifest.json`).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CdnManifest {
    pub version: String,
    #[serde(default)]
    pub updated: Option<String>,
    pub files: BTreeMap<String, CdnManifestFile>,
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CdnManifestFile {
    pub sha256: String,
}

/// Canonical JSON body signed by the publisher (no `signature` field).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ManifestSignPayload {
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated: Option<String>,
    files: BTreeMap<String, CdnManifestFile>,
}

/// Whether integrity verification is disabled (development only).
pub fn skip_integrity_check() -> bool {
    #[cfg(debug_assertions)]
    {
        std::env::var("KOTISATAMA_CDN_SKIP_INTEGRITY")
            .ok()
            .is_some_and(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes"
                )
            })
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

/// Parse manifest JSON from disk.
pub fn load_manifest(path: &Path) -> Result<CdnManifest, SearchError> {
    let contents = fs::read_to_string(path).map_err(SearchError::Io)?;
    parse_manifest_str(&contents)
}

/// Parse and validate manifest JSON string.
pub fn parse_manifest_str(json: &str) -> Result<CdnManifest, SearchError> {
    let manifest: CdnManifest = serde_json::from_str(json).map_err(SearchError::Json)?;
    validate_manifest_fields(&manifest)?;
    Ok(manifest)
}

/// Verify manifest Ed25519 signature (required unless integrity skip is enabled).
pub fn verify_manifest_signature(manifest: &CdnManifest) -> Result<(), SearchError> {
    if skip_integrity_check() {
        return Ok(());
    }

    let signature_hex = manifest.signature.as_deref().ok_or_else(|| {
        SearchError::Integrity("CDN manifest is missing signature".into())
    })?;
    let signature_bytes = decode_hex(signature_hex).map_err(|error| {
        SearchError::Integrity(format!("invalid manifest signature hex: {error}"))
    })?;
    if signature_bytes.len() != 64 {
        return Err(SearchError::Integrity(
            "manifest signature must be 64 bytes".into(),
        ));
    }
    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&signature_bytes);
    let signature = Signature::from_bytes(&sig_array);

    let payload = signing_payload(manifest);
    let message = serde_json::to_string(&payload).map_err(SearchError::Json)?;

    let verifying_key = load_verifying_key()?;
    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| SearchError::Integrity("manifest Ed25519 signature invalid".into()))
}

fn signing_payload(manifest: &CdnManifest) -> ManifestSignPayload {
    ManifestSignPayload {
        version: manifest.version.clone(),
        updated: manifest.updated.clone(),
        files: manifest.files.clone(),
    }
}

fn validate_manifest_fields(manifest: &CdnManifest) -> Result<(), SearchError> {
    if manifest.files.is_empty() {
        return Err(SearchError::Integrity(
            "CDN manifest contains no files".into(),
        ));
    }
    for (name, entry) in &manifest.files {
        if entry.sha256.len() != 64 || !entry.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(SearchError::Integrity(format!(
                "invalid sha256 for {name} in manifest"
            )));
        }
    }
    Ok(())
}

fn load_verifying_key() -> Result<VerifyingKey, SearchError> {
    let hex_key = std::env::var("KOTISATAMA_CDN_PUBLIC_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            include_str!("../../../../config/cdn-signing-public.hex")
                .trim()
                .to_owned()
        });
    let bytes = decode_hex(hex_key.trim()).map_err(|error| {
        SearchError::Integrity(format!("invalid CDN public key hex: {error}"))
    })?;
    if bytes.len() != 32 {
        return Err(SearchError::Integrity(
            "CDN public key must be 32 bytes".into(),
        ));
    }
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&bytes);
    VerifyingKey::from_bytes(&key_bytes)
        .map_err(|error| SearchError::Integrity(format!("CDN public key invalid: {error}")))
}

/// SHA-256 hex digest of a file on disk.
pub fn sha256_file(path: &Path) -> Result<String, SearchError> {
    let mut file = File::open(path).map_err(SearchError::Io)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer).map_err(SearchError::Io)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex_encode(hasher.finalize().as_slice()))
}

/// Verify that `path` matches the manifest entry for `file_name`.
pub fn verify_file(manifest: &CdnManifest, file_name: &str, path: &Path) -> Result<(), SearchError> {
    let expected = manifest
        .files
        .get(file_name)
        .ok_or_else(|| SearchError::Integrity(format!("manifest missing entry for {file_name}")))?;
    let actual = sha256_file(path)?;
    if !expected.sha256.eq_ignore_ascii_case(&actual) {
        return Err(SearchError::Integrity(format!(
            "{file_name} sha256 mismatch (expected {}, got {actual})",
            expected.sha256
        )));
    }
    Ok(())
}

/// Fetch URL body to a temp file, verify against manifest, then atomically replace `dest`.
pub fn fetch_verify_and_install(
    url: &str,
    dest: &Path,
    manifest: &CdnManifest,
    file_name: &str,
) -> Result<(), SearchError> {
    let parent = dest
        .parent()
        .ok_or_else(|| SearchError::Io(io::Error::new(io::ErrorKind::NotFound, "no parent")))?;
    fs::create_dir_all(parent).map_err(SearchError::Io)?;

    let temp = dest.with_extension("tmp");
    fetch_to_file(url, &temp)?;

    if skip_integrity_check() {
        log::warn!("Kotisatama CDN: integrity check skipped for {file_name}");
    } else {
        verify_file(manifest, file_name, &temp)?;
    }

    if dest.exists() {
        fs::remove_file(dest).map_err(SearchError::Io)?;
    }
    fs::rename(&temp, dest).map_err(SearchError::Io)?;
    Ok(())
}

/// Fetch manifest from CDN base URL and verify signature.
pub fn fetch_manifest(base_url: &str) -> Result<CdnManifest, SearchError> {
    let base = base_url.trim_end_matches('/');
    let url = format!("{base}/free/manifest.json");
    let temp = data_dir().join("cache").join("manifest.json.tmp");
    if let Some(parent) = temp.parent() {
        fs::create_dir_all(parent).map_err(SearchError::Io)?;
    }
    fetch_to_file(&url, &temp)?;
    let manifest = load_manifest(&temp)?;
    verify_manifest_signature(&manifest)?;
    let dest = data_dir().join("cache").join("manifest.json");
    if dest.exists() {
        let _ = fs::remove_file(&dest);
    }
    fs::rename(&temp, &dest).map_err(SearchError::Io)?;
    Ok(manifest)
}

fn fetch_to_file(url: &str, dest: &Path) -> Result<(), SearchError> {
    let response = ureq::get(url)
        .call()
        .map_err(|error| SearchError::Http(error.to_string()))?;
    if response.status() != 200 {
        return Err(SearchError::Http(format!(
            "GET {url} returned HTTP {}",
            response.status()
        )));
    }
    let mut reader = response.into_reader();
    let mut file = File::create(dest).map_err(SearchError::Io)?;
    io::copy(&mut reader, &mut file).map_err(SearchError::Io)?;
    Ok(())
}

fn data_dir() -> std::path::PathBuf {
    std::env::var("KOTISATAMA_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("index-data"))
}

fn decode_hex(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("odd length".into());
    }
    (0..hex.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&hex[index..index + 2], 16)
                .map_err(|error| error.to_string())
        })
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use std::io::Write;

    fn test_manifest_with_signature(signing_key: &SigningKey) -> CdnManifest {
        let mut files = BTreeMap::new();
        files.insert(
            "whitelist.json".into(),
            CdnManifestFile {
                sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .into(),
            },
        );
        let mut manifest = CdnManifest {
            version: "1".into(),
            updated: Some("2026-06-30".into()),
            files,
            signature: None,
        };
        let payload = signing_payload(&manifest);
        let message = serde_json::to_string(&payload).unwrap();
        let signature = signing_key.sign(message.as_bytes());
        manifest.signature = Some(hex_encode(signature.to_bytes().as_slice()));
        manifest
    }

    #[test]
    fn parses_manifest_and_verifies_hash() {
        let signing_key = SigningKey::from_bytes(&[0x42; 32]);
        let manifest = test_manifest_with_signature(&signing_key);
        let temp = std::env::temp_dir().join("kotisatama-manifest-test-empty.json");
        let mut file = File::create(&temp).unwrap();
        file.write_all(b"").unwrap();
        verify_file(&manifest, "whitelist.json", &temp).unwrap();
        let _ = fs::remove_file(&temp);
    }

    #[test]
    fn rejects_hash_mismatch() {
        let signing_key = SigningKey::from_bytes(&[0x42; 32]);
        let manifest = test_manifest_with_signature(&signing_key);
        let temp = std::env::temp_dir().join("kotisatama-manifest-test-mismatch.json");
        let mut file = File::create(&temp).unwrap();
        file.write_all(b"x").unwrap();
        assert!(verify_file(&manifest, "whitelist.json", &temp).is_err());
        let _ = fs::remove_file(&temp);
    }

    #[test]
    fn verifies_manifest_signature_roundtrip() {
        let signing_key = SigningKey::from_bytes(&[0x42; 32]);
        let manifest = test_manifest_with_signature(&signing_key);
        let public_hex = hex_encode(signing_key.verifying_key().as_bytes());
        unsafe {
            std::env::set_var("KOTISATAMA_CDN_PUBLIC_KEY", &public_hex);
        }
        verify_manifest_signature(&manifest).unwrap();
        unsafe {
            std::env::remove_var("KOTISATAMA_CDN_PUBLIC_KEY");
        }
    }

    #[test]
    fn rejects_tampered_manifest_signature() {
        let signing_key = SigningKey::from_bytes(&[0x42; 32]);
        let public_hex = hex_encode(signing_key.verifying_key().as_bytes());
        unsafe {
            std::env::set_var("KOTISATAMA_CDN_PUBLIC_KEY", &public_hex);
        }
        let mut manifest = test_manifest_with_signature(&signing_key);
        manifest.version = "2".into();
        assert!(verify_manifest_signature(&manifest).is_err());
        unsafe {
            std::env::remove_var("KOTISATAMA_CDN_PUBLIC_KEY");
        }
    }
}
