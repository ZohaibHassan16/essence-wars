//! Bot factory for centralized bot creation and weight resolution.
//!
//! This module provides a unified interface for creating bots, reducing
//! duplication across arena, validate, and tune binaries.

use std::path::{Path, PathBuf};

use crate::cards::CardDatabase;
use crate::decks::Faction;

use super::weights::BotWeights;
use super::{Bot, GreedyBot, MctsBot, MctsConfig, RandomBot};

/// Bot types that can participate in matches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BotType {
    /// Random action selection
    Random,
    /// Greedy single-step evaluation
    Greedy,
    /// Monte Carlo Tree Search
    Mcts,
    /// Agent specialist for a faction (uses MCTS with specialist weights)
    AgentSpecialist(Faction),
    /// Agent generalist (uses MCTS with generalist weights)
    AgentGeneralist,
}

/// Error type for parsing BotType from string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotTypeParseError(String);

impl std::fmt::Display for BotTypeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown bot type: '{}'", self.0)
    }
}

impl std::error::Error for BotTypeParseError {}

impl std::str::FromStr for BotType {
    type Err = BotTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "random" => Ok(BotType::Random),
            "greedy" => Ok(BotType::Greedy),
            "mcts" => Ok(BotType::Mcts),
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
/// * `weights` - Optional custom weights (used by Greedy and MCTS)
/// * `mcts_config` - Configuration for MCTS bots
/// * `seed` - Random seed for this bot
///
/// # Returns
/// A boxed Bot trait object.
pub fn create_bot<'a>(
    card_db: &'a CardDatabase,
    bot_type: &BotType,
    weights: Option<&BotWeights>,
    mcts_config: &MctsConfig,
    seed: u64,
) -> Box<dyn Bot + 'a> {
    match bot_type {
        BotType::Random => Box::new(RandomBot::new(seed)),
        BotType::Greedy => match weights {
            Some(w) => Box::new(GreedyBot::from_bot_weights(card_db, w, None, seed)),
            None => Box::new(GreedyBot::new(card_db, seed)),
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
        println!(
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
                println!("Auto-loaded {} weights: {}", bot_type.name(), w.name);
                return Ok(Some(w));
            }
            Err(_) => {
                println!(
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
        assert!(BotType::Mcts.uses_mcts());
        assert!(BotType::AgentSpecialist(Faction::Argentum).uses_mcts());
        assert!(BotType::AgentGeneralist.uses_mcts());
    }

    #[test]
    fn test_bot_type_agent_weights_path() {
        assert_eq!(BotType::Random.agent_weights_path(), None);
        assert_eq!(BotType::Greedy.agent_weights_path(), None);
        assert_eq!(BotType::Mcts.agent_weights_path(), None);
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
}
