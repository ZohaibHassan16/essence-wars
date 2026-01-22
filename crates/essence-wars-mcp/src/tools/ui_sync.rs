//! UI synchronization for pushing game state to Tauri.
//!
//! This module handles serializing MCP game state and pushing it to the
//! Tauri UI via HTTP, enabling the UI to display the current MCP game state.

use cardgame::client_api::GameClient;
use cardgame::{CardDatabase, CardType, Creature, Keywords, PlayerId, Support};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// URL for the Tauri UI sync endpoint.
const SYNC_URL: &str = "http://127.0.0.1:9999/sync_state";

/// Game state DTO matching Tauri's expected format.
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

/// Player state DTO.
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
}

/// Card in hand DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardDto {
    pub card_id: u16,
    pub name: String,
    pub cost: u8,
    pub card_type: String,
    pub faction: String,
    pub attack: Option<u8>,
    pub health: Option<u8>,
    pub keywords: Vec<String>,
    pub durability: Option<u8>,
    pub art_path: Option<String>,
}

/// Creature on board DTO.
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

/// Support on board DTO.
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

/// Game event DTO.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEventDto {
    pub event_type: String,
    pub data: serde_json::Value,
}

/// Request body for sync endpoint.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncRequest {
    state: GameStateDto,
    events: Vec<GameEventDto>,
    timestamp: u64,
}

/// Response from sync endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResponse {
    pub status: String,
    pub synced_at: u64,
}

/// Push game state to Tauri UI.
///
/// Returns Ok(()) if sync succeeds, Err with message if it fails.
/// Failures are non-fatal - the game continues even if sync fails.
pub fn push_state_to_ui(
    client: &GameClient,
    card_db: &Arc<CardDatabase>,
    player_id: PlayerId,
    game_id: &str,
) -> Result<SyncResponse, String> {
    // Verify game state exists
    client
        .get_state()
        .ok_or_else(|| "No game state available".to_string())?;

    let game_state_dto = build_game_state_dto(client, card_db, player_id, game_id);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let request = SyncRequest {
        state: game_state_dto,
        events: Vec::new(), // Events can be added later if needed
        timestamp,
    };

    // Send HTTP POST to Tauri
    let response = ureq::post(SYNC_URL)
        .header("Content-Type", "application/json")
        .send_json(&request)
        .map_err(|e| format!("Failed to sync to UI: {}", e))?;

    if response.status() != 200 {
        return Err(format!(
            "UI sync failed with status: {}",
            response.status()
        ));
    }

    let sync_response: SyncResponse = response
        .into_body()
        .read_json()
        .map_err(|e| format!("Failed to parse sync response: {}", e))?;

    Ok(sync_response)
}

/// Try to push state to UI, logging but not failing on errors.
pub fn try_push_state_to_ui(
    client: &GameClient,
    card_db: &Arc<CardDatabase>,
    player_id: PlayerId,
    game_id: &str,
) {
    match push_state_to_ui(client, card_db, player_id, game_id) {
        Ok(_) => {
            // Successfully synced
        }
        Err(e) => {
            // Log error but don't fail - UI sync is optional
            eprintln!("UI sync warning: {}", e);
        }
    }
}

/// Manually sync current game state to UI.
///
/// Returns a status message indicating success or failure.
pub fn sync_to_ui(
    client: &GameClient,
    card_db: &Arc<CardDatabase>,
    player_id: PlayerId,
    game_id: &str,
) -> String {
    match push_state_to_ui(client, card_db, player_id, game_id) {
        Ok(response) => {
            format!(
                "# UI Sync Success\n\n\
                 State synced to Tauri UI.\n\n\
                 - **Status**: {}\n\
                 - **Timestamp**: {}\n\n\
                 The Tauri UI should now display the current MCP game state.",
                response.status,
                response.synced_at
            )
        }
        Err(e) => {
            format!(
                "# UI Sync Failed\n\n\
                 Could not sync state to Tauri UI.\n\n\
                 **Error**: {}\n\n\
                 **Troubleshooting**:\n\
                 1. Is the Tauri desktop app running?\n\
                 2. Check if http://127.0.0.1:9999/health responds",
                e
            )
        }
    }
}

/// Build a GameStateDto from the current game state.
fn build_game_state_dto(
    client: &GameClient,
    card_db: &Arc<CardDatabase>,
    player_id: PlayerId,
    game_id: &str,
) -> GameStateDto {
    let state = client.get_state().unwrap();
    let opponent_id = player_id.opponent();

    let player_state = build_player_state_dto(&state.players[player_id.index()], card_db, state.current_turn, true);
    let opponent_state = build_player_state_dto(&state.players[opponent_id.index()], card_db, state.current_turn, false);

    // Determine game over status
    let (is_game_over, winner, game_over_reason) = if let Some(result) = client.get_result() {
        match result {
            cardgame::core::state::GameResult::Win { winner, reason } => {
                let winner_num = if winner == player_id { 1 } else { 2 };
                (true, Some(winner_num), Some(format!("{:?}", reason)))
            }
            cardgame::core::state::GameResult::Draw => {
                (true, None, Some("Draw".to_string()))
            }
        }
    } else {
        (false, None, None)
    };

    // Active player from player's perspective (1 = player, 2 = opponent)
    let active_player = if state.active_player == player_id { 1 } else { 2 };

    GameStateDto {
        id: game_id.to_string(),
        turn: state.current_turn,
        phase: format!("{:?}", state.phase),
        active_player,
        player: player_state,
        opponent: opponent_state,
        is_game_over,
        winner,
        game_over_reason,
    }
}

/// Build player state DTO.
fn build_player_state_dto(
    player: &cardgame::PlayerState,
    card_db: &Arc<CardDatabase>,
    current_turn: u16,
    show_hand: bool,
) -> PlayerStateDto {
    let hand = if show_hand {
        player
            .hand
            .iter()
            .map(|h| build_card_dto(h.card_id, card_db))
            .collect()
    } else {
        // Hidden hand for opponent
        player
            .hand
            .iter()
            .map(|_| CardDto {
                card_id: 0,
                name: "Hidden".to_string(),
                cost: 0,
                card_type: "unknown".to_string(),
                faction: "unknown".to_string(),
                attack: None,
                health: None,
                keywords: Vec::new(),
                durability: None,
                art_path: Some("cards/card_back.png".to_string()),
            })
            .collect()
    };

    let creatures: Vec<Option<CreatureDto>> = (0..5)
        .map(|i| {
            player
                .get_creature(cardgame::Slot(i as u8))
                .map(|c| build_creature_dto(c, card_db, current_turn))
        })
        .collect();

    let supports: Vec<Option<SupportDto>> = (0..2)
        .map(|i| {
            player
                .get_support(cardgame::Slot(i as u8))
                .map(|s| build_support_dto(s, card_db))
        })
        .collect();

    PlayerStateDto {
        life: player.life,
        max_life: 30, // Standard max life
        essence: player.current_essence,
        max_essence: player.max_essence,
        action_points: player.action_points,
        deck_count: player.deck.len(),
        hand,
        creatures,
        supports,
    }
}

/// Build card DTO from card ID.
fn build_card_dto(card_id: cardgame::CardId, card_db: &Arc<CardDatabase>) -> CardDto {
    let card = match card_db.get(card_id) {
        Some(c) => c,
        None => {
            return CardDto {
                card_id: card_id.0,
                name: "Unknown".to_string(),
                cost: 0,
                card_type: "unknown".to_string(),
                faction: "unknown".to_string(),
                attack: None,
                health: None,
                keywords: Vec::new(),
                durability: None,
                art_path: None,
            }
        }
    };

    let faction = faction_from_card_id(card_id.0);
    let card_type_str = card_type_to_string(&card.card_type);

    let keywords: Vec<String> = match &card.card_type {
        CardType::Creature { keywords, .. } => keywords.clone(),
        _ => Vec::new(),
    };

    let art_path = Some(format!(
        "cards/{}/{}_{}.png",
        faction,
        card.id,
        card.name.to_lowercase().replace(' ', "_")
    ));

    CardDto {
        card_id: card_id.0,
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

/// Build creature DTO.
fn build_creature_dto(
    creature: &Creature,
    card_db: &Arc<CardDatabase>,
    current_turn: u16,
) -> CreatureDto {
    let card = card_db.get(creature.card_id);
    let name = card.map(|c| c.name.clone()).unwrap_or_else(|| "Token".to_string());
    let faction = faction_from_card_id(creature.card_id.0);

    let art_path = card.map(|c| {
        format!(
            "cards/{}/{}_{}.png",
            faction,
            c.id,
            c.name.to_lowercase().replace(' ', "_")
        )
    });

    CreatureDto {
        instance_id: creature.instance_id.0,
        card_id: creature.card_id.0,
        name,
        slot: creature.slot.0,
        faction: faction.to_string(),
        attack: creature.attack,
        base_attack: creature.base_attack,
        health: creature.current_health,
        max_health: creature.max_health,
        keywords: keywords_to_strings(&creature.keywords),
        can_attack: creature.can_attack(current_turn),
        is_exhausted: creature.status.is_exhausted(),
        art_path,
    }
}

/// Build support DTO.
fn build_support_dto(support: &Support, card_db: &Arc<CardDatabase>) -> SupportDto {
    let card = card_db.get(support.card_id);
    let name = card.map(|c| c.name.clone()).unwrap_or_else(|| "Unknown".to_string());
    let faction = faction_from_card_id(support.card_id.0);

    let art_path = card.map(|c| {
        format!(
            "cards/{}/{}_{}.png",
            faction,
            c.id,
            c.name.to_lowercase().replace(' ', "_")
        )
    });

    SupportDto {
        card_id: support.card_id.0,
        name,
        slot: support.slot.0,
        faction: faction.to_string(),
        durability: support.current_durability,
        art_path,
    }
}

/// Get faction string from card ID.
fn faction_from_card_id(card_id: u16) -> &'static str {
    match card_id {
        1000..=1999 => "argentum",
        2000..=2999 => "symbiote",
        3000..=3999 => "obsidion",
        4000..=4999 => "neutral",
        _ => "unknown",
    }
}

/// Convert card type to string.
fn card_type_to_string(card_type: &CardType) -> &'static str {
    match card_type {
        CardType::Creature { .. } => "creature",
        CardType::Spell { .. } => "spell",
        CardType::Support { .. } => "support",
    }
}

/// Convert keywords to string list.
fn keywords_to_strings(keywords: &Keywords) -> Vec<String> {
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
