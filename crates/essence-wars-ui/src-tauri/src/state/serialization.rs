//! Data Transfer Objects for serializing game state to the frontend.

use cardgame::{
    Action, CardDatabase, CardDefinition, CardType, Creature, Faction, Keywords, Support, Target,
};
use cardgame::cards::CommanderDefinition;
use cardgame::client_api::GameEvent;
use cardgame::core::cards::{AbilityDefinition, EffectDefinition, PassiveEffectDefinition, PassiveModifier};
use cardgame::core::effects::{TargetingRule, TokenAbility, TokenEffect, Trigger};
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
    /// Short playstyle tag (e.g., "Token Pack", "Aggressive Piercing")
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
    /// Essence extracted from opponent (total face damage dealt).
    /// In EssenceWar mode, reaching 50 wins the game.
    pub essence_extracted: u16,

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

    /// Effect description for support cards (None for creatures/spells)
    pub effect_description: Option<String>,
}

/// Activated ability on a creature (typically tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityDto {
    /// Ability index (0-5) for action matching
    pub index: u8,
    /// Display name (e.g., "Fungal Rot")
    pub name: String,
    /// Essence cost to activate
    pub essence_cost: u8,
    /// Targeting type: "no_target", "enemy_creature", "enemy_player", "any", etc.
    pub targeting_type: String,
    /// Human-readable effect description
    pub description: String,
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

    /// Activated abilities (empty for most creatures, populated for tokens with abilities)
    pub abilities: Vec<AbilityDto>,
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

    /// Human-readable description of what this support does
    pub effect_description: String,
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

    /// For use_ability actions: which ability (0-5)
    pub ability_index: Option<u8>,
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

        // Generate effect description for support cards
        let effect_description = match &card.card_type {
            CardType::Support { passive_effects, triggered_effects, .. } => {
                Some(describe_support_effects(passive_effects, triggered_effects))
            }
            _ => None,
        };

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
            effect_description,
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
            effect_description: None,
        }
    }
}

/// Convert a TargetingRule to a frontend-friendly string
fn targeting_rule_to_string(rule: &TargetingRule) -> &'static str {
    match rule {
        TargetingRule::NoTarget => "no_target",
        TargetingRule::TargetCreature(_) => "creature",
        TargetingRule::TargetAllyCreature => "ally_creature",
        TargetingRule::TargetEnemyCreature => "enemy_creature",
        TargetingRule::TargetPlayer => "player",
        TargetingRule::TargetEnemyPlayer => "enemy_player",
        TargetingRule::TargetAny => "any",
        TargetingRule::TargetSlot => "slot",
    }
}

/// Generate a human-readable description of token effects
fn describe_token_effects(effects: &[TokenEffect]) -> String {
    let parts: Vec<String> = effects
        .iter()
        .filter_map(|effect| match effect {
            TokenEffect::DestroySelf => None, // Don't mention self-sacrifice in description
            TokenEffect::Damage { amount } => Some(format!("{} damage", amount)),
            TokenEffect::Debuff { attack, health } => {
                if *attack != 0 && *health != 0 {
                    Some(format!("{:+}/{:+}", attack, health))
                } else if *attack != 0 {
                    Some(format!("{:+} attack", attack))
                } else {
                    Some(format!("{:+} health", health))
                }
            }
            TokenEffect::HealSelf { amount } => Some(format!("heal {} to self", amount)),
        })
        .collect();

    if parts.is_empty() {
        "Activate".to_string()
    } else {
        parts.join(", ")
    }
}

/// Convert token abilities to AbilityDto list
fn token_abilities_to_dtos(abilities: Option<&[TokenAbility]>) -> Vec<AbilityDto> {
    abilities
        .map(|abs| {
            abs.iter()
                .enumerate()
                .map(|(i, ability)| AbilityDto {
                    index: i as u8,
                    name: ability.name.clone(),
                    essence_cost: ability.essence_cost,
                    targeting_type: targeting_rule_to_string(&ability.targeting).to_string(),
                    description: describe_token_effects(&ability.effects),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Describe a trigger in human-readable form
fn trigger_to_string(trigger: &Trigger) -> &'static str {
    match trigger {
        Trigger::OnPlay => "On play",
        Trigger::OnAttack => "On attack",
        Trigger::OnDealDamage => "On deal damage",
        Trigger::OnTakeDamage => "On take damage",
        Trigger::OnKill => "On kill",
        Trigger::OnDeath => "On death",
        Trigger::StartOfTurn => "Start of turn",
        Trigger::EndOfTurn => "End of turn",
        Trigger::OnAllyPlayed => "On ally played",
        Trigger::OnAllyDeath => "On ally death",
        Trigger::OnEnemyDeath => "On enemy death",
        Trigger::OnCreaturePlayed => "On creature played",
        Trigger::Activated => "Activated",
    }
}

/// Describe an effect in human-readable form
fn describe_effect(effect: &EffectDefinition) -> String {
    match effect {
        EffectDefinition::Damage { amount, filter } => {
            let target = filter.as_ref().map_or("target", |_| "filtered targets");
            format!("Deal {} damage to {}", amount, target)
        }
        EffectDefinition::Heal { amount, filter } => {
            let target = filter.as_ref().map_or("your commander", |_| "filtered creatures");
            format!("Heal {} to {}", amount, target)
        }
        EffectDefinition::Draw { count } => {
            if *count == 1 {
                "Draw a card".to_string()
            } else {
                format!("Draw {} cards", count)
            }
        }
        EffectDefinition::BuffStats { attack, health, filter } => {
            let target = filter.as_ref().map_or("target", |_| "filtered creatures");
            format!("Give {} {:+}/{:+}", target, attack, health)
        }
        EffectDefinition::Destroy { .. } => "Destroy target".to_string(),
        EffectDefinition::GrantKeyword { keyword, .. } => {
            format!("Grant {}", keyword)
        }
        EffectDefinition::RemoveKeyword { keyword, .. } => {
            format!("Remove {}", keyword)
        }
        EffectDefinition::Silence { .. } => "Silence target".to_string(),
        EffectDefinition::GainEssence { amount } => {
            format!("Gain {} essence", amount)
        }
        EffectDefinition::RefreshCreature => "Refresh creature".to_string(),
        EffectDefinition::Bounce { .. } => "Return to hand".to_string(),
        EffectDefinition::SummonToken { token } => {
            format!("Summon a {}/{} {}", token.attack, token.health, token.name)
        }
        EffectDefinition::Transform { into } => {
            format!("Transform into {}/{} {}", into.attack, into.health, into.name)
        }
        EffectDefinition::Copy => "Create a copy".to_string(),
    }
}

/// Describe a passive modifier
fn describe_passive(passive: &PassiveEffectDefinition) -> String {
    match &passive.modifier {
        PassiveModifier::AttackBonus(amount) => {
            format!("Your creatures have {:+} Attack", amount)
        }
        PassiveModifier::HealthBonus(amount) => {
            format!("Your creatures have {:+} Health", amount)
        }
        PassiveModifier::GrantKeyword(keyword) => {
            format!("Your creatures have {}", keyword)
        }
    }
}

/// Describe a triggered ability
fn describe_triggered_ability(ability: &AbilityDefinition) -> String {
    let trigger = trigger_to_string(&ability.trigger);
    let effects: Vec<String> = ability.effects.iter().map(describe_effect).collect();
    format!("{}: {}", trigger, effects.join(", "))
}

/// Generate a human-readable description of what a support does
fn describe_support_effects(
    passives: &[PassiveEffectDefinition],
    triggered: &[AbilityDefinition],
) -> String {
    let mut parts = Vec::new();

    // Add passive effects
    for passive in passives {
        parts.push(describe_passive(passive));
    }

    // Add triggered effects
    for ability in triggered {
        parts.push(describe_triggered_ability(ability));
    }

    if parts.is_empty() {
        "No effects".to_string()
    } else {
        parts.join(". ")
    }
}

impl CreatureDto {
    pub fn from_creature(creature: &Creature, card: &CardDefinition, current_turn: u16) -> Self {
        let faction = faction_from_card_id(card.id);
        let keywords = keywords_to_strings(&creature.keywords);

        // Art path matches files in static/cards/core_set/{id}.webp
        let art_path = Some(format!("cards/core_set/{}.webp", card.id));

        // Convert token abilities to DTOs (using helper method)
        let abilities = token_abilities_to_dtos(creature.token_abilities());

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
            abilities,
        }
    }

    /// Create a DTO for a token creature (CardId 0)
    /// Tokens don't have card database entries, so we use the creature's stats directly
    pub fn from_token(creature: &Creature, current_turn: u16) -> Self {
        let keywords = keywords_to_strings(&creature.keywords);

        // Convert token abilities to DTOs (using helper method)
        let abilities = token_abilities_to_dtos(creature.token_abilities());

        // Get token name from the creature (stored during token creation)
        let name = creature
            .token_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Token".to_string());

        // Infer faction from keywords for styling
        let faction = infer_faction_from_keywords(&creature.keywords);

        // Generate art path: try named token first, then generic fallback
        let art_path = Some(token_art_path(&name));

        Self {
            instance_id: creature.instance_id.0,
            card_id: 0,
            name,
            slot: creature.slot.0,
            faction,
            attack: creature.attack,
            base_attack: creature.base_attack,
            health: creature.current_health,
            max_health: creature.max_health,
            keywords,
            can_attack: creature.can_attack(current_turn),
            is_exhausted: creature.status.is_exhausted(),
            art_path,
            abilities,
        }
    }
}

/// Infer faction from creature keywords for tokens
/// Uses the faction-characteristic keywords as hints
fn infer_faction_from_keywords(keywords: &Keywords) -> String {
    // Argentum: Guard, Piercing, Shield, Fortify
    if keywords.has_guard() || keywords.has_piercing() || keywords.has_shield() {
        return "argentum".to_string();
    }
    // Symbiote: Rush, Lethal, Regenerate, Volatile
    if keywords.has_rush() || keywords.has_lethal() || keywords.has_regenerate() {
        return "symbiote".to_string();
    }
    // Obsidion: Lifesteal, Stealth, Quick
    if keywords.has_lifesteal() || keywords.has_stealth() || keywords.has_quick() {
        return "obsidion".to_string();
    }
    "neutral".to_string()
}

/// Generate token art path (snake_case name)
fn token_art_path(name: &str) -> String {
    let snake_name = name.to_lowercase().replace(' ', "_");
    format!("tokens/named/{}.webp", snake_name)
}

/// Generate fallback art path for a faction
#[allow(dead_code)]
pub fn fallback_art_path(faction: &str) -> String {
    format!("tokens/generic/{}.webp", faction)
}

impl SupportDto {
    pub fn from_support(support: &Support, card: &CardDefinition) -> Self {
        let faction = faction_from_card_id(card.id);

        // Art path matches files in static/cards/core_set/{id}.webp
        let art_path = Some(format!("cards/core_set/{}.webp", card.id));

        // Generate effect description from the card definition
        let effect_description = match &card.card_type {
            CardType::Support { passive_effects, triggered_effects, .. } => {
                describe_support_effects(passive_effects, triggered_effects)
            }
            _ => "Unknown support".to_string(),
        };

        Self {
            card_id: card.id,
            name: card.name.clone(),
            slot: support.slot.0,
            faction: faction.to_string(),
            durability: support.current_durability,
            art_path,
            effect_description,
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
            ability_index: None,
        },
        Action::Attack { attacker, defender } => ActionInfo {
            index,
            action_type: "attack".to_string(),
            description: format!("Attack slot {} with creature in slot {}", defender.0, attacker.0),
            source_slot: Some(attacker.0),
            target_slot: Some(defender.0),
            hand_index: None,
            card_id: None,
            ability_index: None,
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
                ability_index: Some(*ability_index),
            }
        }
        Action::CommanderInsight => ActionInfo {
            index,
            action_type: "commander_insight".to_string(),
            description: "Commander's Insight - draw a card for 4 essence".to_string(),
            source_slot: None,
            target_slot: None,
            hand_index: None,
            card_id: None,
            ability_index: None,
        },
        Action::EndTurn => ActionInfo {
            index,
            action_type: "end_turn".to_string(),
            description: "End turn".to_string(),
            source_slot: None,
            target_slot: None,
            hand_index: None,
            card_id: None,
            ability_index: None,
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
        GameEvent::LifeChanged { player, old_life, new_life, source } => (
            "life_changed",
            serde_json::json!({
                "player": player.0,
                "old": old_life,
                "new": new_life,
                "source": format!("{:?}", source)
            }),
        ),
        GameEvent::KeywordActivated { player, slot, keyword, value, target_player, target_slot } => (
            "keyword_activated",
            serde_json::json!({
                "player": player.0,
                "slot": slot.0,
                "keyword": format!("{:?}", keyword),
                "value": value,
                "target_player": target_player.map(|p| p.0),
                "target_slot": target_slot.map(|s| s.0)
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
