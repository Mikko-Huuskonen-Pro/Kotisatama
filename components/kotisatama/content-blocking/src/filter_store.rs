//! Paketoidun suodatinlistan lukeminen.
//!
//! Bundled lista upotetaan binääriin (`include_str!`), jotta Androidilla
//! `CARGO_MANIFEST_DIR`-polku ei tarvitse olla olemassa laitteella.

use std::path::{Path, PathBuf};

/// Upotettu oletuslista (kääntyy binääriin).
pub const BUNDLED_FILTERS: &str = include_str!("../assets/filters.txt");

/// Tiedostopohjainen lista (OTA / testit env-polun kautta).
#[derive(Debug, Clone)]
pub struct FilterListStore {
    path: PathBuf,
}

impl FilterListStore {
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<String, std::io::Error> {
        std::fs::read_to_string(&self.path)
    }
}
