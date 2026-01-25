//! Data Transfer Objects for serializing game state to the frontend.

use cardgame::{
    Action, CardDatabase, CardDefinition, CardType, Creature, Faction, Keywords, Support, Target,
};
use cardgame::cards::CommanderDefinition;
use cardgame::client_api::GameEvent;
use cardgame::core::state::GameResult;
use cardgame::types::CardId;
use serde::{Deserialize, Serialize};

/// Deck information for selection screen
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Short playstyle tag (e.g., "Token Swarm", "Aggressive Piercing")
    pub playstyle: String,
    pub faction: String,
    pub card_count: usize,
    /// Commander information for this deck
    pub commander: Option<CommanderDto>,
}

/// Bot information for selection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Game configuration for starting a new game
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameConfig {
    pub player_deck_id: String,
    pub opponent_deck_id: String,
    pub opponent_bot_type: String,
    pub player_goes_first: Option<bool>,
    /// Optional seed for reproducible games (e.g., tutorials)
    pub seed: Option<u64>,
}

/// Full game state sent to the frontend
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

/// Player state (our side vs opponent side)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStateDto {
    pub life: i16,
    pub max_life: i16,
    pub essence: u8,
    pub max_essence: u8,
    pub action_points: u8,
    pub deck_count: usize,

    pub hand: Vec<CardDto>,
    pub creatures: Vec<Option<CreatureDto>>,
    pub supports: Vec<Option<SupportDto>>,

    /// Commander information (always present in a game)
    pub commander: Option<CommanderDto>,
}

/// Commander information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommanderDto {
    pub id: u16,
    pub name: String,
    pub faction: String,
    pub ability_description: String,
    /// Portrait path (relative to static folder)
    pub portrait_path: String,
}

/// Card in hand
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardDto {
    pub card_id: u16,
    pub name: String,
    pub cost: u8,
    pub card_type: String,
    pub faction: String,

    // Creature stats (if applicable)
    pub attack: Option<u8>,
    pub health: Option<u8>,
    pub keywords: Vec<String>,

    // Support stats (if applicable)
    pub durability: Option<u8>,

    // Art path (relative to assets)
    pub art_path: Option<String>,
}

/// Creature on board
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatureDto {
    pub instance_id: u32,
    pub card_id: u16,
    pub name: String,
    pub slot: u8,
    pub faction: String,

    pub attack: i8,
    pub base_attack: u8,
    pub health: i8,
    pub max_health: i8,

    pub keywords: Vec<String>,
    pub can_attack: bool,
    pub is_exhausted: bool,

    pub art_path: Option<String>,
}

/// Support on board
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportDto {
    pub card_id: u16,
    pub name: String,
    pub slot: u8,
    pub faction: String,

    pub durability: u8,

    pub art_path: Option<String>,
}

/// Action information for the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionInfo {
    pub index: u8,
    pub action_type: String,
    pub description: String,

    // For highlighting
    pub source_slot: Option<u8>,
    pub target_slot: Option<u8>,
    pub hand_index: Option<u8>,
    pub card_id: Option<u16>,
}

/// Game state update after an action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStateUpdate {
    pub state: GameStateDto,
    pub last_action: Option<ActionInfo>,
    pub events: Vec<GameEventDto>,
}

/// Game event for animation/sound triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEventDto {
    pub event_type: String,
    pub data: serde_json::Value,
}

/// Result of a completed game
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameResultDto {
    pub winner: Option<u8>,
    pub reason: String,
    pub final_turn: u16,
    pub player_final_life: i16,
    pub opponent_final_life: i16,
}

/// AI hint response with recommended action and alternatives
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiHintResponse {
    pub recommended_action: ActionInfo,
    pub score: f32,
    pub alternatives: Vec<AlternativeAction>,
    pub thinking_time_ms: u64,
}

/// An alternative action with explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlternativeAction {
    pub action: ActionInfo,
    pub score: f32,
    pub score_delta: f32,
}

// Conversion implementations

/// Derive faction from card ID (based on ID ranges in CLAUDE.md)
pub fn faction_from_card_id(card_id: u16) -> &'static str {
    match card_id {
        1000..=1999 => "argentum",
        2000..=2999 => "symbiote",
        3000..=3999 => "obsidion",
        4000..=4999 => "neutral",
        _ => "unknown",
    }
}

pub fn faction_to_string(faction: Faction) -> &'static str {
    match faction {
        Faction::Argentum => "argentum",
        Faction::Symbiote => "symbiote",
        Faction::Obsidion => "obsidion",
        Faction::Neutral => "neutral",
    }
}

/// Convert a commander ID to a CommanderDto, looking up the definition in the database
pub fn commander_to_dto(commander_id: Option<CardId>, card_db: &CardDatabase) -> Option<CommanderDto> {
    let id = commander_id?;
    let commander = card_db.get_commander(id)?;
    Some(CommanderDto::from_commander_def(commander))
}

impl CommanderDto {
    /// Create a CommanderDto from a CommanderDefinition
    pub fn from_commander_def(commander: &CommanderDefinition) -> Self {
        // Convert commander faction to string (using core::cards::Faction, not decks::Faction)
        let faction = match commander.faction {
            cardgame::core::cards::Faction::Argentum => "argentum",
            cardgame::core::cards::Faction::Symbiote => "symbiote",
            cardgame::core::cards::Faction::Obsidion => "obsidion",
            cardgame::core::cards::Faction::Neutral => "neutral",
        };

        // Get ability description from the commander's ability
        let ability_description = commander.ability.description();

        // Portrait path uses snake_case name (matching the renamed portrait files)
        let portrait_name = commander.name.to_lowercase().replace(' ', "_");
        let portrait_path = format!("portrait/{}.webp", portrait_name);

        Self {
            id: commander.id,
            name: commander.name.clone(),
            faction: faction.to_string(),
            ability_description,
            portrait_path,
        }
    }
}

pub fn card_type_to_string(card_type: &CardType) -> &'static str {
    match card_type {
        CardType::Creature { .. } => "creature",
        CardType::Spell { .. } => "spell",
        CardType::Support { .. } => "support",
    }
}

pub fn keywords_to_strings(keywords: &Keywords) -> Vec<String> {
    let mut result = Vec::new();
    if keywords.has_rush() { result.push("Rush".to_string()); }
    if keywords.has_guard() { result.push("Guard".to_string()); }
    if keywords.has_ranged() { result.push("Ranged".to_string()); }
    if keywords.has_piercing() { result.push("Piercing".to_string()); }
    if keywords.has_lifesteal() { result.push("Lifesteal".to_string()); }
    if keywords.has_lethal() { result.push("Lethal".to_string()); }
    if keywords.has_shield() { result.push("Shield".to_string()); }
    if keywords.has_quick() { result.push("Quick".to_string()); }
    if keywords.has_ephemeral() { result.push("Ephemeral".to_string()); }
    if keywords.has_regenerate() { result.push("Regenerate".to_string()); }
    if keywords.has_stealth() { result.push("Stealth".to_string()); }
    if keywords.has_charge() { result.push("Charge".to_string()); }
    if keywords.has_frenzy() { result.push("Frenzy".to_string()); }
    if keywords.has_volatile() { result.push("Volatile".to_string()); }
    result
}

impl CardDto {
    pub fn from_card_def(card: &CardDefinition) -> Self {
        let faction = faction_from_card_id(card.id);
        let card_type_str = card_type_to_string(&card.card_type);

        let keywords: Vec<String> = match &card.card_type {
            CardType::Creature { keywords, .. } => keywords.clone(),
            _ => Vec::new(),
        };

        // Art path matches files in static/cards/core_set/{id}.webp
        let art_path = Some(format!("cards/core_set/{}.webp", card.id));

        Self {
            card_id: card.id,
            name: card.name.clone(),
            cost: card.cost,
            card_type: card_type_str.to_string(),
            faction: faction.to_string(),
            attack: card.attack(),
            health: card.health(),
            keywords,
            durability: card.durability(),
            art_path,
        }
    }

    pub fn hidden() -> Self {
        Self {
            card_id: 0,
            name: "Hidden".to_string(),
            cost: 0,
            card_type: "unknown".to_string(),
            faction: "unknown".to_string(),
            attack: None,
            health: None,
            keywords: Vec::new(),
            durability: None,
            art_path: None, // Hidden cards use CSS card back design
        }
    }
}

impl CreatureDto {
    pub fn from_creature(creature: &Creature, card: &CardDefinition, current_turn: u16) -> Self {
        let faction = faction_from_card_id(card.id);
        let keywords = keywords_to_strings(&creature.keywords);

        // Art path matches files in static/cards/core_set/{id}.webp
        let art_path = Some(format!("cards/core_set/{}.webp", card.id));

        Self {
            instance_id: creature.instance_id.0,
            card_id: card.id,
            name: card.name.clone(),
            slot: creature.slot.0,
            faction: faction.to_string(),
            attack: creature.attack,
            base_attack: creature.base_attack,
            health: creature.current_health,
            max_health: creature.max_health,
            keywords,
            can_attack: creature.can_attack(current_turn),
            is_exhausted: creature.status.is_exhausted(),
            art_path,
        }
    }

    /// Create a DTO for a token creature (CardId 0)
    /// Tokens don't have card database entries, so we use the creature's stats directly
    pub fn from_token(creature: &Creature, current_turn: u16) -> Self {
        let keywords = keywords_to_strings(&creature.keywords);

        Self {
            instance_id: creature.instance_id.0,
            card_id: 0,
            name: "Token".to_string(),
            slot: creature.slot.0,
            faction: "neutral".to_string(),
            attack: creature.attack,
            base_attack: creature.base_attack,
            health: creature.current_health,
            max_health: creature.max_health,
            keywords,
            can_attack: creature.can_attack(current_turn),
            is_exhausted: creature.status.is_exhausted(),
            art_path: None, // Tokens don't have art
        }
    }
}

impl SupportDto {
    pub fn from_support(support: &Support, card: &CardDefinition) -> Self {
        let faction = faction_from_card_id(card.id);

        // Art path matches files in static/cards/core_set/{id}.webp
        let art_path = Some(format!("cards/core_set/{}.webp", card.id));

        Self {
            card_id: card.id,
            name: card.name.clone(),
            slot: support.slot.0,
            faction: faction.to_string(),
            durability: support.current_durability,
            art_path,
        }
    }
}

pub fn action_to_info(action: &Action, index: u8) -> ActionInfo {
    match action {
        Action::PlayCard { hand_index, slot } => ActionInfo {
            index,
            action_type: "play_card".to_string(),
            description: format!("Play card from hand {} to slot {}", hand_index, slot.0),
            source_slot: None,
            target_slot: Some(slot.0),
            hand_index: Some(*hand_index),
            card_id: None,
        },
        Action::Attack { attacker, defender } => ActionInfo {
            index,
            action_type: "attack".to_string(),
            description: format!("Attack slot {} with creature in slot {}", defender.0, attacker.0),
            source_slot: Some(attacker.0),
            target_slot: Some(defender.0),
            hand_index: None,
            card_id: None,
        },
        Action::UseAbility { slot, ability_index, target } => {
            let target_desc = match target {
                Target::NoTarget => "".to_string(),
                Target::EnemySlot(s) => format!(" on enemy slot {}", s.0),
                Target::Self_ => " on self".to_string(),
            };
            ActionInfo {
                index,
                action_type: "use_ability".to_string(),
                description: format!("Use ability {} from slot {}{}", ability_index, slot.0, target_desc),
                source_slot: Some(slot.0),
                target_slot: match target {
                    Target::EnemySlot(s) => Some(s.0),
                    _ => None,
                },
                hand_index: None,
                card_id: None,
            }
        }
        Action::EndTurn => ActionInfo {
            index,
            action_type: "end_turn".to_string(),
            description: "End turn".to_string(),
            source_slot: None,
            target_slot: None,
            hand_index: None,
            card_id: None,
        },
    }
}

pub fn game_event_to_dto(event: &GameEvent) -> GameEventDto {
    let (event_type, data) = match event {
        GameEvent::GameStarted { seed, mode, .. } => (
            "game_started",
            serde_json::json!({ "seed": seed, "mode": format!("{:?}", mode) }),
        ),
        GameEvent::TurnStarted { player, turn_number, .. } => (
            "turn_started",
            serde_json::json!({ "player": player.0, "turn": turn_number }),
        ),
        GameEvent::TurnEnded { player, turn_number } => (
            "turn_ended",
            serde_json::json!({ "player": player.0, "turn": turn_number }),
        ),
        GameEvent::ActionTaken { player, action, turn } => (
            "action_taken",
            serde_json::json!({
                "player": player.0,
                "action": format!("{:?}", action),
                "turn": turn
            }),
        ),
        GameEvent::CreatureSpawned { player, slot, card_id, .. } => (
            "creature_spawned",
            serde_json::json!({ "player": player.0, "slot": slot.0, "card_id": card_id.0 }),
        ),
        GameEvent::CreatureDied { player, slot, card_id, .. } => (
            "creature_died",
            serde_json::json!({ "player": player.0, "slot": slot.0, "card_id": card_id.0 }),
        ),
        GameEvent::LifeChanged { player, old_life, new_life, .. } => (
            "life_changed",
            serde_json::json!({
                "player": player.0,
                "old": old_life,
                "new": new_life
            }),
        ),
        GameEvent::GameEnded { result, final_turn } => {
            let (winner, reason) = match result {
                GameResult::Win { winner, reason } => (Some(winner.0), format!("{:?}", reason)),
                GameResult::Draw => (None, "draw".to_string()),
            };
            (
                "game_ended",
                serde_json::json!({
                    "winner": winner,
                    "reason": reason,
                    "final_turn": final_turn
                }),
            )
        }
        _ => ("unknown", serde_json::json!({})),
    };

    GameEventDto {
        event_type: event_type.to_string(),
        data,
    }
}
