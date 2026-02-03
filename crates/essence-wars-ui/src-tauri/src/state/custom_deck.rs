//! Data structures for custom deck building.
//!
//! Defines types for custom decks, playstyle classification, validation,
//! and browsable card information.

use cardgame::core::types::Rarity;
use serde::{Deserialize, Serialize};

/// Copy limits based on card rarity
pub const COPY_LIMIT_COMMON: u8 = 3;
pub const COPY_LIMIT_UNCOMMON: u8 = 2;
pub const COPY_LIMIT_RARE: u8 = 1;
pub const COPY_LIMIT_LEGENDARY: u8 = 1;

/// Minimum cards in a custom deck (excluding commander)
pub const MIN_DECK_SIZE: usize = 29;

/// Maximum cards in a custom deck (excluding commander)
pub const MAX_DECK_SIZE: usize = 100;

/// Get the copy limit for a given rarity
pub fn copy_limit_for_rarity(rarity: Rarity) -> u8 {
    match rarity {
        Rarity::Common => COPY_LIMIT_COMMON,
        Rarity::Uncommon => COPY_LIMIT_UNCOMMON,
        Rarity::Rare => COPY_LIMIT_RARE,
        Rarity::Legendary => COPY_LIMIT_LEGENDARY,
    }
}

/// A custom deck definition, stored as TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDeck {
    /// Unique identifier (e.g., "user_my_rush_deck")
    pub id: String,
    /// Display name
    pub name: String,
    /// Commander card ID
    pub commander: u16,
    /// Card IDs in the deck (29-100 cards, can have duplicates)
    pub cards: Vec<u16>,
    /// User-provided description
    #[serde(default)]
    pub description: String,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// Metadata section
    #[serde(default)]
    pub metadata: DeckMetadata,
}

/// Metadata for custom decks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeckMetadata {
    /// ISO 8601 timestamp when created
    #[serde(default)]
    pub created_at: Option<String>,
    /// ISO 8601 timestamp when last modified
    #[serde(default)]
    pub modified_at: Option<String>,
    /// Schema version for future compatibility
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_version() -> u32 {
    1
}

/// Summary info for a custom deck (used in deck lists)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomDeckInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub commander_id: u16,
    pub commander_name: String,
    pub faction: String,
    pub card_count: usize,
    pub playstyle: Option<PlaystyleScore>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
}

/// Playstyle classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Playstyle {
    Aggro,
    Control,
    Tempo,
    Midrange,
}

impl Playstyle {
    /// Get the display name for this playstyle
    pub fn display_name(&self) -> &'static str {
        match self {
            Playstyle::Aggro => "Aggro",
            Playstyle::Control => "Control",
            Playstyle::Tempo => "Tempo",
            Playstyle::Midrange => "Midrange",
        }
    }

    /// Get the corresponding weight file name
    pub fn weight_file(&self) -> &'static str {
        match self {
            Playstyle::Aggro => "aggro",
            Playstyle::Control => "control",
            Playstyle::Tempo => "tempo",
            Playstyle::Midrange => "midrange",
        }
    }
}

impl std::fmt::Display for Playstyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Playstyle scores for all archetypes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaystyleBreakdown {
    pub aggro: f32,
    pub control: f32,
    pub tempo: f32,
    pub midrange: f32,
}

impl PlaystyleBreakdown {
    pub fn new() -> Self {
        Self {
            aggro: 0.0,
            control: 0.0,
            tempo: 0.0,
            midrange: 0.0,
        }
    }
}

impl Default for PlaystyleBreakdown {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of playstyle calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaystyleScore {
    /// The dominant playstyle
    pub primary: Playstyle,
    /// Scores for all playstyles
    pub scores: PlaystyleBreakdown,
}

/// Deck validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckValidation {
    /// Whether the deck is valid and playable
    pub is_valid: bool,
    /// Critical errors that prevent playing
    pub errors: Vec<String>,
    /// Non-critical warnings
    pub warnings: Vec<String>,
}

impl DeckValidation {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Card information for the deck builder browser
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowsableCard {
    pub card_id: u16,
    pub name: String,
    pub cost: u8,
    pub card_type: String,
    pub faction: String,
    pub rarity: String,
    /// For creatures
    pub attack: Option<u8>,
    pub health: Option<u8>,
    pub keywords: Vec<String>,
    /// For supports
    pub durability: Option<u8>,
    /// Art path (relative to static folder)
    pub art_path: String,
    /// Copy limit based on rarity
    pub copy_limit: u8,
    /// Effect description for spells/supports
    pub effect_description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_limits() {
        assert_eq!(copy_limit_for_rarity(Rarity::Common), 3);
        assert_eq!(copy_limit_for_rarity(Rarity::Uncommon), 2);
        assert_eq!(copy_limit_for_rarity(Rarity::Rare), 1);
        assert_eq!(copy_limit_for_rarity(Rarity::Legendary), 1);
    }

    #[test]
    fn test_playstyle_display() {
        assert_eq!(Playstyle::Aggro.display_name(), "Aggro");
        assert_eq!(Playstyle::Control.display_name(), "Control");
        assert_eq!(Playstyle::Tempo.display_name(), "Tempo");
        assert_eq!(Playstyle::Midrange.display_name(), "Midrange");
    }

    #[test]
    fn test_validation_builder() {
        let mut validation = DeckValidation::valid();
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());

        validation.add_error("Test error".to_string());
        assert!(!validation.is_valid);
        assert_eq!(validation.errors.len(), 1);

        validation.add_warning("Test warning".to_string());
        assert_eq!(validation.warnings.len(), 1);
    }
}
