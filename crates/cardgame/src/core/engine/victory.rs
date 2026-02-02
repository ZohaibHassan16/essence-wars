//! Victory condition checking functions.
//!
//! Handles all win/loss/draw conditions including:
//! - Life (Tactical Stability) reaching zero → forced retreat
//! - Essence extraction threshold (50 VP in EssenceWar mode)
//! - Turn limit tiebreaker (VP in EssenceWar, life in Attrition)

use crate::core::config::game;
use crate::core::state::{GameMode, GamePhase, GameResult, GameState, WinReason};
use crate::core::types::PlayerId;

/// Check all victory conditions and update game result if needed.
///
/// Call this after each action to check for game-ending conditions.
pub fn check_victory_conditions(state: &mut GameState) {
    check_life_victory(state);
    check_essence_extraction_victory(state);
}

/// Check for turn limit victory condition.
///
/// In EssenceWar mode: Player with more essence extracted wins.
/// In Attrition mode: Player with higher life wins.
/// On tie, Player 1 wins.
pub fn check_turn_limit_victory(state: &mut GameState) {
    let winner = match state.game_mode {
        GameMode::EssenceWar => {
            // In EssenceWar, winner is determined by essence extracted (VP)
            let p1_vp = state.players[0].total_damage_dealt;
            let p2_vp = state.players[1].total_damage_dealt;
            if p1_vp >= p2_vp {
                PlayerId::PLAYER_ONE
            } else {
                PlayerId::PLAYER_TWO
            }
        }
        GameMode::Attrition => {
            // In Attrition, winner is determined by life remaining
            let p1_life = state.players[0].life;
            let p2_life = state.players[1].life;
            if p1_life >= p2_life {
                PlayerId::PLAYER_ONE
            } else {
                PlayerId::PLAYER_TWO
            }
        }
    };

    state.result = Some(GameResult::Win {
        winner,
        reason: WinReason::TurnLimitTiebreaker,
    });
    state.phase = GamePhase::Ended;
}

/// Check if a player has lost due to life reaching 0.
///
/// If both players reach 0 life simultaneously, the game is a draw.
pub fn check_life_victory(state: &mut GameState) {
    // Don't override existing result
    if state.result.is_some() {
        return;
    }

    let p1_dead = state.players[0].life <= 0;
    let p2_dead = state.players[1].life <= 0;

    if p1_dead && p2_dead {
        // Both players dead simultaneously = draw
        state.result = Some(GameResult::Draw);
        state.phase = GamePhase::Ended;
    } else if p1_dead {
        state.result = Some(GameResult::Win {
            winner: PlayerId::PLAYER_TWO,
            reason: WinReason::LifeReachedZero,
        });
        state.phase = GamePhase::Ended;
    } else if p2_dead {
        state.result = Some(GameResult::Win {
            winner: PlayerId::PLAYER_ONE,
            reason: WinReason::LifeReachedZero,
        });
        state.phase = GamePhase::Ended;
    }
}

/// Check if a player has won via Essence Extraction (EssenceWar mode only).
///
/// In EssenceWar, first player to extract 50 essence (cumulative face damage) wins.
pub fn check_essence_extraction_victory(state: &mut GameState) {
    // Only applies in EssenceWar mode
    if state.game_mode != GameMode::EssenceWar {
        return;
    }

    // Already have a result? Don't override
    if state.result.is_some() {
        return;
    }

    for player_idx in 0..2 {
        let essence_extracted = state.players[player_idx].total_damage_dealt;
        if essence_extracted >= game::VICTORY_POINTS_THRESHOLD {
            let winner = PlayerId(player_idx as u8);
            state.result = Some(GameResult::Win {
                winner,
                reason: WinReason::EssenceExtractionReached,
            });
            state.phase = GamePhase::Ended;
            return;
        }
    }
}
