//! Bot factory for centralized bot creation and weight resolution.
//!
//! This module provides a unified interface for creating bots, reducing
//! duplication across arena, validate, and tune binaries.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::cards::CardDatabase;
use crate::decks::Faction;

use super::weights::BotWeights;
use super::{AlphaBetaBot, AlphaBetaConfig, Bot, GreedyBot, MctsBot, MctsConfig, RandomBot};

/// Bot types that can participate in matches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BotType {
    /// Random action selection
    Random,
    /// Greedy single-step evaluation
    Greedy,
    /// Monte Carlo Tree Search
    Mcts,
    /// Alpha-Beta Pruning (minimax search)
    AlphaBeta,
    /// Agent specialist for a faction (uses MCTS with specialist weights)
    AgentSpecialist(Faction),
    /// Agent generalist (uses MCTS with generalist weights)
    AgentGeneralist,
}

/// Error type for parsing BotType from string.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown bot type: '{0}'")]
pub struct BotTypeParseError(String);

impl std::str::FromStr for BotType {
    type Err = BotTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "random" => Ok(BotType::Random),
            "greedy" => Ok(BotType::Greedy),
            "mcts" => Ok(BotType::Mcts),
            "alphabeta" | "alpha-beta" | "ab" => Ok(BotType::AlphaBeta),
            "agent-argentum" => Ok(BotType::AgentSpecialist(Faction::Argentum)),
            "agent-symbiote" => Ok(BotType::AgentSpecialist(Faction::Symbiote)),
            "agent-obsidion" => Ok(BotType::AgentSpecialist(Faction::Obsidion)),
            "agent-generalist" => Ok(BotType::AgentGeneralist),
            _ => Err(BotTypeParseError(s.to_string())),
        }
    }
}

impl BotType {

    /// Get the display name for this bot type.
    pub fn name(&self) -> &'static str {
        match self {
            BotType::Random => "RandomBot",
            BotType::Greedy => "GreedyBot",
            BotType::Mcts => "MctsBot",
            BotType::AlphaBeta => "AlphaBetaBot",
            BotType::AgentSpecialist(Faction::Argentum) => "Agent-Argentum",
            BotType::AgentSpecialist(Faction::Symbiote) => "Agent-Symbiote",
            BotType::AgentSpecialist(Faction::Obsidion) => "Agent-Obsidion",
            BotType::AgentSpecialist(Faction::Neutral) => "Agent-Neutral",
            BotType::AgentGeneralist => "Agent-Generalist",
        }
    }

    /// Returns true if this bot type uses MCTS.
    pub fn uses_mcts(&self) -> bool {
        matches!(
            self,
            BotType::Mcts | BotType::AgentSpecialist(_) | BotType::AgentGeneralist
        )
    }

    /// Returns true for bots suitable for balance validation and serious analysis.
    ///
    /// Competitive bots (MCTS, Alpha-Beta, Agents) use proper search or tuned weights.
    /// Debug bots (Random, Greedy) are only suitable for quick sanity checks.
    pub fn is_competitive(&self) -> bool {
        matches!(
            self,
            BotType::Mcts
                | BotType::AlphaBeta
                | BotType::AgentSpecialist(_)
                | BotType::AgentGeneralist
        )
    }

    /// Returns the weights file path for Agent bot types, if applicable.
    ///
    /// Returns the relative path from the project root.
    pub fn agent_weights_path(&self) -> Option<PathBuf> {
        match self {
            BotType::AgentSpecialist(faction) => Some(PathBuf::from(format!(
                "data/weights/specialists/{}.toml",
                faction.as_tag()
            ))),
            BotType::AgentGeneralist => Some(PathBuf::from("data/weights/generalist.toml")),
            _ => None,
        }
    }

    /// Returns the faction for specialist agents, if applicable.
    pub fn faction(&self) -> Option<Faction> {
        match self {
            BotType::AgentSpecialist(f) => Some(*f),
            _ => None,
        }
    }

    /// All available bot type names for CLI help text.
    pub fn all_names() -> &'static [&'static str] {
        &[
            "random",
            "greedy",
            "mcts",
            "alphabeta",
            "agent-argentum",
            "agent-symbiote",
            "agent-obsidion",
            "agent-generalist",
        ]
    }
}

/// Create a bot instance based on type and configuration.
///
/// # Arguments
/// * `card_db` - Card database for greedy/mcts evaluation
/// * `bot_type` - The type of bot to create
/// * `weights` - Optional custom weights (used by Greedy, MCTS, and Alpha-Beta)
/// * `mcts_config` - Configuration for MCTS bots
/// * `alphabeta_config` - Configuration for Alpha-Beta bots
/// * `seed` - Random seed for this bot
///
/// # Returns
/// A boxed Bot trait object.
pub fn create_bot<'a>(
    card_db: &'a CardDatabase,
    bot_type: &BotType,
    weights: Option<&BotWeights>,
    mcts_config: &MctsConfig,
    alphabeta_config: &AlphaBetaConfig,
    seed: u64,
) -> Box<dyn Bot + 'a> {
    match bot_type {
        BotType::Random => Box::new(RandomBot::new(seed)),
        BotType::Greedy => match weights {
            Some(w) => Box::new(GreedyBot::from_bot_weights(card_db, w, None, seed)),
            None => Box::new(GreedyBot::new(card_db, seed)),
        },
        BotType::AlphaBeta => match weights {
            Some(w) => Box::new(AlphaBetaBot::with_config_and_weights(
                card_db,
                alphabeta_config.clone(),
                w,
                seed,
            )),
            None => Box::new(AlphaBetaBot::with_config(card_db, alphabeta_config.clone(), seed)),
        },
        BotType::Mcts | BotType::AgentSpecialist(_) | BotType::AgentGeneralist => match weights {
            Some(w) => Box::new(MctsBot::with_config_and_weights(
                card_db,
                mcts_config.clone(),
                w,
                seed,
            )),
            None => Box::new(MctsBot::with_config(card_db, mcts_config.clone(), seed)),
        },
    }
}

/// Resolve weights for a bot, checking explicit path first, then agent defaults.
///
/// # Arguments
/// * `bot_type` - The bot type (used to determine agent weights path)
/// * `explicit_path` - Explicitly specified weights path (takes precedence)
/// * `base_dir` - Base directory for resolving relative paths
///
/// # Returns
/// * `Ok(Some(weights))` - Weights loaded successfully
/// * `Ok(None)` - No weights needed or available
/// * `Err(error)` - Failed to load required weights
pub fn resolve_weights(
    bot_type: &BotType,
    explicit_path: Option<&Path>,
    base_dir: &Path,
) -> Result<Option<BotWeights>, WeightResolutionError> {
    // Explicit path takes precedence
    if let Some(path) = explicit_path {
        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            base_dir.join(path)
        };
        return BotWeights::load(&full_path)
            .map(Some)
            .map_err(|e| WeightResolutionError::LoadFailed(full_path, e.to_string()));
    }

    // Check for agent weights
    if let Some(agent_path) = bot_type.agent_weights_path() {
        let full_path = base_dir.join(&agent_path);
        match BotWeights::load(&full_path) {
            Ok(w) => return Ok(Some(w)),
            Err(_) => {
                // Agent weights are optional - fall back to defaults
                return Ok(None);
            }
        }
    }

    Ok(None)
}

/// Resolve weights based on deck archetype (playstyle field).
///
/// This function loads archetype-specific weights based on the deck's playstyle.
/// Valid archetypes: "Aggro", "Control", "Tempo", "Midrange" (case-insensitive).
///
/// # Arguments
/// * `archetype` - The deck's playstyle/archetype string
/// * `base_dir` - Base directory for resolving relative paths
///
/// # Returns
/// * `Ok(Some(weights))` - Archetype weights loaded successfully
/// * `Ok(None)` - Unknown archetype or weights not found (use defaults)
/// * `Err(error)` - Failed to load weights file
pub fn resolve_archetype_weights(
    archetype: &str,
    base_dir: &Path,
) -> Result<Option<BotWeights>, WeightResolutionError> {
    let archetype_lower = archetype.to_lowercase();
    let weight_file = match archetype_lower.as_str() {
        "aggro" => "aggro.toml",
        "control" => "control.toml",
        "tempo" => "tempo.toml",
        "midrange" => "midrange.toml",
        _ => return Ok(None), // Unknown archetype, use defaults
    };

    let full_path = base_dir.join("data/weights/archetypes").join(weight_file);
    match BotWeights::load(&full_path) {
        Ok(w) => Ok(Some(w)),
        Err(_) => Ok(None), // Archetype weights are optional
    }
}

/// Resolve weights with archetype fallback.
///
/// Resolution order:
/// 1. Explicit path (if provided)
/// 2. Agent-specific weights (if bot is an Agent type)
/// 3. Archetype weights (if archetype provided and recognized)
/// 4. None (use built-in defaults)
///
/// # Arguments
/// * `bot_type` - The bot type
/// * `explicit_path` - Explicitly specified weights path (takes precedence)
/// * `archetype` - Optional deck archetype/playstyle for weight selection
/// * `base_dir` - Base directory for resolving relative paths
pub fn resolve_weights_with_archetype(
    bot_type: &BotType,
    explicit_path: Option<&Path>,
    archetype: Option<&str>,
    base_dir: &Path,
) -> Result<Option<BotWeights>, WeightResolutionError> {
    // First try explicit path and agent weights
    if let Some(weights) = resolve_weights(bot_type, explicit_path, base_dir)? {
        return Ok(Some(weights));
    }

    // Then try archetype weights
    if let Some(arch) = archetype {
        if let Some(weights) = resolve_archetype_weights(arch, base_dir)? {
            return Ok(Some(weights));
        }
    }

    Ok(None)
}

/// Resolve weights with verbose output to stdout.
///
/// Same as `resolve_weights` but prints status messages.
pub fn resolve_weights_verbose(
    bot_type: &BotType,
    explicit_path: Option<&Path>,
    base_dir: &Path,
    bot_label: &str,
) -> Result<Option<BotWeights>, WeightResolutionError> {
    // Explicit path takes precedence
    if let Some(path) = explicit_path {
        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            base_dir.join(path)
        };
        let weights = BotWeights::load(&full_path)
            .map_err(|e| WeightResolutionError::LoadFailed(full_path.clone(), e.to_string()))?;
        log::info!(
            "Loaded weights for {}: {} ({} deck-specific)",
            bot_label,
            weights.name,
            weights.deck_specific.len()
        );
        return Ok(Some(weights));
    }

    // Check for agent weights
    if let Some(agent_path) = bot_type.agent_weights_path() {
        let full_path = base_dir.join(&agent_path);
        match BotWeights::load(&full_path) {
            Ok(w) => {
                log::info!("Auto-loaded {} weights: {}", bot_type.name(), w.name);
                return Ok(Some(w));
            }
            Err(_) => {
                log::debug!(
                    "Note: {} using default weights (no specialist weights at {:?})",
                    bot_type.name(),
                    full_path
                );
                return Ok(None);
            }
        }
    }

    Ok(None)
}

/// Resolve weights with archetype fallback and verbose output.
///
/// Same as `resolve_weights_with_archetype` but prints status messages.
pub fn resolve_weights_with_archetype_verbose(
    bot_type: &BotType,
    explicit_path: Option<&Path>,
    archetype: Option<&str>,
    base_dir: &Path,
    bot_label: &str,
) -> Result<Option<BotWeights>, WeightResolutionError> {
    // First try explicit path and agent weights (verbose)
    if let Some(weights) = resolve_weights_verbose(bot_type, explicit_path, base_dir, bot_label)? {
        return Ok(Some(weights));
    }

    // Then try archetype weights
    if let Some(arch) = archetype {
        if let Some(weights) = resolve_archetype_weights(arch, base_dir)? {
            log::info!(
                "Auto-loaded archetype weights for {}: {} (archetype: {})",
                bot_label, weights.name, arch
            );
            return Ok(Some(weights));
        }
    }

    log::debug!("Note: {} using built-in default weights", bot_label);
    Ok(None)
}

/// Error type for weight resolution.
#[derive(Debug, Clone)]
pub enum WeightResolutionError {
    /// Failed to load weights from the specified path.
    LoadFailed(PathBuf, String),
}

impl std::fmt::Display for WeightResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeightResolutionError::LoadFailed(path, msg) => {
                write!(f, "Failed to load weights from {:?}: {}", path, msg)
            }
        }
    }
}

impl std::error::Error for WeightResolutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_type_from_str() {
        assert_eq!("random".parse::<BotType>().unwrap(), BotType::Random);
        assert_eq!("Random".parse::<BotType>().unwrap(), BotType::Random);
        assert_eq!("RANDOM".parse::<BotType>().unwrap(), BotType::Random);
        assert_eq!("greedy".parse::<BotType>().unwrap(), BotType::Greedy);
        assert_eq!("mcts".parse::<BotType>().unwrap(), BotType::Mcts);
        assert_eq!("alphabeta".parse::<BotType>().unwrap(), BotType::AlphaBeta);
        assert_eq!("alpha-beta".parse::<BotType>().unwrap(), BotType::AlphaBeta);
        assert_eq!("ab".parse::<BotType>().unwrap(), BotType::AlphaBeta);
        assert_eq!(
            "agent-argentum".parse::<BotType>().unwrap(),
            BotType::AgentSpecialist(Faction::Argentum)
        );
        assert_eq!(
            "agent-symbiote".parse::<BotType>().unwrap(),
            BotType::AgentSpecialist(Faction::Symbiote)
        );
        assert_eq!(
            "agent-obsidion".parse::<BotType>().unwrap(),
            BotType::AgentSpecialist(Faction::Obsidion)
        );
        assert_eq!(
            "agent-generalist".parse::<BotType>().unwrap(),
            BotType::AgentGeneralist
        );
        assert!("invalid".parse::<BotType>().is_err());
    }

    #[test]
    fn test_bot_type_name() {
        assert_eq!(BotType::Random.name(), "RandomBot");
        assert_eq!(BotType::Greedy.name(), "GreedyBot");
        assert_eq!(BotType::Mcts.name(), "MctsBot");
        assert_eq!(BotType::AlphaBeta.name(), "AlphaBetaBot");
        assert_eq!(
            BotType::AgentSpecialist(Faction::Argentum).name(),
            "Agent-Argentum"
        );
        assert_eq!(BotType::AgentGeneralist.name(), "Agent-Generalist");
    }

    #[test]
    fn test_bot_type_uses_mcts() {
        assert!(!BotType::Random.uses_mcts());
        assert!(!BotType::Greedy.uses_mcts());
        assert!(!BotType::AlphaBeta.uses_mcts());
        assert!(BotType::Mcts.uses_mcts());
        assert!(BotType::AgentSpecialist(Faction::Argentum).uses_mcts());
        assert!(BotType::AgentGeneralist.uses_mcts());
    }

    #[test]
    fn test_bot_type_agent_weights_path() {
        assert_eq!(BotType::Random.agent_weights_path(), None);
        assert_eq!(BotType::Greedy.agent_weights_path(), None);
        assert_eq!(BotType::Mcts.agent_weights_path(), None);
        assert_eq!(BotType::AlphaBeta.agent_weights_path(), None);
        assert_eq!(
            BotType::AgentSpecialist(Faction::Argentum).agent_weights_path(),
            Some(PathBuf::from("data/weights/specialists/argentum.toml"))
        );
        assert_eq!(
            BotType::AgentGeneralist.agent_weights_path(),
            Some(PathBuf::from("data/weights/generalist.toml"))
        );
    }

    #[test]
    fn test_bot_type_faction() {
        assert_eq!(BotType::Random.faction(), None);
        assert_eq!(BotType::Mcts.faction(), None);
        assert_eq!(
            BotType::AgentSpecialist(Faction::Argentum).faction(),
            Some(Faction::Argentum)
        );
        assert_eq!(BotType::AgentGeneralist.faction(), None);
    }

    #[test]
    fn test_bot_type_is_competitive() {
        // Debug bots - not competitive
        assert!(!BotType::Random.is_competitive());
        assert!(!BotType::Greedy.is_competitive());
        // Competitive bots
        assert!(BotType::Mcts.is_competitive());
        assert!(BotType::AlphaBeta.is_competitive());
        assert!(BotType::AgentSpecialist(Faction::Argentum).is_competitive());
        assert!(BotType::AgentGeneralist.is_competitive());
    }
}
