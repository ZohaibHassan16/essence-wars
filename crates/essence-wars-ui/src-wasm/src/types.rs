//! Serialization types for WASM bindings.
//!
//! These types mirror the TypeScript types in the Svelte frontend
//! and the DTOs used in the Tauri backend.

use serde::{Deserialize, Serialize};
use cardgame::cards::{CardDefinition, CommanderDefinition};
use cardgame::cards::Faction as CardFaction;
use cardgame::Faction as DeckFaction;
use cardgame::client_api::GameEvent;
use cardgame::core::state::{Creature, Support};
use cardgame::Action;

// =============================================================================
// Game Setup Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub playstyle: String,
    pub faction: String,
    pub card_count: usize,
    pub commander: Option<CommanderDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameConfig {
    pub player_deck_id: String,
    pub opponent_deck_id: String,
    pub opponent_bot_type: String,
    pub player_goes_first: Option<bool>,
    pub seed: Option<u64>,
}

// =============================================================================
// Card Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardDto {
    pub id: u16,
    pub name: String,
    pub cost: u8,
    pub card_type: String,
    pub faction: String,
    pub description: String,
    pub flavor_text: String,
    pub rarity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<i8>,
    pub keywords: Vec<String>,
    pub is_hidden: bool,
}

impl CardDto {
    pub fn from_card(card: &CardDefinition) -> Self {
        // Use methods to get optional attack/health
        let attack = card.attack().map(|a| a as i8);
        let health = card.health().map(|h| h as i8);

        // Get card type name
        let card_type = match &card.card_type {
            cardgame::cards::CardType::Creature { .. } => "Creature",
            cardgame::cards::CardType::Spell { .. } => "Spell",
            cardgame::cards::CardType::Support { .. } => "Support",
        };

        Self {
            id: card.id,
            name: card.name.clone(),
            cost: card.cost,
            card_type: card_type.to_string(),
            faction: faction_from_card_id(card.id).to_string(),
            description: String::new(), // Card descriptions are in abilities, not base card
            flavor_text: String::new(), // Not stored in CardDefinition
            rarity: format!("{:?}", card.rarity),
            attack,
            health,
            keywords: card.keywords().to_names().iter().map(|s| s.to_string()).collect(),
            is_hidden: false,
        }
    }

    pub fn hidden() -> Self {
        Self {
            id: 0,
            name: "Hidden".to_string(),
            cost: 0,
            card_type: "Unknown".to_string(),
            faction: "unknown".to_string(),
            description: String::new(),
            flavor_text: String::new(),
            rarity: "Unknown".to_string(),
            attack: None,
            health: None,
            keywords: Vec::new(),
            is_hidden: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommanderDto {
    pub id: u16,
    pub name: String,
    pub title: String,
    pub faction: String,
    pub description: String,
    pub flavor_text: String,
    pub ability_name: String,
    pub ability_description: String,
}

impl CommanderDto {
    pub fn from_commander(cmd: &CommanderDefinition) -> Self {
        // Abilities have description but not name - derive ability name from trigger/effect type
        let (ability_name, ability_description) = if let Some(passive) = cmd.passive_ability() {
            ("Passive Ability".to_string(), passive.description.clone())
        } else if let Some(triggered) = cmd.triggered_ability() {
            let trigger_name = format!("{:?}", triggered.trigger);
            (trigger_name, triggered.description.clone())
        } else {
            ("None".to_string(), "No ability".to_string())
        };

        Self {
            id: cmd.id,
            name: cmd.name.clone(),
            title: String::new(), // CommanderDefinition doesn't have title
            faction: card_faction_to_string(cmd.faction).to_string(),
            description: ability_description.clone(), // Use ability description as main description
            flavor_text: cmd.flavor.clone().unwrap_or_default(),
            ability_name,
            ability_description,
        }
    }
}

// =============================================================================
// Game State Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStateDto {
    pub id: String,
    pub turn: u16,
    pub phase: String,
    pub active_player: u8,
    pub player: PlayerStateDto,
    pub opponent: PlayerStateDto,
    pub is_game_over: bool,
    pub winner: Option<u8>,
    pub game_over_reason: Option<String>,
}

impl GameStateDto {
    pub fn empty(game_id: &str) -> Self {
        Self {
            id: game_id.to_string(),
            turn: 0,
            phase: "not_started".to_string(),
            active_player: 0,
            player: PlayerStateDto::empty(),
            opponent: PlayerStateDto::empty(),
            is_game_over: false,
            winner: None,
            game_over_reason: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStateDto {
    pub life: i16,
    pub max_life: i16,
    pub essence: u8,
    pub max_essence: u8,
    pub action_points: u8,
    pub deck_count: usize,
    pub essence_extracted: u16,
    pub hand: Vec<CardDto>,
    pub creatures: Vec<Option<CreatureDto>>,
    pub supports: Vec<Option<SupportDto>>,
    pub commander: Option<CommanderDto>,
}

impl PlayerStateDto {
    pub fn empty() -> Self {
        Self {
            life: 0,
            max_life: 30,
            essence: 0,
            max_essence: 0,
            action_points: 0,
            deck_count: 0,
            essence_extracted: 0,
            hand: Vec::new(),
            creatures: vec![None; 5],
            supports: vec![None; 2],
            commander: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatureDto {
    pub card_id: u16,
    pub name: String,
    pub attack: i8,
    pub health: i8,
    pub base_attack: i8,
    pub base_health: i8,
    pub keywords: Vec<String>,
    pub can_attack: bool,
    pub has_attacked: bool,
    pub is_exhausted: bool,
    pub slot: u8,
    pub turn_played: u16,
}

impl CreatureDto {
    pub fn from_creature(creature: &Creature, card: &CardDefinition, current_turn: u16) -> Self {
        // Use card methods to get stats
        let base_attack = card.attack().unwrap_or(0) as i8;
        let base_health = card.health().unwrap_or(0) as i8;
        let can_attack = creature.can_attack(current_turn);
        let is_exhausted = creature.status.is_exhausted();

        Self {
            card_id: card.id,
            name: card.name.clone(),
            attack: creature.attack,
            health: creature.current_health,
            base_attack,
            base_health,
            keywords: creature.keywords.to_names().iter().map(|s| s.to_string()).collect(),
            can_attack,
            has_attacked: is_exhausted, // Use exhausted state as proxy for "has attacked"
            is_exhausted,
            slot: creature.slot.0,
            turn_played: creature.turn_played,
        }
    }

    pub fn from_token(creature: &Creature, current_turn: u16) -> Self {
        let can_attack = creature.can_attack(current_turn);
        let is_exhausted = creature.status.is_exhausted();

        Self {
            card_id: 0,
            name: "Token".to_string(),
            attack: creature.attack,
            health: creature.current_health,
            base_attack: creature.base_attack as i8,
            base_health: creature.base_health as i8,
            keywords: creature.keywords.to_names().iter().map(|s| s.to_string()).collect(),
            can_attack,
            has_attacked: is_exhausted,
            is_exhausted,
            slot: creature.slot.0,
            turn_played: creature.turn_played,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportDto {
    pub card_id: u16,
    pub name: String,
    pub slot: u8,
    pub durability: Option<u8>,
    pub keywords: Vec<String>,
}

impl SupportDto {
    pub fn from_support(support: &Support, card: &CardDefinition) -> Self {
        Self {
            card_id: card.id,
            name: card.name.clone(),
            slot: support.slot.0,
            durability: Some(support.current_durability),
            keywords: Vec::new(), // Supports don't have runtime keywords like creatures
        }
    }
}

// =============================================================================
// Action Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionInfo {
    pub index: u8,
    pub action_type: String,
    pub description: String,
    pub source_slot: Option<u8>,
    pub target_slot: Option<u8>,
    pub card_index: Option<u8>,
}

pub fn action_to_info(action: &Action, index: u8) -> ActionInfo {
    match action {
        Action::PlayCard { hand_index, slot } => ActionInfo {
            index,
            action_type: "PlayCard".to_string(),
            description: format!("Play card {} to slot {}", hand_index, slot.0),
            source_slot: None,
            target_slot: Some(slot.0),
            card_index: Some(*hand_index),
        },
        Action::Attack { attacker, defender } => ActionInfo {
            index,
            action_type: "Attack".to_string(),
            description: format!("Attack from slot {} to slot {}", attacker.0, defender.0),
            source_slot: Some(attacker.0),
            target_slot: Some(defender.0),
            card_index: None,
        },
        Action::UseAbility {
            slot,
            ability_index,
            target,
        } => {
            // Convert Target enum to slot number for UI
            let target_slot = match target {
                cardgame::core::actions::Target::NoTarget => None,
                cardgame::core::actions::Target::EnemySlot(s) => Some(s.0),
                cardgame::core::actions::Target::Self_ => Some(slot.0), // Self-targeting
            };
            ActionInfo {
                index,
                action_type: "UseAbility".to_string(),
                description: format!(
                    "Use ability {} from slot {} on {:?}",
                    ability_index, slot.0, target
                ),
                source_slot: Some(slot.0),
                target_slot,
                card_index: Some(*ability_index),
            }
        },
        Action::CommanderInsight => ActionInfo {
            index,
            action_type: "CommanderInsight".to_string(),
            description: "Draw a card (Commander's Insight)".to_string(),
            source_slot: None,
            target_slot: None,
            card_index: None,
        },
        Action::EndTurn => ActionInfo {
            index,
            action_type: "EndTurn".to_string(),
            description: "End turn".to_string(),
            source_slot: None,
            target_slot: None,
            card_index: None,
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStateUpdate {
    pub state: GameStateDto,
    pub last_action: Option<ActionInfo>,
    pub events: Vec<GameEventDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEventDto {
    pub event_type: String,
    pub description: String,
}

pub fn game_event_to_dto(event: &GameEvent) -> GameEventDto {
    let (event_type, description) = match event {
        GameEvent::GameStarted { seed, .. } => {
            ("GameStarted".to_string(), format!("Game started with seed {}", seed))
        }
        GameEvent::TurnStarted { player, turn_number, .. } => {
            ("TurnStarted".to_string(), format!("Turn {} started for player {:?}", turn_number, player))
        }
        GameEvent::TurnEnded { player, turn_number } => {
            ("TurnEnded".to_string(), format!("Turn {} ended for player {:?}", turn_number, player))
        }
        GameEvent::CardDrawn { player, card_id, .. } => {
            ("CardDrawn".to_string(), format!("Player {:?} drew card {}", player, card_id.0))
        }
        GameEvent::CardPlayed { player, card_id, target_slot, .. } => (
            "CardPlayed".to_string(),
            format!("Player {:?} played card {} to slot {:?}", player, card_id.0, target_slot),
        ),
        GameEvent::CreatureSpawned { player, slot, .. } => (
            "CreatureSpawned".to_string(),
            format!("Creature spawned for player {:?} at slot {:?}", player, slot),
        ),
        GameEvent::CombatStarted { attacker_slot, defender_slot, .. } => (
            "CombatStarted".to_string(),
            format!("Combat: slot {:?} attacks slot {:?}", attacker_slot, defender_slot),
        ),
        GameEvent::CreatureDied { player, slot, .. } => (
            "CreatureDied".to_string(),
            format!("Creature died at player {:?} slot {:?}", player, slot),
        ),
        GameEvent::LifeChanged { player, old_life, new_life, .. } => (
            "LifeChanged".to_string(),
            format!("Player {:?} life changed by {} to {}", player, new_life - old_life, new_life),
        ),
        GameEvent::GameEnded { result, .. } => {
            ("GameEnded".to_string(), format!("Game ended: {:?}", result))
        }
        _ => ("Unknown".to_string(), "Unknown event".to_string()),
    };

    GameEventDto {
        event_type,
        description,
    }
}

// =============================================================================
// AI Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiHintResponse {
    pub recommended_action: ActionInfo,
    pub score: f32,
    pub alternatives: Vec<AlternativeAction>,
    pub thinking_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlternativeAction {
    pub action: ActionInfo,
    pub score: f32,
    pub score_delta: f32,
}

// =============================================================================
// Game Result Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameResultDto {
    pub winner: Option<u8>,
    pub reason: String,
    pub final_turn: u16,
    pub player_final_life: i16,
    pub opponent_final_life: i16,
}

// =============================================================================
// Helpers
// =============================================================================

pub fn card_faction_to_string(faction: CardFaction) -> &'static str {
    match faction {
        CardFaction::Argentum => "argentum",
        CardFaction::Symbiote => "symbiote",
        CardFaction::Obsidion => "obsidion",
        CardFaction::Neutral => "neutral",
    }
}

pub fn deck_faction_to_string(faction: DeckFaction) -> &'static str {
    match faction {
        DeckFaction::Argentum => "argentum",
        DeckFaction::Symbiote => "symbiote",
        DeckFaction::Obsidion => "obsidion",
        DeckFaction::Neutral => "neutral",
    }
}

/// Derive faction from card ID (based on ID ranges)
pub fn faction_from_card_id(card_id: u16) -> &'static str {
    match card_id {
        1000..=1999 => "argentum",
        2000..=2999 => "symbiote",
        3000..=3999 => "obsidion",
        4000..=4999 => "neutral",
        _ => "unknown",
    }
}

/// Parse faction from card ID
pub fn parse_faction_from_card_id(card_id: u16) -> Option<CardFaction> {
    match card_id {
        1000..=1999 => Some(CardFaction::Argentum),
        2000..=2999 => Some(CardFaction::Symbiote),
        3000..=3999 => Some(CardFaction::Obsidion),
        4000..=4999 => Some(CardFaction::Neutral),
        _ => None,
    }
}
