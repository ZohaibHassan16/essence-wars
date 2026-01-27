//! Unified data loading for game execution.
//!
//! Consolidates the common data loading patterns used across arena, validate,
//! and diagnose binaries into a single reusable module.

use std::path::Path;

use crate::bots::weights::ArchetypeWeights;
use crate::cards::CardDatabase;
use crate::decks::DeckRegistry;

/// All game data needed for execution.
///
/// This struct consolidates the loading of cards, commanders, decks, and
/// archetype-specific weights into a single operation, eliminating code
/// duplication across binaries.
///
/// # Example
///
/// ```ignore
/// // Load from explicit paths
/// let data = GameData::load(
///     Path::new("data/cards/core_set"),
///     Path::new("data/commanders"),
///     Path::new("data/decks"),
///     Path::new("data/weights"),
///     false, // not quiet
/// )?;
///
/// // Or load from default paths
/// let data = GameData::load_default(false)?;
/// ```
#[derive(Debug)]
pub struct GameData {
    /// Card database including commanders.
    pub card_db: CardDatabase,
    /// Registry of all available decks.
    pub deck_registry: DeckRegistry,
    /// Archetype-specific bot weights (Aggro, Control, Tempo, Midrange).
    pub archetype_weights: ArchetypeWeights,
}

impl GameData {
    /// Load all game data from explicit directory paths.
    ///
    /// # Arguments
    ///
    /// * `cards_dir` - Path to card YAML files (e.g., "data/cards/core_set")
    /// * `commanders_dir` - Path to commander YAML files (e.g., "data/commanders")
    /// * `decks_dir` - Path to deck TOML files (e.g., "data/decks")
    /// * `weights_dir` - Path to weight TOML files (e.g., "data/weights")
    /// * `quiet` - If true, suppress loading messages
    ///
    /// # Errors
    ///
    /// Returns an error if any of the directories cannot be loaded or
    /// contain invalid data.
    pub fn load(
        cards_dir: &Path,
        commanders_dir: &Path,
        decks_dir: &Path,
        weights_dir: &Path,
        quiet: bool,
    ) -> Result<Self, GameDataError> {
        let card_db = CardDatabase::load_with_commanders(cards_dir, commanders_dir)
            .map_err(|e| GameDataError::CardLoadError(e.to_string()))?;

        let deck_registry = DeckRegistry::load_from_directory(decks_dir)
            .map_err(|e| GameDataError::DeckLoadError(e.to_string()))?;

        let archetype_weights = ArchetypeWeights::load_from_directory(weights_dir, quiet);

        Ok(Self {
            card_db,
            deck_registry,
            archetype_weights,
        })
    }

    /// Load all game data from default paths relative to data_dir().
    ///
    /// Uses the standard directory structure:
    /// - `data/cards/core_set` - Card definitions
    /// - `data/commanders` - Commander definitions
    /// - `data/decks` - Deck definitions
    /// - `data/weights` - Bot weights
    ///
    /// # Arguments
    ///
    /// * `quiet` - If true, suppress loading messages
    pub fn load_default(quiet: bool) -> Result<Self, GameDataError> {
        let data = crate::data_dir();
        Self::load(
            &data.join("cards/core_set"),
            &data.join("commanders"),
            &data.join("decks"),
            &data.join("weights"),
            quiet,
        )
    }

    /// Load with custom paths, falling back to defaults for unspecified paths.
    ///
    /// This is useful for CLI binaries where users may override some paths
    /// but not others.
    pub fn load_with_overrides(
        cards_dir: Option<&Path>,
        commanders_dir: Option<&Path>,
        decks_dir: Option<&Path>,
        weights_dir: Option<&Path>,
        quiet: bool,
    ) -> Result<Self, GameDataError> {
        let data = crate::data_dir();
        Self::load(
            cards_dir.unwrap_or(&data.join("cards/core_set")),
            commanders_dir.unwrap_or(&data.join("commanders")),
            decks_dir.unwrap_or(&data.join("decks")),
            weights_dir.unwrap_or(&data.join("weights")),
            quiet,
        )
    }

    /// Get the number of decks loaded.
    pub fn deck_count(&self) -> usize {
        self.deck_registry.len()
    }

    /// Get the number of cards loaded (including commanders).
    pub fn card_count(&self) -> usize {
        self.card_db.len()
    }

    /// Get the number of commanders loaded.
    pub fn commander_count(&self) -> usize {
        self.card_db.commander_count()
    }
}

/// Errors that can occur when loading game data.
#[derive(Debug, Clone)]
pub enum GameDataError {
    /// Failed to load card database.
    CardLoadError(String),
    /// Failed to load deck registry.
    DeckLoadError(String),
}

impl std::fmt::Display for GameDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameDataError::CardLoadError(e) => write!(f, "Failed to load cards: {}", e),
            GameDataError::DeckLoadError(e) => write!(f, "Failed to load decks: {}", e),
        }
    }
}

impl std::error::Error for GameDataError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default() {
        let result = GameData::load_default(true);
        assert!(result.is_ok(), "Failed to load default game data: {:?}", result.err());

        let data = result.unwrap();
        assert!(data.card_count() > 0, "No cards loaded");
        assert!(data.deck_count() > 0, "No decks loaded");
        assert!(data.commander_count() > 0, "No commanders loaded");
    }

    #[test]
    fn test_expected_counts() {
        let data = GameData::load_default(true).unwrap();

        // Should have 12 decks (4 per faction)
        assert_eq!(data.deck_count(), 12, "Expected 12 decks");

        // Should have 12 commanders (4 per faction)
        assert_eq!(data.commander_count(), 12, "Expected 12 commanders");

        // Should have 300 cards (75 per faction including neutral)
        assert!(data.card_count() >= 300, "Expected at least 300 cards");
    }
}
