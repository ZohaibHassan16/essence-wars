//! Replay manager for saving, loading, listing, and deleting replays.
//!
//! Replays are stored in `~/.essence-wars/replays/` as JSON files.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use cardgame::bots::BotType;
use cardgame::client_api::GameClient;
use cardgame::{CardDatabase, DeckRegistry, PlayerId};

use super::replay_types::{ReplayInfo, ReplayMetadata, StoredReplay};
use super::serialization::{action_to_info, game_event_to_dto, GameEventDto, GameStateDto, PlayerStateDto, CardDto, CreatureDto, SupportDto};
use super::spectator::{SpectatorAction, SpectatorConfig, SpectatorMatch, SpectatorResult};
use super::game_manager::GameSession;

/// Manager for replay file operations.
pub struct ReplayManager {
    replays_dir: PathBuf,
    card_db: Arc<CardDatabase>,
    deck_registry: Arc<DeckRegistry>,
}

// ReplayManager is Send + Sync because it only holds Arc references and PathBuf
unsafe impl Send for ReplayManager {}
unsafe impl Sync for ReplayManager {}

impl ReplayManager {
    /// Create a new ReplayManager.
    ///
    /// Creates the replays directory if it doesn't exist.
    pub fn new(card_db: Arc<CardDatabase>, deck_registry: Arc<DeckRegistry>) -> Result<Self, String> {
        // Get the replays directory: ~/.essence-wars/replays/
        let replays_dir = dirs::data_dir()
            .or_else(dirs::home_dir)
            .ok_or_else(|| "Could not determine home directory".to_string())?
            .join(".essence-wars")
            .join("replays");

        // Create directory if it doesn't exist
        fs::create_dir_all(&replays_dir)
            .map_err(|e| format!("Failed to create replays directory: {}", e))?;

        Ok(Self {
            replays_dir,
            card_db,
            deck_registry,
        })
    }

    /// Save a replay from a completed game session.
    ///
    /// Returns the path to the saved replay file.
    pub fn save_from_session(
        &self,
        session: &GameSession,
        custom_name: Option<&str>,
    ) -> Result<String, String> {
        // Get deck names
        let player_deck = self
            .deck_registry
            .get(&session.original_config.player_deck_id)
            .ok_or_else(|| "Player deck not found".to_string())?;
        let opponent_deck = self
            .deck_registry
            .get(&session.original_config.opponent_deck_id)
            .ok_or_else(|| "Opponent deck not found".to_string())?;

        // Build the SpectatorMatch by replaying the game
        let spectator_match = self.build_spectator_match_from_session(session)?;

        // Create metadata
        let metadata = ReplayMetadata::new(
            player_deck.name.clone(),
            opponent_deck.name.clone(),
            "Human".to_string(),
            Self::bot_display_name(&session.opponent_bot_type),
            spectator_match.result.winner,
            spectator_match.total_turns,
            spectator_match.actions.len(),
        );

        // Create stored replay
        let stored = StoredReplay {
            metadata,
            match_data: spectator_match,
        };

        // Generate filename
        let filename = self.generate_filename(
            &player_deck.name,
            &opponent_deck.name,
            custom_name,
        );
        let path = self.replays_dir.join(&filename);

        // Serialize and save
        let json = serde_json::to_string_pretty(&stored)
            .map_err(|e| format!("Failed to serialize replay: {}", e))?;

        fs::write(&path, json)
            .map_err(|e| format!("Failed to write replay file: {}", e))?;

        Ok(path.to_string_lossy().to_string())
    }

    /// List all saved replays.
    pub fn list_replays(&self) -> Result<Vec<ReplayInfo>, String> {
        let mut replays = Vec::new();

        let entries = fs::read_dir(&self.replays_dir)
            .map_err(|e| format!("Failed to read replays directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(info) = self.read_replay_info(&path) {
                    replays.push(info);
                }
            }
        }

        // Sort by timestamp descending (newest first)
        replays.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(replays)
    }

    /// Load a replay file and return the SpectatorMatch for playback.
    pub fn load_replay(&self, path: &str) -> Result<SpectatorMatch, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read replay file: {}", e))?;

        let stored: StoredReplay = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse replay file: {}", e))?;

        Ok(stored.match_data)
    }

    /// Delete a replay file.
    pub fn delete_replay(&self, path: &str) -> Result<(), String> {
        fs::remove_file(path)
            .map_err(|e| format!("Failed to delete replay file: {}", e))?;
        Ok(())
    }

    /// Save a SpectatorMatch directly (for AI vs AI games).
    ///
    /// Returns the path to the saved replay file.
    pub fn save_spectator_match(
        &self,
        spectator_match: &SpectatorMatch,
        custom_name: Option<&str>,
    ) -> Result<String, String> {
        // Create metadata from the match
        let metadata = ReplayMetadata::new(
            spectator_match.player1_deck_name.clone(),
            spectator_match.player2_deck_name.clone(),
            spectator_match.player1_bot_name.clone(),
            spectator_match.player2_bot_name.clone(),
            spectator_match.result.winner,
            spectator_match.total_turns,
            spectator_match.actions.len(),
        );

        // Create stored replay
        let stored = StoredReplay {
            metadata,
            match_data: spectator_match.clone(),
        };

        // Generate filename
        let filename = self.generate_filename(
            &spectator_match.player1_deck_name,
            &spectator_match.player2_deck_name,
            custom_name,
        );
        let path = self.replays_dir.join(&filename);

        // Serialize and save
        let json = serde_json::to_string_pretty(&stored)
            .map_err(|e| format!("Failed to serialize replay: {}", e))?;

        fs::write(&path, json)
            .map_err(|e| format!("Failed to write replay file: {}", e))?;

        Ok(path.to_string_lossy().to_string())
    }

    /// Build a SpectatorMatch from a GameSession by replaying the game.
    fn build_spectator_match_from_session(
        &self,
        session: &GameSession,
    ) -> Result<SpectatorMatch, String> {
        // Get decks
        let player_deck = self
            .deck_registry
            .get(&session.original_config.player_deck_id)
            .ok_or_else(|| "Player deck not found".to_string())?;
        let opponent_deck = self
            .deck_registry
            .get(&session.original_config.opponent_deck_id)
            .ok_or_else(|| "Opponent deck not found".to_string())?;

        // Determine deck order based on who went first
        let player_first = session.original_config.player_goes_first.unwrap_or(true);
        let (deck1, deck2) = if player_first {
            (player_deck, opponent_deck)
        } else {
            (opponent_deck, player_deck)
        };

        // Create new game client and start with the same seed
        let mut client = GameClient::new(self.card_db.clone());
        client.start_game(deck1, deck2, session.game_seed);

        let match_id = uuid::Uuid::new_v4().to_string();

        // Capture initial state
        let initial_state = self.client_to_spectator_dto(&client, &match_id);

        // Replay all actions, capturing state after each
        let mut actions: Vec<SpectatorAction> = Vec::new();

        for &action_index in &session.action_indices {
            let current_player = client
                .current_player()
                .ok_or_else(|| "No active player".to_string())?;
            let player_num = current_player.0 + 1;
            let turn = client.turn_number();

            // Apply action and capture events
            let events = client
                .apply_action_by_index(action_index)
                .map_err(|e| e.to_string())?;

            let action = cardgame::Action::from_index(action_index)
                .ok_or_else(|| format!("Invalid action index: {}", action_index))?;

            let action_info = action_to_info(&action, action_index);
            let event_dtos: Vec<GameEventDto> = events.iter().map(game_event_to_dto).collect();
            let state_after = self.client_to_spectator_dto(&client, &match_id);

            actions.push(SpectatorAction {
                turn,
                player: player_num,
                action: action_info,
                state_after,
                events: event_dtos,
                thinking: None, // No MCTS data for human games
                insights: None, // No AI insights for human replays
                thinking_time_ms: 0,
            });
        }

        // Get final result
        let result = client.get_result();
        let final_state = client.get_state();

        let (winner, reason) = match result {
            Some(cardgame::core::state::GameResult::Win { winner, reason }) => {
                // Convert winner based on perspective
                let winner_num = if player_first {
                    if winner == PlayerId::PLAYER_ONE { 1 } else { 2 }
                } else if winner == PlayerId::PLAYER_ONE { 2 } else { 1 };
                (Some(winner_num), format!("{:?}", reason))
            }
            Some(cardgame::core::state::GameResult::Draw) => (None, "TurnLimit".to_string()),
            None => (None, "Unknown".to_string()),
        };

        let (p1_life, p2_life) = final_state
            .map(|s| (s.players[0].life, s.players[1].life))
            .unwrap_or((0, 0));

        let spectator_result = SpectatorResult {
            winner,
            reason,
            player1_final_life: p1_life,
            player2_final_life: p2_life,
        };

        // Build SpectatorConfig
        let config = SpectatorConfig {
            player1_deck_id: if player_first {
                session.original_config.player_deck_id.clone()
            } else {
                session.original_config.opponent_deck_id.clone()
            },
            player1_bot_type: if player_first {
                "human".to_string()
            } else {
                session.original_config.opponent_bot_type.clone()
            },
            player2_deck_id: if player_first {
                session.original_config.opponent_deck_id.clone()
            } else {
                session.original_config.player_deck_id.clone()
            },
            player2_bot_type: if player_first {
                session.original_config.opponent_bot_type.clone()
            } else {
                "human".to_string()
            },
            seed: Some(session.game_seed),
            mcts_simulations: 100,
            alphabeta_depth: 4,
        };

        // Get display names
        let (player1_deck_name, player2_deck_name) = if player_first {
            (player_deck.name.clone(), opponent_deck.name.clone())
        } else {
            (opponent_deck.name.clone(), player_deck.name.clone())
        };

        let (player1_bot_name, player2_bot_name) = if player_first {
            ("Human".to_string(), Self::bot_display_name(&session.opponent_bot_type))
        } else {
            (Self::bot_display_name(&session.opponent_bot_type), "Human".to_string())
        };

        Ok(SpectatorMatch {
            id: match_id,
            config,
            initial_state,
            actions,
            result: spectator_result,
            total_turns: client.turn_number(),
            player1_deck_name,
            player2_deck_name,
            player1_bot_name,
            player2_bot_name,
            // Human replays don't have AI insights, so no eval history
            eval_history: Vec::new(),
            key_moments: Vec::new(),
        })
    }

    /// Read replay info from a file (just metadata, not full match).
    fn read_replay_info(&self, path: &PathBuf) -> Result<ReplayInfo, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let stored: StoredReplay = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse file: {}", e))?;

        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Format timestamp as human-readable date
        let date_string = Self::format_timestamp(stored.metadata.timestamp);

        Ok(ReplayInfo {
            path: path.to_string_lossy().to_string(),
            filename,
            timestamp: stored.metadata.timestamp,
            date_string,
            player1_deck_name: stored.metadata.player1_deck_name,
            player2_deck_name: stored.metadata.player2_deck_name,
            player1_type: stored.metadata.player1_type,
            player2_type: stored.metadata.player2_type,
            winner: stored.metadata.winner,
            total_turns: stored.metadata.total_turns,
            total_actions: stored.metadata.total_actions,
        })
    }

    /// Generate a filename for a replay.
    fn generate_filename(
        &self,
        player_deck: &str,
        opponent_deck: &str,
        custom_name: Option<&str>,
    ) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Format: YYYY-MM-DD_HHMM
        let datetime = Self::format_timestamp_for_filename(timestamp);

        // Sanitize deck names for filename
        let p1_safe = Self::sanitize_for_filename(player_deck);
        let p2_safe = Self::sanitize_for_filename(opponent_deck);

        if let Some(name) = custom_name {
            let name_safe = Self::sanitize_for_filename(name);
            format!("{}_{}_vs_{}_{}.replay.json", datetime, p1_safe, p2_safe, name_safe)
        } else {
            format!("{}_{}_vs_{}.replay.json", datetime, p1_safe, p2_safe)
        }
    }

    /// Format a Unix timestamp to YYYY-MM-DD_HHMM.
    fn format_timestamp_for_filename(timestamp: u64) -> String {
        use chrono::{DateTime, Utc};

        let dt = DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(Utc::now);
        dt.format("%Y-%m-%d_%H%M").to_string()
    }

    /// Format a Unix timestamp to human-readable string.
    fn format_timestamp(timestamp: u64) -> String {
        use chrono::{DateTime, Local, Utc};

        let dt = DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(Utc::now);
        let local: DateTime<Local> = DateTime::from(dt);
        local.format("%b %d, %Y %H:%M").to_string()
    }

    /// Sanitize a string for use in filenames.
    fn sanitize_for_filename(s: &str) -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .take(30)
            .collect::<String>()
            .to_lowercase()
    }

    /// Get display name for a bot type.
    fn bot_display_name(bot_type: &BotType) -> String {
        match bot_type {
            BotType::Random => "Random Bot".to_string(),
            BotType::Greedy => "Greedy Bot".to_string(),
            BotType::Mcts => "MCTS Bot".to_string(),
            BotType::AlphaBeta => "Alpha-Beta Bot".to_string(),
            BotType::AgentSpecialist(faction) => format!("Agent ({:?})", faction),
            BotType::AgentGeneralist => "Agent (Generalist)".to_string(),
        }
    }

    /// Convert GameClient state to DTO for spectator mode (both hands visible).
    fn client_to_spectator_dto(&self, client: &GameClient, game_id: &str) -> GameStateDto {
        let state = match client.get_state() {
            Some(s) => s,
            None => {
                return GameStateDto {
                    id: game_id.to_string(),
                    turn: 0,
                    phase: "not_started".to_string(),
                    active_player: 0,
                    player: PlayerStateDto {
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
                    },
                    opponent: PlayerStateDto {
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
                    },
                    is_game_over: false,
                    winner: None,
                    game_over_reason: None,
                };
            }
        };

        let player1_state = &state.players[0];
        let player2_state = &state.players[1];

        // Convert both hands (visible in spectator/replay mode)
        let player1_hand: Vec<CardDto> = player1_state
            .hand
            .iter()
            .filter_map(|card_inst| {
                self.card_db
                    .get(card_inst.card_id)
                    .map(CardDto::from_card_def)
            })
            .collect();

        let player2_hand: Vec<CardDto> = player2_state
            .hand
            .iter()
            .filter_map(|card_inst| {
                self.card_db
                    .get(card_inst.card_id)
                    .map(CardDto::from_card_def)
            })
            .collect();

        // Convert creatures to slot-indexed arrays
        let player1_creatures = self.creatures_to_slots(player1_state, state.current_turn);
        let player2_creatures = self.creatures_to_slots(player2_state, state.current_turn);

        // Convert supports to slot-indexed arrays
        let player1_supports = self.supports_to_slots(player1_state);
        let player2_supports = self.supports_to_slots(player2_state);

        let active_player = state.active_player.0;

        let result = client.get_result();
        let winner = result.and_then(|r| match r {
            cardgame::core::state::GameResult::Win { winner, .. } => Some(winner.0),
            cardgame::core::state::GameResult::Draw => None,
        });

        let game_over_reason = result.map(|r| match r {
            cardgame::core::state::GameResult::Win { reason, .. } => format!("{:?}", reason),
            cardgame::core::state::GameResult::Draw => "draw".to_string(),
        });

        GameStateDto {
            id: game_id.to_string(),
            turn: state.current_turn,
            phase: format!("{:?}", state.phase),
            active_player,
            player: PlayerStateDto {
                life: player1_state.life,
                max_life: 30,
                essence: player1_state.current_essence,
                max_essence: player1_state.max_essence,
                action_points: player1_state.action_points,
                deck_count: player1_state.deck.len(),
                essence_extracted: player1_state.total_damage_dealt,
                hand: player1_hand,
                creatures: player1_creatures,
                supports: player1_supports,
                commander: None, // TODO: Load commander from game state
            },
            opponent: PlayerStateDto {
                life: player2_state.life,
                max_life: 30,
                essence: player2_state.current_essence,
                max_essence: player2_state.max_essence,
                action_points: player2_state.action_points,
                deck_count: player2_state.deck.len(),
                essence_extracted: player2_state.total_damage_dealt,
                hand: player2_hand,
                creatures: player2_creatures,
                supports: player2_supports,
                commander: None, // TODO: Load commander from game state
            },
            is_game_over: client.is_game_over(),
            winner,
            game_over_reason,
        }
    }

    /// Convert creatures to slot-indexed array.
    fn creatures_to_slots(
        &self,
        player_state: &cardgame::core::state::PlayerState,
        current_turn: u16,
    ) -> Vec<Option<CreatureDto>> {
        let mut slots: Vec<Option<CreatureDto>> = vec![None; 5];
        for creature in player_state.creatures.iter() {
            if creature.card_id.0 == 0 {
                let dto = CreatureDto::from_token(creature, current_turn);
                slots[creature.slot.0 as usize] = Some(dto);
            } else {
                match self.card_db.get(creature.card_id) {
                    Some(card) => {
                        let dto = CreatureDto::from_creature(creature, card, current_turn);
                        slots[creature.slot.0 as usize] = Some(dto);
                    }
                    None => {
                        eprintln!(
                            "Warning: Creature card ID {} not found in database",
                            creature.card_id.0
                        );
                    }
                }
            }
        }
        slots
    }

    /// Convert supports to slot-indexed array.
    fn supports_to_slots(
        &self,
        player_state: &cardgame::core::state::PlayerState,
    ) -> Vec<Option<SupportDto>> {
        let mut slots: Vec<Option<SupportDto>> = vec![None; 2];
        for support in player_state.supports.iter() {
            match self.card_db.get(support.card_id) {
                Some(card) => {
                    let dto = SupportDto::from_support(support, card);
                    slots[support.slot.0 as usize] = Some(dto);
                }
                None => {
                    eprintln!(
                        "Warning: Support card ID {} not found in database",
                        support.card_id.0
                    );
                }
            }
        }
        slots
    }
}
