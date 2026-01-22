//! Hand rendering for displaying player's cards.

use cardgame::core::config::player;
use cardgame::core::state::CardInstance;
use cardgame::{CardDatabase, CardType};

/// Render the player's hand as a formatted list.
pub fn render_hand(
    hand: &arrayvec::ArrayVec<CardInstance, { player::MAX_HAND_SIZE }>,
    card_db: &CardDatabase,
    current_essence: u8,
) -> String {
    if hand.is_empty() {
        return "# Your Hand\n\nYour hand is empty.\n".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!(
        "# Your Hand ({} cards, {} essence available)\n\n",
        hand.len(),
        current_essence
    ));

    output.push_str("| # | Cost | Name | Type | Stats | Keywords | Can Play |\n");
    output.push_str("|---|------|------|------|-------|----------|----------|\n");

    for (i, card_inst) in hand.iter().enumerate() {
        let card = match card_db.get(card_inst.card_id) {
            Some(c) => c,
            None => {
                output.push_str(&format!("| {} | ? | Unknown | ? | ? | ? | ? |\n", i));
                continue;
            }
        };

        let can_play = if card.cost <= current_essence {
            "Yes"
        } else {
            "No"
        };

        let (type_str, stats, keywords) = match &card.card_type {
            CardType::Creature { attack, health, keywords, .. } => {
                let kw_str = if keywords.is_empty() {
                    "-".to_string()
                } else {
                    keywords.join(", ")
                };
                ("Creature", format!("{}/{}", attack, health), kw_str)
            }
            CardType::Support { durability, .. } => {
                ("Support", format!("Dur:{}", durability), "-".to_string())
            }
            CardType::Spell { .. } => {
                ("Spell", "-".to_string(), "-".to_string())
            }
        };

        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            i,
            card.cost,
            card.name,
            type_str,
            stats,
            keywords,
            can_play
        ));
    }

    // Add card details section
    output.push_str("\n## Card Details\n\n");

    for (i, card_inst) in hand.iter().enumerate() {
        let card = match card_db.get(card_inst.card_id) {
            Some(c) => c,
            None => continue,
        };

        output.push_str(&format!("### [{}] {} (Cost {})\n", i, card.name, card.cost));

        // Add card-specific details
        match &card.card_type {
            CardType::Creature { attack, health, keywords, abilities } => {
                output.push_str(&format!("**{}/{}**", attack, health));
                if !keywords.is_empty() {
                    output.push_str(&format!(" - {}", keywords.join(", ")));
                }
                output.push('\n');
                if !abilities.is_empty() {
                    output.push_str(&format!("*{} abilities*\n", abilities.len()));
                }
            }
            CardType::Spell { effects, .. } => {
                output.push_str(&format!("*{} effects*\n", effects.len()));
            }
            CardType::Support { durability, .. } => {
                output.push_str(&format!("**Durability: {}**\n", durability));
            }
        }

        output.push('\n');
    }

    output
}
