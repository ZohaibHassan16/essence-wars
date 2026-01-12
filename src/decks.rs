//! Deck definitions and registry for managing deck configurations.
//!
//! This module provides:
//! - TOML-based deck definitions
//! - Deck validation against card database
//! - DeckRegistry for loading and managing multiple decks

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::cards::CardDatabase;
use crate::types::CardId;

/// A deck definition loaded from a TOML file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeckDefinition {
    /// Unique identifier for this deck
    pub id: String,
    /// Display name
    pub name: String,
    /// Optional description of the deck's strategy
    #[serde(default)]
    pub description: String,
    /// Card IDs in the deck (may have duplicates)
    pub cards: Vec<u16>,
    /// Tags for categorization (e.g., "aggro", "control", "midrange")
    #[serde(default)]
    pub tags: Vec<String>,
}

impl DeckDefinition {
    /// Convert to a vector of CardIds.
    pub fn to_card_ids(&self) -> Vec<CardId> {
        self.cards.iter().map(|&id| CardId(id)).collect()
    }

    /// Validate that all cards exist in the database.
    pub fn validate(&self, card_db: &CardDatabase) -> Result<(), DeckError> {
        for &card_id in &self.cards {
            if card_db.get(CardId(card_id)).is_none() {
                return Err(DeckError::InvalidCard {
                    deck_id: self.id.clone(),
                    card_id,
                });
            }
        }
        Ok(())
    }

    /// Get the deck size.
    pub fn size(&self) -> usize {
        self.cards.len()
    }

    /// Check if deck has a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
    }
}

/// Registry for managing multiple deck definitions.
#[derive(Clone, Debug, Default)]
pub struct DeckRegistry {
    decks: HashMap<String, DeckDefinition>,
}

impl DeckRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            decks: HashMap::new(),
        }
    }

    /// Load decks from a directory.
    ///
    /// Searches for .toml files in the directory and loads them as deck definitions.
    pub fn load_from_directory<P: AsRef<Path>>(path: P) -> Result<Self, DeckError> {
        let mut registry = Self::new();
        let dir_path = path.as_ref();

        if !dir_path.exists() {
            return Err(DeckError::DirectoryNotFound(dir_path.display().to_string()));
        }

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().map_or(false, |ext| ext == "toml") {
                let content = fs::read_to_string(&file_path)?;
                let deck: DeckDefinition = toml::from_str(&content)
                    .map_err(|e| DeckError::ParseError {
                        path: file_path.display().to_string(),
                        error: e.to_string(),
                    })?;

                if registry.decks.contains_key(&deck.id) {
                    return Err(DeckError::DuplicateId(deck.id.clone()));
                }

                registry.decks.insert(deck.id.clone(), deck);
            }
        }

        Ok(registry)
    }

    /// Load a single deck from a TOML file.
    pub fn load_deck<P: AsRef<Path>>(path: P) -> Result<DeckDefinition, DeckError> {
        let content = fs::read_to_string(path.as_ref())?;
        let deck: DeckDefinition = toml::from_str(&content)
            .map_err(|e| DeckError::ParseError {
                path: path.as_ref().display().to_string(),
                error: e.to_string(),
            })?;
        Ok(deck)
    }

    /// Add a deck to the registry.
    pub fn add(&mut self, deck: DeckDefinition) -> Result<(), DeckError> {
        if self.decks.contains_key(&deck.id) {
            return Err(DeckError::DuplicateId(deck.id.clone()));
        }
        self.decks.insert(deck.id.clone(), deck);
        Ok(())
    }

    /// Get a deck by ID.
    pub fn get(&self, id: &str) -> Option<&DeckDefinition> {
        self.decks.get(id)
    }

    /// List all deck IDs.
    pub fn deck_ids(&self) -> Vec<&str> {
        self.decks.keys().map(|s| s.as_str()).collect()
    }

    /// List all decks.
    pub fn decks(&self) -> impl Iterator<Item = &DeckDefinition> {
        self.decks.values()
    }

    /// Get decks filtered by tag.
    pub fn decks_with_tag(&self, tag: &str) -> Vec<&DeckDefinition> {
        self.decks.values().filter(|d| d.has_tag(tag)).collect()
    }

    /// Validate all decks against the card database.
    pub fn validate_all(&self, card_db: &CardDatabase) -> Result<(), DeckError> {
        for deck in self.decks.values() {
            deck.validate(card_db)?;
        }
        Ok(())
    }

    /// Number of decks in the registry.
    pub fn len(&self) -> usize {
        self.decks.len()
    }

    /// Check if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.decks.is_empty()
    }
}

/// Errors that can occur when working with decks.
#[derive(Debug)]
pub enum DeckError {
    /// IO error reading deck file
    Io(std::io::Error),
    /// TOML parse error
    ParseError { path: String, error: String },
    /// Directory not found
    DirectoryNotFound(String),
    /// Duplicate deck ID
    DuplicateId(String),
    /// Card not found in database
    InvalidCard { deck_id: String, card_id: u16 },
}

impl std::fmt::Display for DeckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeckError::Io(e) => write!(f, "IO error: {}", e),
            DeckError::ParseError { path, error } => {
                write!(f, "Parse error in {}: {}", path, error)
            }
            DeckError::DirectoryNotFound(path) => write!(f, "Directory not found: {}", path),
            DeckError::DuplicateId(id) => write!(f, "Duplicate deck ID: {}", id),
            DeckError::InvalidCard { deck_id, card_id } => {
                write!(f, "Card {} not found (deck: {})", card_id, deck_id)
            }
        }
    }
}

impl std::error::Error for DeckError {}

impl From<std::io::Error> for DeckError {
    fn from(e: std::io::Error) -> Self {
        DeckError::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_definition() {
        let deck = DeckDefinition {
            id: "test".to_string(),
            name: "Test Deck".to_string(),
            description: "A test deck".to_string(),
            cards: vec![1, 1, 2, 2, 3],
            tags: vec!["aggro".to_string()],
        };

        assert_eq!(deck.size(), 5);
        assert!(deck.has_tag("aggro"));
        assert!(deck.has_tag("AGGRO"));
        assert!(!deck.has_tag("control"));

        let card_ids = deck.to_card_ids();
        assert_eq!(card_ids.len(), 5);
        assert_eq!(card_ids[0], CardId(1));
    }

    #[test]
    fn test_deck_validation() {
        let card_db = CardDatabase::load_from_directory("data/cards")
            .expect("Failed to load cards");

        // Valid deck
        let valid_deck = DeckDefinition {
            id: "valid".to_string(),
            name: "Valid".to_string(),
            description: String::new(),
            cards: vec![1, 1, 2, 2, 3, 3],
            tags: vec![],
        };
        assert!(valid_deck.validate(&card_db).is_ok());

        // Invalid deck (card 9999 doesn't exist)
        let invalid_deck = DeckDefinition {
            id: "invalid".to_string(),
            name: "Invalid".to_string(),
            description: String::new(),
            cards: vec![1, 1, 9999],
            tags: vec![],
        };
        assert!(invalid_deck.validate(&card_db).is_err());
    }

    #[test]
    fn test_deck_registry() {
        let mut registry = DeckRegistry::new();

        let deck1 = DeckDefinition {
            id: "deck1".to_string(),
            name: "Deck 1".to_string(),
            description: String::new(),
            cards: vec![1, 2, 3],
            tags: vec!["aggro".to_string()],
        };

        let deck2 = DeckDefinition {
            id: "deck2".to_string(),
            name: "Deck 2".to_string(),
            description: String::new(),
            cards: vec![4, 5, 6],
            tags: vec!["control".to_string()],
        };

        registry.add(deck1).unwrap();
        registry.add(deck2).unwrap();

        assert_eq!(registry.len(), 2);
        assert!(registry.get("deck1").is_some());
        assert!(registry.get("deck2").is_some());
        assert!(registry.get("deck3").is_none());

        let aggro_decks = registry.decks_with_tag("aggro");
        assert_eq!(aggro_decks.len(), 1);
        assert_eq!(aggro_decks[0].id, "deck1");
    }

    #[test]
    fn test_duplicate_id_rejected() {
        let mut registry = DeckRegistry::new();

        let deck1 = DeckDefinition {
            id: "same".to_string(),
            name: "Deck 1".to_string(),
            description: String::new(),
            cards: vec![1],
            tags: vec![],
        };

        let deck2 = DeckDefinition {
            id: "same".to_string(),
            name: "Deck 2".to_string(),
            description: String::new(),
            cards: vec![2],
            tags: vec![],
        };

        assert!(registry.add(deck1).is_ok());
        assert!(registry.add(deck2).is_err());
    }
}
