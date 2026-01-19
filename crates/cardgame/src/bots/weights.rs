//! Weight configurations for bot evaluation functions.
//!
//! This module provides serializable weight structures that control how bots
//! evaluate game states. Weights can be loaded from TOML files, enabling
//! easy tuning and the creation of specialist bots for different decks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Complete weight configuration for a bot.
///
/// Supports both generalist (default) weights and deck-specific specialist weights.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotWeights {
    /// Name of this weight configuration
    pub name: String,

    /// Version for compatibility tracking
    #[serde(default = "default_version")]
    pub version: u32,

    /// Default weights used for any deck
    pub default: WeightSet,

    /// Deck-specific weight overrides
    /// Key is the deck identifier (e.g., "aggressive_assault")
    #[serde(default)]
    pub deck_specific: HashMap<String, WeightSet>,
}

fn default_version() -> u32 { 1 }

impl BotWeights {
    /// Create new BotWeights with default greedy weights.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: 1,
            default: WeightSet::default(),
            deck_specific: HashMap::new(),
        }
    }

    /// Load weights from a TOML file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, WeightError> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| WeightError::Io(e.to_string()))?;
        toml::from_str(&content)
            .map_err(|e| WeightError::Parse(e.to_string()))
    }

    /// Save weights to a TOML file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), WeightError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| WeightError::Serialize(e.to_string()))?;
        fs::write(path.as_ref(), content)
            .map_err(|e| WeightError::Io(e.to_string()))
    }

    /// Get weights for a specific deck, falling back to default.
    pub fn for_deck(&self, deck_id: &str) -> &WeightSet {
        self.deck_specific.get(deck_id).unwrap_or(&self.default)
    }

    /// Add deck-specific weights.
    pub fn with_deck_weights(mut self, deck_id: impl Into<String>, weights: WeightSet) -> Self {
        self.deck_specific.insert(deck_id.into(), weights);
        self
    }
}

impl Default for BotWeights {
    fn default() -> Self {
        Self::new("default")
    }
}

/// A complete set of weights for evaluating game states.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WeightSet {
    /// Weights for evaluating game state
    pub greedy: GreedyWeights,
}

/// Weights for the greedy evaluation function.
///
/// Higher positive values mean the feature is more desirable.
/// Negative values would penalize that feature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GreedyWeights {
    // === Life and Damage ===
    /// Value per point of own life
    pub own_life: f32,
    /// Value per point of enemy life lost (negative of enemy life)
    pub enemy_life_damage: f32,

    // === Creatures ===
    /// Value per point of own creature attack
    pub own_creature_attack: f32,
    /// Value per point of own creature health
    pub own_creature_health: f32,
    /// Value per point of enemy creature attack (usually negative)
    pub enemy_creature_attack: f32,
    /// Value per point of enemy creature health (usually negative)
    pub enemy_creature_health: f32,

    // === Board Control ===
    /// Value for each creature we have on board
    pub creature_count: f32,
    /// Value for each creature slot advantage over opponent
    pub board_advantage: f32,

    // === Hand and Resources ===
    /// Value per card in hand
    pub cards_in_hand: f32,
    /// Value per action point remaining (discourages waste)
    pub action_points: f32,

    // === Keyword Values (Original 8) ===
    /// Value for each creature with Guard
    pub keyword_guard: f32,
    /// Value for each creature with Lethal
    pub keyword_lethal: f32,
    /// Value for each creature with Lifesteal
    pub keyword_lifesteal: f32,
    /// Value for each creature with Rush
    pub keyword_rush: f32,
    /// Value for each creature with Ranged
    pub keyword_ranged: f32,
    /// Value for each creature with Piercing
    pub keyword_piercing: f32,
    /// Value for each creature with Shield
    pub keyword_shield: f32,
    /// Value for each creature with Quick
    pub keyword_quick: f32,

    // === Keyword Values (New 4) ===
    /// Value for each creature with Ephemeral (negative - dies at end of turn)
    pub keyword_ephemeral: f32,
    /// Value for each creature with Regenerate (heals 2 HP at start of turn)
    pub keyword_regenerate: f32,
    /// Value for each creature with Stealth (untargetable by enemy)
    pub keyword_stealth: f32,
    /// Value for each creature with Charge (+2 attack when attacking)
    pub keyword_charge: f32,

    // === Keyword Values (Symbiote - v0.5.0) ===
    /// Value for each creature with Frenzy (+1 attack after each attack this turn)
    pub keyword_frenzy: f32,
    /// Value for each creature with Volatile (deal 2 damage to all enemies on death)
    pub keyword_volatile: f32,

    // === Keyword Values (Phase 5 - v0.5.0) ===
    /// Value for each creature with Fortify (takes 1 less damage from all sources)
    pub keyword_fortify: f32,
    /// Value for each creature with Ward (first spell/ability targeting has no effect)
    pub keyword_ward: f32,

    // === Strategic Bonuses ===
    /// Bonus for winning the game (should be very high)
    pub win_bonus: f32,
    /// Penalty for losing the game (should be very negative)
    pub lose_penalty: f32,
}

impl GreedyWeights {
    /// Create a new set of weights with all zeros.
    pub fn zeros() -> Self {
        Self {
            own_life: 0.0,
            enemy_life_damage: 0.0,
            own_creature_attack: 0.0,
            own_creature_health: 0.0,
            enemy_creature_attack: 0.0,
            enemy_creature_health: 0.0,
            creature_count: 0.0,
            board_advantage: 0.0,
            cards_in_hand: 0.0,
            action_points: 0.0,
            keyword_guard: 0.0,
            keyword_lethal: 0.0,
            keyword_lifesteal: 0.0,
            keyword_rush: 0.0,
            keyword_ranged: 0.0,
            keyword_piercing: 0.0,
            keyword_shield: 0.0,
            keyword_quick: 0.0,
            keyword_ephemeral: 0.0,
            keyword_regenerate: 0.0,
            keyword_stealth: 0.0,
            keyword_charge: 0.0,
            keyword_frenzy: 0.0,
            keyword_volatile: 0.0,
            keyword_fortify: 0.0,
            keyword_ward: 0.0,
            win_bonus: 0.0,
            lose_penalty: 0.0,
        }
    }

    /// Convert weights to a vector for optimization algorithms.
    pub fn to_vec(&self) -> Vec<f32> {
        vec![
            self.own_life,
            self.enemy_life_damage,
            self.own_creature_attack,
            self.own_creature_health,
            self.enemy_creature_attack,
            self.enemy_creature_health,
            self.creature_count,
            self.board_advantage,
            self.cards_in_hand,
            self.action_points,
            self.keyword_guard,
            self.keyword_lethal,
            self.keyword_lifesteal,
            self.keyword_rush,
            self.keyword_ranged,
            self.keyword_piercing,
            self.keyword_shield,
            self.keyword_quick,
            self.keyword_ephemeral,
            self.keyword_regenerate,
            self.keyword_stealth,
            self.keyword_charge,
            self.keyword_frenzy,
            self.keyword_volatile,
            self.keyword_fortify,
            self.keyword_ward,
            self.win_bonus,
            self.lose_penalty,
        ]
    }

    /// Create weights from a vector (for optimization algorithms).
    pub fn from_vec(v: &[f32]) -> Option<Self> {
        if v.len() < Self::PARAM_COUNT {
            return None;
        }
        Some(Self {
            own_life: v[0],
            enemy_life_damage: v[1],
            own_creature_attack: v[2],
            own_creature_health: v[3],
            enemy_creature_attack: v[4],
            enemy_creature_health: v[5],
            creature_count: v[6],
            board_advantage: v[7],
            cards_in_hand: v[8],
            action_points: v[9],
            keyword_guard: v[10],
            keyword_lethal: v[11],
            keyword_lifesteal: v[12],
            keyword_rush: v[13],
            keyword_ranged: v[14],
            keyword_piercing: v[15],
            keyword_shield: v[16],
            keyword_quick: v[17],
            keyword_ephemeral: v[18],
            keyword_regenerate: v[19],
            keyword_stealth: v[20],
            keyword_charge: v[21],
            keyword_frenzy: v[22],
            keyword_volatile: v[23],
            keyword_fortify: v[24],
            keyword_ward: v[25],
            win_bonus: v[26],
            lose_penalty: v[27],
        })
    }

    /// Get parameter bounds for optimization (min, max).
    pub fn bounds() -> Vec<(f32, f32)> {
        vec![
            (0.0, 5.0),    // own_life
            (0.0, 5.0),    // enemy_life_damage
            (0.0, 3.0),    // own_creature_attack
            (0.0, 3.0),    // own_creature_health
            (-3.0, 0.0),   // enemy_creature_attack
            (-3.0, 0.0),   // enemy_creature_health
            (0.0, 10.0),   // creature_count
            (0.0, 5.0),    // board_advantage
            (0.0, 3.0),    // cards_in_hand
            (-1.0, 1.0),   // action_points
            (0.0, 5.0),    // keyword_guard
            (0.0, 5.0),    // keyword_lethal
            (0.0, 5.0),    // keyword_lifesteal
            (0.0, 3.0),    // keyword_rush
            (0.0, 3.0),    // keyword_ranged
            (0.0, 3.0),    // keyword_piercing
            (0.0, 5.0),    // keyword_shield
            (0.0, 3.0),    // keyword_quick
            (-5.0, 0.0),   // keyword_ephemeral (negative - dies at end of turn)
            (0.0, 5.0),    // keyword_regenerate (positive - survivability)
            (0.0, 5.0),    // keyword_stealth (positive - evasion)
            (0.0, 5.0),    // keyword_charge (positive - burst damage)
            (0.0, 5.0),    // keyword_frenzy (positive - multi-attack aggro)
            (0.0, 5.0),    // keyword_volatile (positive - death trigger AOE)
            (0.0, 5.0),    // keyword_fortify (positive - damage reduction)
            (0.0, 5.0),    // keyword_ward (positive - spell protection)
            (100.0, 10000.0), // win_bonus
            (-10000.0, -100.0), // lose_penalty
        ]
    }

    /// Number of weight parameters.
    pub const PARAM_COUNT: usize = 28;
}

impl Default for GreedyWeights {
    fn default() -> Self {
        // Hand-tuned default weights as a starting point
        Self {
            // Life matters a lot - stay alive!
            own_life: 1.5,
            enemy_life_damage: 2.0,

            // Creature stats
            own_creature_attack: 1.2,
            own_creature_health: 1.0,
            enemy_creature_attack: -1.0,
            enemy_creature_health: -0.8,

            // Board presence
            creature_count: 3.0,
            board_advantage: 2.0,

            // Resources
            cards_in_hand: 0.5,
            action_points: 0.1,

            // Keywords - offensive keywords slightly less valuable than defensive
            keyword_guard: 3.0,
            keyword_lethal: 4.0,
            keyword_lifesteal: 3.0,
            keyword_rush: 1.5,
            keyword_ranged: 2.0,
            keyword_piercing: 2.0,
            keyword_shield: 4.0,
            keyword_quick: 1.0,

            // New keywords
            keyword_ephemeral: -2.0,  // Negative - creature dies at end of turn
            keyword_regenerate: 3.0,  // Positive - survivability
            keyword_stealth: 3.0,     // Positive - evasion
            keyword_charge: 2.0,      // Positive - burst damage

            // Symbiote keywords (v0.5.0)
            keyword_frenzy: 2.5,      // Positive - multi-attack aggro
            keyword_volatile: 3.0,    // Positive - death trigger AOE

            // Phase 5 keywords (v0.5.0)
            keyword_fortify: 3.0,     // Positive - damage reduction (similar to Shield)
            keyword_ward: 3.0,        // Positive - spell protection (similar to Stealth)

            // Win/Lose - must be high to ensure bot prioritizes winning
            win_bonus: 1000.0,
            lose_penalty: -1000.0,
        }
    }
}

/// Errors that can occur when loading/saving weights.
#[derive(Debug)]
pub enum WeightError {
    Io(String),
    Parse(String),
    Serialize(String),
}

impl std::fmt::Display for WeightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeightError::Io(e) => write!(f, "IO error: {}", e),
            WeightError::Parse(e) => write!(f, "Parse error: {}", e),
            WeightError::Serialize(e) => write!(f, "Serialize error: {}", e),
        }
    }
}

impl std::error::Error for WeightError {}
