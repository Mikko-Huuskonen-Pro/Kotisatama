// Kotisatama: whitelist-logiikka
// Lataa JSON-tiedoston ja tarkistaa onko URL sallittu

use url::Url;

pub struct Whitelist {
    domains: Vec<String>,
}

impl Whitelist {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let domains = serde_json::from_str(json)?;
        Ok(Self { domains })
    }

    pub fn is_allowed(&self, url: &Url) -> bool {
        let host = url.host_str().unwrap_or("");
        self.domains.iter().any(|d| host == d || host.ends_with(&format!(".{d}")))
    }
}
