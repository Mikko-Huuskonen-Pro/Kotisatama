//! Estopäätökset — ei adblock-tyyppejä.

/// Mitä tehdään verkkopyynnölle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockingDecision {
    Allow,
    Block,
    /// Varattu myöhempää resurssien korvaamista varten (MVP ei käytä).
    Redirect {
        resource: Vec<u8>,
        mime_type: String,
    },
}
