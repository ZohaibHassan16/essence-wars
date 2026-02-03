//! Playstyle auto-detection algorithm.
//!
//! Analyzes deck composition to determine the best-matching playstyle
//! (Aggro, Control, Tempo, Midrange) using weighted scoring.

use cardgame::{CardDatabase, CardType, Keywords};
use cardgame::types::CardId;

use super::custom_deck::{Playstyle, PlaystyleBreakdown, PlaystyleScore};

/// Weights for each scoring factor per playstyle.
/// Format: (aggro, control, tempo, midrange)
struct ScoringWeights {
    // Mana curve factors
    low_avg_cost: (f32, f32, f32, f32),      // avg < 2.5
    mid_avg_cost: (f32, f32, f32, f32),      // 2.5 <= avg <= 3.5
    high_avg_cost: (f32, f32, f32, f32),     // avg > 3.5

    // Keyword factors (per card with keyword)
    rush: (f32, f32, f32, f32),
    guard: (f32, f32, f32, f32),
    lethal: (f32, f32, f32, f32),
    quick: (f32, f32, f32, f32),
    lifesteal: (f32, f32, f32, f32),
    ward: (f32, f32, f32, f32),
    shield: (f32, f32, f32, f32),
    regenerate: (f32, f32, f32, f32),
    stealth: (f32, f32, f32, f32),
    piercing: (f32, f32, f32, f32),
    frenzy: (f32, f32, f32, f32),
    volatile: (f32, f32, f32, f32),

    // Card type ratio factors
    high_creature_ratio: (f32, f32, f32, f32), // > 70%
    high_spell_ratio: (f32, f32, f32, f32),    // > 30%
    high_support_ratio: (f32, f32, f32, f32),  // > 15%

    // Curve shape factors
    low_curve: (f32, f32, f32, f32),           // 60%+ cards cost 1-3
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            // Mana curve: aggro wants low, control wants high
            low_avg_cost: (2.0, -1.0, 1.5, 0.0),
            mid_avg_cost: (0.5, 0.0, 1.0, 1.5),
            high_avg_cost: (-1.0, 1.5, -0.5, 0.5),

            // Aggressive keywords
            rush: (0.5, -0.2, 0.3, 0.0),
            lethal: (0.4, 0.0, 0.3, 0.0),
            quick: (0.4, 0.0, 0.3, 0.0),
            stealth: (0.3, 0.0, 0.4, 0.0),
            piercing: (0.3, 0.1, 0.2, 0.1),
            frenzy: (0.3, 0.0, 0.2, 0.3),
            volatile: (0.2, 0.0, 0.1, 0.0),

            // Defensive keywords
            guard: (-0.2, 0.5, 0.0, 0.2),
            ward: (-0.2, 0.4, 0.0, 0.2),
            shield: (-0.1, 0.4, 0.1, 0.2),
            regenerate: (0.0, 0.3, 0.0, 0.4),

            // Sustain keyword
            lifesteal: (0.0, 0.3, 0.4, 0.2),

            // Card type ratios
            high_creature_ratio: (1.0, -0.5, 0.5, 0.0),
            high_spell_ratio: (-0.5, 1.0, 0.5, 0.0),
            high_support_ratio: (-0.3, 0.8, 0.0, 0.3),

            // Curve shape
            low_curve: (1.5, -0.5, 1.0, 0.0),
        }
    }
}

/// Calculate the playstyle for a deck based on its cards.
///
/// # Arguments
/// * `cards` - The card IDs in the deck (can have duplicates)
/// * `_commander_id` - The commander ID (currently unused, reserved for future)
/// * `card_db` - The card database for looking up card definitions
///
/// # Returns
/// A `PlaystyleScore` with the primary playstyle and breakdown of all scores.
pub fn calculate_playstyle(
    cards: &[u16],
    _commander_id: u16,
    card_db: &CardDatabase,
) -> PlaystyleScore {
    let weights = ScoringWeights::default();
    let mut scores = PlaystyleBreakdown::new();

    if cards.is_empty() {
        return PlaystyleScore {
            primary: Playstyle::Tempo, // Default for empty deck
            scores,
        };
    }

    // Collect card statistics
    let mut total_cost: u32 = 0;
    let mut creature_count = 0;
    let mut spell_count = 0;
    let mut support_count = 0;
    let mut low_cost_count = 0; // 1-3 cost

    // Keyword counts
    let mut rush_count = 0;
    let mut guard_count = 0;
    let mut lethal_count = 0;
    let mut quick_count = 0;
    let mut lifesteal_count = 0;
    let mut ward_count = 0;
    let mut shield_count = 0;
    let mut regenerate_count = 0;
    let mut stealth_count = 0;
    let mut piercing_count = 0;
    let mut frenzy_count = 0;
    let mut volatile_count = 0;

    for &card_id in cards {
        if let Some(card) = card_db.get(CardId(card_id)) {
            total_cost += card.cost as u32;

            if card.cost >= 1 && card.cost <= 3 {
                low_cost_count += 1;
            }

            match &card.card_type {
                CardType::Creature { .. } => {
                    creature_count += 1;
                    let keywords = card.keywords();
                    count_keywords(&keywords, &mut KeywordCounts {
                        rush: &mut rush_count,
                        guard: &mut guard_count,
                        lethal: &mut lethal_count,
                        quick: &mut quick_count,
                        lifesteal: &mut lifesteal_count,
                        ward: &mut ward_count,
                        shield: &mut shield_count,
                        regenerate: &mut regenerate_count,
                        stealth: &mut stealth_count,
                        piercing: &mut piercing_count,
                        frenzy: &mut frenzy_count,
                        volatile: &mut volatile_count,
                    });
                }
                CardType::Spell { .. } => spell_count += 1,
                CardType::Support { .. } => support_count += 1,
            }
        }
    }

    let card_count = cards.len() as f32;
    let avg_cost = total_cost as f32 / card_count;

    // Apply mana curve scoring
    if avg_cost < 2.5 {
        apply_factor(&mut scores, weights.low_avg_cost);
    } else if avg_cost <= 3.5 {
        apply_factor(&mut scores, weights.mid_avg_cost);
    } else {
        apply_factor(&mut scores, weights.high_avg_cost);
    }

    // Apply keyword scoring (per card with keyword)
    apply_keyword_factor(&mut scores, rush_count, weights.rush);
    apply_keyword_factor(&mut scores, guard_count, weights.guard);
    apply_keyword_factor(&mut scores, lethal_count, weights.lethal);
    apply_keyword_factor(&mut scores, quick_count, weights.quick);
    apply_keyword_factor(&mut scores, lifesteal_count, weights.lifesteal);
    apply_keyword_factor(&mut scores, ward_count, weights.ward);
    apply_keyword_factor(&mut scores, shield_count, weights.shield);
    apply_keyword_factor(&mut scores, regenerate_count, weights.regenerate);
    apply_keyword_factor(&mut scores, stealth_count, weights.stealth);
    apply_keyword_factor(&mut scores, piercing_count, weights.piercing);
    apply_keyword_factor(&mut scores, frenzy_count, weights.frenzy);
    apply_keyword_factor(&mut scores, volatile_count, weights.volatile);

    // Apply card type ratio scoring
    let creature_ratio = creature_count as f32 / card_count;
    let spell_ratio = spell_count as f32 / card_count;
    let support_ratio = support_count as f32 / card_count;

    if creature_ratio > 0.70 {
        apply_factor(&mut scores, weights.high_creature_ratio);
    }
    if spell_ratio > 0.30 {
        apply_factor(&mut scores, weights.high_spell_ratio);
    }
    if support_ratio > 0.15 {
        apply_factor(&mut scores, weights.high_support_ratio);
    }

    // Apply curve shape scoring
    let low_cost_ratio = low_cost_count as f32 / card_count;
    if low_cost_ratio >= 0.60 {
        apply_factor(&mut scores, weights.low_curve);
    }

    // Determine primary playstyle (ties broken by: Tempo > Midrange > Aggro > Control)
    let primary = determine_primary(&scores);

    PlaystyleScore { primary, scores }
}

/// Keyword counts struct to avoid too many arguments
struct KeywordCounts<'a> {
    rush: &'a mut i32,
    guard: &'a mut i32,
    lethal: &'a mut i32,
    quick: &'a mut i32,
    lifesteal: &'a mut i32,
    ward: &'a mut i32,
    shield: &'a mut i32,
    regenerate: &'a mut i32,
    stealth: &'a mut i32,
    piercing: &'a mut i32,
    frenzy: &'a mut i32,
    volatile: &'a mut i32,
}

/// Count keywords from a Keywords bitmask
fn count_keywords(keywords: &Keywords, counts: &mut KeywordCounts) {
    if keywords.has_rush() { *counts.rush += 1; }
    if keywords.has_guard() { *counts.guard += 1; }
    if keywords.has_lethal() { *counts.lethal += 1; }
    if keywords.has_quick() { *counts.quick += 1; }
    if keywords.has_lifesteal() { *counts.lifesteal += 1; }
    if keywords.has_ward() { *counts.ward += 1; }
    if keywords.has_shield() { *counts.shield += 1; }
    if keywords.has_regenerate() { *counts.regenerate += 1; }
    if keywords.has_stealth() { *counts.stealth += 1; }
    if keywords.has_piercing() { *counts.piercing += 1; }
    if keywords.has_frenzy() { *counts.frenzy += 1; }
    if keywords.has_volatile() { *counts.volatile += 1; }
}

/// Apply a factor tuple to scores
fn apply_factor(scores: &mut PlaystyleBreakdown, factor: (f32, f32, f32, f32)) {
    scores.aggro += factor.0;
    scores.control += factor.1;
    scores.tempo += factor.2;
    scores.midrange += factor.3;
}

/// Apply keyword factor based on count
fn apply_keyword_factor(scores: &mut PlaystyleBreakdown, count: i32, factor: (f32, f32, f32, f32)) {
    let multiplier = count as f32;
    scores.aggro += factor.0 * multiplier;
    scores.control += factor.1 * multiplier;
    scores.tempo += factor.2 * multiplier;
    scores.midrange += factor.3 * multiplier;
}

/// Determine the primary playstyle from scores.
/// Ties are broken by preference order: Tempo > Midrange > Aggro > Control
fn determine_primary(scores: &PlaystyleBreakdown) -> Playstyle {
    let mut best = Playstyle::Tempo;
    let mut best_score = scores.tempo;

    // Check in reverse preference order so higher preference wins ties
    if scores.control > best_score {
        best = Playstyle::Control;
        best_score = scores.control;
    }
    if scores.aggro > best_score {
        best = Playstyle::Aggro;
        best_score = scores.aggro;
    }
    if scores.midrange > best_score {
        best = Playstyle::Midrange;
        best_score = scores.midrange;
    }
    if scores.tempo > best_score {
        best = Playstyle::Tempo;
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_deck() {
        let card_db = CardDatabase::empty();
        let score = calculate_playstyle(&[], 5000, &card_db);
        assert_eq!(score.primary, Playstyle::Tempo);
    }

    #[test]
    fn test_determine_primary_clear_winner() {
        let scores = PlaystyleBreakdown {
            aggro: 10.0,
            control: 2.0,
            tempo: 3.0,
            midrange: 4.0,
        };
        assert_eq!(determine_primary(&scores), Playstyle::Aggro);
    }

    #[test]
    fn test_determine_primary_tie_favors_tempo() {
        let scores = PlaystyleBreakdown {
            aggro: 5.0,
            control: 5.0,
            tempo: 5.0,
            midrange: 5.0,
        };
        // Tempo wins ties
        assert_eq!(determine_primary(&scores), Playstyle::Tempo);
    }

    #[test]
    fn test_apply_factor() {
        let mut scores = PlaystyleBreakdown::new();
        apply_factor(&mut scores, (1.0, 2.0, 3.0, 4.0));
        assert_eq!(scores.aggro, 1.0);
        assert_eq!(scores.control, 2.0);
        assert_eq!(scores.tempo, 3.0);
        assert_eq!(scores.midrange, 4.0);
    }
}
