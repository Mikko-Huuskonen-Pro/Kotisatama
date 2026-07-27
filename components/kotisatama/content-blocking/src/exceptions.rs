//! Sivustokohtaiset poikkeukset (normalisoitu domain).

use std::collections::HashSet;
use std::sync::Mutex;

/// Käyttäjän hyväksymät sivustopoikkeukset.
#[derive(Debug, Default)]
pub struct SiteExceptionStore {
    domains: Mutex<HashSet<String>>,
}

impl SiteExceptionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_site(&self, domain: &str) {
        let normalized = normalize_domain(domain);
        if normalized.is_empty() {
            return;
        }
        if let Ok(mut set) = self.domains.lock() {
            set.insert(normalized);
        }
    }

    pub fn remove_site(&self, domain: &str) {
        let normalized = normalize_domain(domain);
        if let Ok(mut set) = self.domains.lock() {
            set.remove(&normalized);
        }
    }

    pub fn is_allowed(&self, domain_or_url: &str) -> bool {
        let host = host_from_url_or_domain(domain_or_url);
        let Ok(set) = self.domains.lock() else {
            return false;
        };
        // Tarkista host ja parent-domainit (sub.example.fi → example.fi)
        let mut candidate = host.as_str();
        loop {
            if set.contains(candidate) {
                return true;
            }
            match candidate.split_once('.') {
                Some((_, rest)) if rest.contains('.') || !rest.is_empty() => {
                    if !rest.contains('.') {
                        break;
                    }
                    candidate = rest;
                }
                _ => break,
            }
        }
        false
    }

    pub fn list(&self) -> Vec<String> {
        self.domains
            .lock()
            .map(|s| {
                let mut v: Vec<_> = s.iter().cloned().collect();
                v.sort();
                v
            })
            .unwrap_or_default()
    }
}

fn normalize_domain(raw: &str) -> String {
    host_from_url_or_domain(raw)
}

fn host_from_url_or_domain(raw: &str) -> String {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Some(rest) = trimmed.strip_prefix("http://")
        .or_else(|| trimmed.strip_prefix("https://"))
    {
        return rest
            .split('/')
            .next()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("")
            .to_string();
    }
    trimmed
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exception_matches_subdomain() {
        let store = SiteExceptionStore::new();
        store.allow_site("example.fi");
        assert!(store.is_allowed("https://www.example.fi/path"));
        assert!(store.is_allowed("example.fi"));
        assert!(!store.is_allowed("other.fi"));
    }
}
