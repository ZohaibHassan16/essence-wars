//! Discovery tools for listing available decks and bots.

use crate::session::SessionManager;
use cardgame::Faction;
use std::collections::BTreeMap;

/// Deck info for display.
struct DeckInfo {
    id: String,
    name: String,
    description: String,
    playstyle: String,
    commander_name: String,
}

/// List all available decks grouped by faction.
pub fn list_decks(manager: &SessionManager) -> String {
    let registry = manager.deck_registry();
    let card_db = manager.card_db();

    // Group decks by faction
    let mut by_faction: BTreeMap<String, Vec<DeckInfo>> = BTreeMap::new();

    for deck in registry.decks() {
        let faction_name = deck
            .faction()
            .map(faction_display_name)
            .unwrap_or_else(|| "Neutral".to_string());

        // Get commander name
        let commander_name = card_db
            .get_commander(deck.commander_id())
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        by_faction.entry(faction_name).or_default().push(DeckInfo {
            id: deck.id.clone(),
            name: deck.name.clone(),
            description: deck.description.clone(),
            playstyle: deck.playstyle.clone(),
            commander_name,
        });
    }

    let mut output = String::new();
    output.push_str("# Available Decks\n\n");

    for (faction, decks) in by_faction {
        output.push_str(&format!("## {} ({})\n\n", faction, decks.len()));

        for deck in decks {
            output.push_str(&format!(
                "- **{}** (`{}`) - Commander: **{}**",
                deck.name, deck.id, deck.commander_name
            ));
            if !deck.playstyle.is_empty() {
                output.push_str(&format!(" [{}]", deck.playstyle));
            }
            if !deck.description.is_empty() {
                output.push_str(&format!("\n  {}", deck.description));
            }
            output.push('\n');
        }
        output.push('\n');
    }

    output
}

/// List available bot types with descriptions.
pub fn list_bots() -> String {
    let mut output = String::new();
    output.push_str("# Available AI Opponents\n\n");

    output.push_str("| Bot Type | Difficulty | Description |\n");
    output.push_str("|----------|------------|-------------|\n");
    output.push_str("| `random` | Easy | Makes completely random moves. Good for testing. |\n");
    output.push_str("| `greedy` | Medium | Evaluates each move and picks the best immediate option. |\n");
    output.push_str("| `mcts` | Hard | Uses Monte Carlo Tree Search with 100 simulations by default. |\n");
    output.push_str("| `alphabeta` | Hard | Uses minimax search with alpha-beta pruning (depth 6). |\n");

    output.push_str("\n## Bot Details\n\n");

    output.push_str("### Random Bot\n");
    output.push_str("Selects uniformly at random from legal actions. ");
    output.push_str("Useful for basic testing and seeing card interactions.\n\n");

    output.push_str("### Greedy Bot\n");
    output.push_str("Uses a heuristic evaluation function to score each legal action ");
    output.push_str("and picks the highest-scoring move. Fast and provides decent opposition.\n\n");

    output.push_str("### MCTS Bot\n");
    output.push_str("Monte Carlo Tree Search with UCB1 selection. ");
    output.push_str("Runs simulations to estimate action values. ");
    output.push_str("Stronger than greedy but takes more time.\n\n");

    output.push_str("### Alpha-Beta Bot\n");
    output.push_str("Classic minimax search with alpha-beta pruning. ");
    output.push_str("Looks ahead multiple turns. Very strong at depth 8+.\n");

    output
}

fn faction_display_name(faction: Faction) -> String {
    match faction {
        Faction::Argentum => "Argentum Combine".to_string(),
        Faction::Symbiote => "Symbiote Circles".to_string(),
        Faction::Obsidion => "Obsidion Syndicate".to_string(),
        Faction::Neutral => "Free-Walkers (Neutral)".to_string(),
    }
}
