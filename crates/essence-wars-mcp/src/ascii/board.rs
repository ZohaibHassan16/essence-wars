//! ASCII board rendering for the game state.

use super::card::{format_creature, format_support};
use cardgame::core::state::GameState;
use cardgame::{CardDatabase, PlayerId, Slot};

const SLOT_WIDTH: usize = 14;
const NUM_CREATURE_SLOTS: usize = 5;
const NUM_SUPPORT_SLOTS: usize = 2;

/// Render the full game board as ASCII art.
pub fn render_board(state: &GameState, card_db: &CardDatabase, player_id: PlayerId) -> String {
    let opponent_id = player_id.opponent();
    let player_state = &state.players[player_id.index()];
    let opponent_state = &state.players[opponent_id.index()];

    let mut output = String::new();

    // Get commander names
    let opponent_commander = get_commander_name(state, opponent_id, card_db);
    let player_commander = get_commander_name(state, player_id, card_db);

    // Header
    output.push_str(&"=".repeat(92));
    output.push('\n');

    // Opponent info line
    output.push_str(&format!(
        "  OPPONENT   Life: {}/30  Essence: {}/{}  AP: {}  Deck: {}  [{}]\n",
        opponent_state.life,
        opponent_state.current_essence,
        opponent_state.max_essence,
        opponent_state.action_points,
        opponent_state.deck.len(),
        opponent_commander
    ));

    // Opponent creatures row (slots 0-4)
    let (opp_names, opp_stats) = render_creature_row(opponent_state, card_db, state.current_turn);

    // Opponent support
    let opp_support = render_support_column(opponent_state, card_db);

    output.push_str(&format!(
        "  {}  |  {}\n",
        opp_names,
        opp_support.0
    ));
    output.push_str(&format!(
        "  {}  |  {}\n",
        opp_stats,
        opp_support.1
    ));

    // Separator
    output.push_str(&"-".repeat(92));
    output.push('\n');

    // Player creatures row (slots 0-4)
    let (plr_names, plr_stats) = render_creature_row(player_state, card_db, state.current_turn);

    // Player support
    let plr_support = render_support_column(player_state, card_db);

    output.push_str(&format!(
        "  {}  |  {}\n",
        plr_names,
        plr_support.0
    ));
    output.push_str(&format!(
        "  {}  |  {}\n",
        plr_stats,
        plr_support.1
    ));

    // Player info line
    output.push_str(&format!(
        "  YOU        Life: {}/30  Essence: {}/{}  AP: {}  Deck: {}  [{}]\n",
        player_state.life,
        player_state.current_essence,
        player_state.max_essence,
        player_state.action_points,
        player_state.deck.len(),
        player_commander
    ));

    // Footer
    output.push_str(&"=".repeat(92));
    output.push('\n');

    // Legend
    output.push_str("Legend: * = exhausted, ~ = summoning sick, ! = silenced\n");
    output.push_str("Keywords: G=Guard Rs=Rush Rn=Ranged P=Piercing Ls=Lifesteal Lt=Lethal S=Shield Q=Quick\n");

    output
}

/// Render a row of creature slots.
fn render_creature_row(
    player: &cardgame::core::state::PlayerState,
    card_db: &CardDatabase,
    current_turn: u16,
) -> (String, String) {
    let mut names = Vec::new();
    let mut stats = Vec::new();

    for i in 0..NUM_CREATURE_SLOTS {
        let slot = Slot(i as u8);
        if let Some(creature) = player.get_creature(slot) {
            let (name, stat) = format_creature(creature, card_db, current_turn);
            names.push(format!("[{}]{}", i, pad_to_width(&name, SLOT_WIDTH - 3)));
            stats.push(pad_to_width(&stat, SLOT_WIDTH));
        } else {
            names.push(format!("[{}]{}", i, pad_to_width("(empty)", SLOT_WIDTH - 3)));
            stats.push(pad_to_width("", SLOT_WIDTH));
        }
    }

    (names.join(""), stats.join(""))
}

/// Render the support column (2 slots).
fn render_support_column(
    player: &cardgame::core::state::PlayerState,
    card_db: &CardDatabase,
) -> (String, String) {
    let mut names = Vec::new();
    let mut durabilities = Vec::new();

    for i in 0..NUM_SUPPORT_SLOTS {
        let slot = Slot(i as u8);
        if let Some(support) = player.get_support(slot) {
            let (name, dur) = format_support(support, card_db);
            names.push(name);
            durabilities.push(dur);
        } else {
            names.push("(empty)".to_string());
            durabilities.push("".to_string());
        }
    }

    (
        format!("Sup: {}, {}", names[0], names[1]),
        format!("     {}, {}", durabilities[0], durabilities[1]),
    )
}

/// Pad a string to a specific width.
fn pad_to_width(s: &str, width: usize) -> String {
    if s.len() >= width {
        s[..width].to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

/// Get the commander name for a player, or "No Commander" if not set.
fn get_commander_name(state: &GameState, player: PlayerId, card_db: &CardDatabase) -> String {
    state
        .get_commander(player)
        .and_then(|id| card_db.get_commander(id))
        .map(|cmd| cmd.name.clone())
        .unwrap_or_else(|| "No Commander".to_string())
}
