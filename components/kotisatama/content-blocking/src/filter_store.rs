//! Paketoidun suodatinlistan lukeminen.

use std::path::{Path, PathBuf};

/// Oletuslista craten `assets/filters.txt`-tiedostosta (tai annetusta polusta).
#[derive(Debug, Clone)]
pub struct FilterListStore {
    path: PathBuf,
}

impl FilterListStore {
    pub fn bundled() -> Self {
        Self {
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/filters.txt"),
        }
    }

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
