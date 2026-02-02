//! Hand rendering for displaying player's cards.

use cardgame::core::cards::{AbilityDefinition, EffectDefinition};
use cardgame::core::config::player;
use cardgame::core::effects::{TargetingRule, Trigger};
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
            CardType::Creature {
                attack,
                health,
                keywords,
                abilities,
                ..
            } => {
                output.push_str(&format!("**{}/{}**", attack, health));
                if !keywords.is_empty() {
                    output.push_str(&format!(" - {}", keywords.join(", ")));
                }
                output.push('\n');

                // Show detailed ability information
                for (idx, ability) in abilities.iter().enumerate() {
                    output.push_str(&format_ability(ability, idx));
                }
            }
            CardType::Spell { effects, .. } => {
                output.push_str("**Effects:** ");
                output.push_str(&format_effects(effects));
                output.push('\n');
            }
            CardType::Support {
                durability,
                triggered_effects,
                ..
            } => {
                output.push_str(&format!("**Durability: {}**\n", durability));
                for (idx, ability) in triggered_effects.iter().enumerate() {
                    output.push_str(&format_ability(ability, idx));
                }
            }
        }

        output.push('\n');
    }

    output
}

/// Format a single ability for display.
fn format_ability(ability: &AbilityDefinition, index: usize) -> String {
    let mut out = String::new();

    let trigger_str = match ability.trigger {
        Trigger::Activated => {
            if ability.essence_cost > 0 {
                format!("**[Activated - {} Essence]**", ability.essence_cost)
            } else {
                "**[Activated - Free]**".to_string()
            }
        }
        Trigger::OnPlay => "**[On Play]**".to_string(),
        Trigger::OnAttack => "**[On Attack]**".to_string(),
        Trigger::OnDealDamage => "**[On Deal Damage]**".to_string(),
        Trigger::OnTakeDamage => "**[On Take Damage]**".to_string(),
        Trigger::OnKill => "**[On Kill]**".to_string(),
        Trigger::OnDeath => "**[On Death]**".to_string(),
        Trigger::StartOfTurn => "**[Start of Turn]**".to_string(),
        Trigger::EndOfTurn => "**[End of Turn]**".to_string(),
        Trigger::OnAllyPlayed => "**[On Ally Played]**".to_string(),
        Trigger::OnAllyDeath => "**[On Ally Death]**".to_string(),
        Trigger::OnEnemyDeath => "**[On Enemy Death]**".to_string(),
        Trigger::OnCreaturePlayed => "**[On Creature Played]**".to_string(),
    };

    let target_str = format_targeting(&ability.targeting);
    let effects_str = format_effects(&ability.effects);

    out.push_str(&format!(
        "- Ability {}: {} {}: {}\n",
        index, trigger_str, target_str, effects_str
    ));

    out
}

/// Format targeting rule for display.
fn format_targeting(targeting: &TargetingRule) -> String {
    match targeting {
        TargetingRule::NoTarget => "".to_string(),
        TargetingRule::TargetCreature(_) => "(Target: Any Creature)".to_string(),
        TargetingRule::TargetAllyCreature => "(Target: Ally)".to_string(),
        TargetingRule::TargetEnemyCreature => "(Target: Enemy)".to_string(),
        TargetingRule::TargetPlayer => "(Target: Player)".to_string(),
        TargetingRule::TargetEnemyPlayer => "(Target: Enemy Player)".to_string(),
        TargetingRule::TargetAny => "(Target: Any)".to_string(),
        TargetingRule::TargetSlot => "(Target: Slot)".to_string(),
    }
}

/// Format a list of effects for display.
fn format_effects(effects: &[EffectDefinition]) -> String {
    effects
        .iter()
        .map(format_single_effect)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Format a single effect for display.
fn format_single_effect(effect: &EffectDefinition) -> String {
    match effect {
        EffectDefinition::Damage { amount, .. } => format!("Deal {} damage", amount),
        EffectDefinition::Heal { amount, .. } => format!("Heal {}", amount),
        EffectDefinition::Draw { count } => {
            if *count == 1 {
                "Draw a card".to_string()
            } else {
                format!("Draw {} cards", count)
            }
        }
        EffectDefinition::BuffStats { attack, health, .. } => {
            let atk = if *attack >= 0 {
                format!("+{}", attack)
            } else {
                format!("{}", attack)
            };
            let hp = if *health >= 0 {
                format!("+{}", health)
            } else {
                format!("{}", health)
            };
            format!("Give {}/{}", atk, hp)
        }
        EffectDefinition::Destroy { .. } => "Destroy".to_string(),
        EffectDefinition::GrantKeyword { keyword, .. } => format!("Grant {}", keyword),
        EffectDefinition::RemoveKeyword { keyword, .. } => format!("Remove {}", keyword),
        EffectDefinition::Silence { .. } => "Silence".to_string(),
        EffectDefinition::GainEssence { amount } => format!("Gain {} essence", amount),
        EffectDefinition::RefreshCreature => "Refresh (ready to attack)".to_string(),
        EffectDefinition::Bounce { .. } => "Return to hand".to_string(),
        EffectDefinition::SummonToken { token } => {
            format!("Summon {}/{} {}", token.attack, token.health, token.name)
        }
        EffectDefinition::Transform { into } => {
            format!("Transform into {}/{} {}", into.attack, into.health, into.name)
        }
        EffectDefinition::Copy => "Create a copy".to_string(),
    }
}
