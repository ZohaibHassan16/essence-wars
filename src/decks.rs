//! Deck definitions and registry for managing deck configurations.
//!
//! This module provides:
//! - TOML-based deck definitions
//! - Deck validation against card database
//! - DeckRegistry for loading and managing multiple decks
//! - Faction system for organizing decks by faction identity

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::cards::CardDatabase;
use crate::types::CardId;

/// The three true factions plus neutral (Free-Walkers).
///
/// Faction decks contain a core of faction cards (typically 14) plus
/// a splash of neutral Free-Walker cards (typically 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Faction {
    /// Argentum Combine - "The Wall"
    /// Defense-focused with Guard, Piercing, Shield
    Argentum,
    /// Symbiote Circles - "The Swarm"
    /// Aggressive tempo with Rush, Lethal, Regenerate
    Symbiote,
    /// Obsidion Syndicate - "The Glass Cannon"
    /// Burst damage with Lifesteal, Stealth, Ephemeral, Quick
    Obsidion,
    /// Free-Walkers / Neutral - "The Toolbox"
    /// Flexible neutrals with Ranged, Charge (not a true faction)
    Neutral,
}

impl Faction {
    /// Returns all true factions (excluding Neutral).
    pub fn all_factions() -> &'static [Faction] {
        &[Faction::Argentum, Faction::Symbiote, Faction::Obsidion]
    }

    /// Returns all factions including Neutral.
    pub fn all() -> &'static [Faction] {
        &[Faction::Argentum, Faction::Symbiote, Faction::Obsidion, Faction::Neutral]
    }

    /// Check if this is a true faction (not neutral).
    pub fn is_true_faction(&self) -> bool {
        !matches!(self, Faction::Neutral)
    }

    /// Get the faction's tag name (lowercase).
    pub fn as_tag(&self) -> &'static str {
        match self {
            Faction::Argentum => "argentum",
            Faction::Symbiote => "symbiote",
            Faction::Obsidion => "obsidion",
            Faction::Neutral => "neutral",
        }
    }

    /// Get the faction's display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Faction::Argentum => "Argentum Combine",
            Faction::Symbiote => "Symbiote Circles",
            Faction::Obsidion => "Obsidion Syndicate",
            Faction::Neutral => "Free-Walkers",
        }
    }

    /// Get the faction's short description.
    pub fn description(&self) -> &'static str {
        match self {
            Faction::Argentum => "The Wall - Defense and durability",
            Faction::Symbiote => "The Swarm - Aggressive tempo",
            Faction::Obsidion => "The Glass Cannon - Burst damage",
            Faction::Neutral => "The Toolbox - Flexible neutrals",
        }
    }
}

impl fmt::Display for Faction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for Faction {
    type Err = FactionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "argentum" => Ok(Faction::Argentum),
            "symbiote" => Ok(Faction::Symbiote),
            "obsidion" => Ok(Faction::Obsidion),
            "neutral" | "freewalker" | "free-walker" | "freewalkers" | "free-walkers" => Ok(Faction::Neutral),
            _ => Err(FactionParseError(s.to_string())),
        }
    }
}

/// Error when parsing a faction from string.
#[derive(Debug, Clone)]
pub struct FactionParseError(pub String);

impl fmt::Display for FactionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown faction: '{}'. Valid factions: argentum, symbiote, obsidion, neutral", self.0)
    }
}

impl std::error::Error for FactionParseError {}

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

    /// Determine the faction of this deck based on tags.
    ///
    /// Returns the first matching faction tag found, or None if no faction tag.
    /// Faction decks should have exactly one faction tag (argentum, symbiote, or obsidion).
    pub fn faction(&self) -> Option<Faction> {
        for tag in &self.tags {
            if let Ok(faction) = tag.parse::<Faction>() {
                // Only return true factions, not neutral
                if faction.is_true_faction() {
                    return Some(faction);
                }
            }
        }
        None
    }

    /// Check if this deck belongs to a specific faction.
    pub fn is_faction(&self, faction: Faction) -> bool {
        self.faction() == Some(faction)
    }

    /// Check if this deck is a faction deck (has a faction tag).
    pub fn is_faction_deck(&self) -> bool {
        self.faction().is_some()
    }

    /// Check if this deck is compatible with a specialist agent for the given faction.
    ///
    /// A specialist agent can only play decks of its own faction.
    pub fn is_compatible_with_specialist(&self, faction: Faction) -> bool {
        self.faction() == Some(faction)
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
    /// Recursively searches for .toml files in the directory and all subdirectories,
    /// loading them as deck definitions.
    pub fn load_from_directory<P: AsRef<Path>>(path: P) -> Result<Self, DeckError> {
        let mut registry = Self::new();
        let dir_path = path.as_ref();

        if !dir_path.exists() {
            return Err(DeckError::DirectoryNotFound(dir_path.display().to_string()));
        }

        // Collect all TOML files recursively
        let toml_files = Self::collect_toml_files(dir_path)?;

        for file_path in toml_files {
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

        Ok(registry)
    }

    /// Recursively collect all .toml files from a directory and its subdirectories.
    fn collect_toml_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, DeckError> {
        let mut files = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively collect from subdirectory
                files.extend(Self::collect_toml_files(&path)?);
            } else if path.extension().is_some_and(|ext| ext == "toml") {
                files.push(path);
            }
        }

        Ok(files)
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

    /// Get all decks belonging to a specific faction.
    pub fn decks_for_faction(&self, faction: Faction) -> Vec<&DeckDefinition> {
        self.decks.values().filter(|d| d.is_faction(faction)).collect()
    }

    /// Get all faction decks (decks with a faction tag).
    pub fn faction_decks(&self) -> Vec<&DeckDefinition> {
        self.decks.values().filter(|d| d.is_faction_deck()).collect()
    }

    /// Get all non-faction decks (neutral/mixed decks).
    pub fn neutral_decks(&self) -> Vec<&DeckDefinition> {
        self.decks.values().filter(|d| !d.is_faction_deck()).collect()
    }

    /// Get all decks compatible with a specialist agent.
    pub fn decks_for_specialist(&self, faction: Faction) -> Vec<&DeckDefinition> {
        self.decks.values()
            .filter(|d| d.is_compatible_with_specialist(faction))
            .collect()
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
