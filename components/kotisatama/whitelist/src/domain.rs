/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Domain normalization and host matching.

use url::Url;

use crate::WhitelistError;

/// Normalize user or curated input into a registrable domain hostname.
pub fn normalize_domain(input: &str) -> Result<String, WhitelistError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(WhitelistError::InvalidDomain("tyhjä domain".into()));
    }

    if trimmed.contains("://") || trimmed.starts_with("//") {
        let url_str = if trimmed.starts_with("//") {
            format!("https:{trimmed}")
        } else {
            trimmed.to_string()
        };
        if let Ok(url) = Url::parse(&url_str) {
            if let Some(host) = url.host_str() {
                return normalize_host(host.trim_start_matches("www."));
            }
        }
    }

    let without_path = trimmed.split('/').next().unwrap_or(trimmed);
    let without_port = without_path.split(':').next().unwrap_or(without_path);
    normalize_host(without_port.trim_start_matches("www."))
}

fn normalize_host(host: &str) -> Result<String, WhitelistError> {
    let host = host
        .trim()
        .to_ascii_lowercase()
        .trim_end_matches('.')
        .to_string();
    if host.is_empty() {
        return Err(WhitelistError::InvalidDomain("tyhjä domain".into()));
    }
    if host.contains(' ') || !host.contains('.') {
        return Err(WhitelistError::InvalidDomain(host));
    }
    if !host
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(WhitelistError::InvalidDomain(host));
    }
    Ok(host)
}

/// Whether `host` matches an allowed registrable `domain` (subdomains included).
pub fn host_matches_domain(host: &str, domain: &str) -> bool {
    let host = host.trim().to_ascii_lowercase();
    let domain = domain.trim().to_ascii_lowercase();
    if domain.is_empty() {
        return false;
    }
    host == domain || host.ends_with(&format!(".{domain}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_url_and_www() {
        assert_eq!(
            normalize_domain("https://www.Kela.fi/path").unwrap(),
            "kela.fi"
        );
    }

    #[test]
    fn rejects_invalid_domain() {
        assert!(normalize_domain("not-a-domain").is_err());
    }
}
