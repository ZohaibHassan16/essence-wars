//! Card rendering utilities for ASCII display.

use cardgame::core::state::{Creature, Support};
use cardgame::CardDatabase;
use cardgame::Keywords;

/// Format a creature for display.
///
/// Returns a tuple of (name_line, stats_line).
pub fn format_creature(
    creature: &Creature,
    card_db: &CardDatabase,
    current_turn: u16,
) -> (String, String) {
    let name = card_db
        .get(creature.card_id)
        .map(|c| truncate_name(&c.name, 12))
        .unwrap_or_else(|| format!("C#{}", creature.card_id.0));

    let keywords = format_keywords(&creature.keywords);
    let stats = format!("{}/{}", creature.attack, creature.current_health);

    // Status indicators
    let mut status = String::new();
    if creature.status.is_exhausted() {
        status.push('*');
    }
    if creature.status.is_silenced() {
        status.push('!');
    }
    // Summoning sickness (not rush)
    if creature.turn_played == current_turn && !creature.keywords.has_rush() {
        status.push('~');
    }

    let name_line = name;
    let stats_line = if keywords.is_empty() {
        format!("{}{}", stats, status)
    } else {
        format!("{} [{}]{}", stats, keywords, status)
    };

    (name_line, stats_line)
}

/// Format a support for display.
pub fn format_support(support: &Support, card_db: &CardDatabase) -> (String, String) {
    let name = card_db
        .get(support.card_id)
        .map(|c| truncate_name(&c.name, 12))
        .unwrap_or_else(|| format!("S#{}", support.card_id.0));

    let durability = format!("Dur:{}", support.current_durability);

    (name, durability)
}

/// Format keywords as abbreviated string.
///
/// Abbreviations:
/// - Guard -> G
/// - Rush -> Rs
/// - Ranged -> Rn
/// - Piercing -> P
/// - Lifesteal -> Ls
/// - Lethal -> Lt
/// - Shield -> S
/// - Quick -> Q
/// - Ephemeral -> E
/// - Regenerate -> Rg
/// - Stealth -> St
/// - Charge -> C
/// - Frenzy -> Fr
/// - Volatile -> V
pub fn format_keywords(keywords: &Keywords) -> String {
    let mut parts = Vec::new();

    if keywords.has_guard() {
        parts.push("G");
    }
    if keywords.has_rush() {
        parts.push("Rs");
    }
    if keywords.has_ranged() {
        parts.push("Rn");
    }
    if keywords.has_piercing() {
        parts.push("P");
    }
    if keywords.has_lifesteal() {
        parts.push("Ls");
    }
    if keywords.has_lethal() {
        parts.push("Lt");
    }
    if keywords.has_shield() {
        parts.push("S");
    }
    if keywords.has_quick() {
        parts.push("Q");
    }
    if keywords.has_ephemeral() {
        parts.push("E");
    }
    if keywords.has_regenerate() {
        parts.push("Rg");
    }
    if keywords.has_stealth() {
        parts.push("St");
    }
    if keywords.has_charge() {
        parts.push("C");
    }
    if keywords.has_frenzy() {
        parts.push("Fr");
    }
    if keywords.has_volatile() {
        parts.push("V");
    }

    parts.join("")
}

/// Truncate a name to fit in the display.
fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}.", &name[..max_len - 1])
    }
}
