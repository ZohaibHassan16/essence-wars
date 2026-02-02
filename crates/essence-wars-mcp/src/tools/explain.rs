//! Educational tools for explaining game rules, keywords, and cards.

use crate::session::SessionManager;
use cardgame::core::cards::{CardType, EffectDefinition, PassiveModifier};
use cardgame::types::CardId;

/// Explain game keywords.
///
/// If no keyword is specified, returns a list of all keywords.
/// If a keyword is specified, returns detailed explanation.
pub fn explain_keywords(keyword: Option<&str>) -> String {
    match keyword {
        None => list_all_keywords(),
        Some(kw) => explain_single_keyword(kw),
    }
}

fn list_all_keywords() -> String {
    let mut output = String::new();
    output.push_str("# Essence Wars Keywords\n\n");
    output.push_str("There are 16 keywords in Essence Wars, organized by when they take effect:\n\n");

    output.push_str("## Combat Keywords\n\n");
    output.push_str("| Keyword | Effect |\n");
    output.push_str("|---------|--------|\n");
    output.push_str("| **Rush** | Can attack the turn it's played (ignores summoning sickness) |\n");
    output.push_str("| **Guard** | Enemies must attack this creature first |\n");
    output.push_str("| **Ranged** | Can attack past Guard creatures |\n");
    output.push_str("| **Piercing** | Excess damage hits the enemy commander |\n");
    output.push_str("| **Lifesteal** | Heals your commander equal to damage dealt |\n");
    output.push_str("| **Lethal** | Destroys any creature it damages, regardless of health |\n");
    output.push_str("| **Quick** | Strikes first in combat; if it kills, takes no damage |\n");
    output.push_str("| **Charge** | +2 Attack on the turn it's played |\n");
    output.push_str("| **Frenzy** | Gains +1 Attack each time it attacks this turn |\n");
    output.push_str("\n");

    output.push_str("## Defensive Keywords\n\n");
    output.push_str("| Keyword | Effect |\n");
    output.push_str("|---------|--------|\n");
    output.push_str("| **Shield** | Blocks the first instance of damage, then is removed |\n");
    output.push_str("| **Stealth** | Cannot be targeted by attacks or abilities until it attacks |\n");
    output.push_str("| **Fortify** | Cannot be reduced below 1 HP by damage |\n");
    output.push_str("| **Ward** | Immune to spells and abilities (can still be attacked) |\n");
    output.push_str("\n");

    output.push_str("## Lifecycle Keywords\n\n");
    output.push_str("| Keyword | Effect |\n");
    output.push_str("|---------|--------|\n");
    output.push_str("| **Ephemeral** | Dies at the end of your turn |\n");
    output.push_str("| **Regenerate** | Heals to full health at the start of your turn |\n");
    output.push_str("| **Volatile** | When it dies, deals 2 damage to adjacent creatures |\n");
    output.push_str("\n");

    output.push_str("---\n");
    output.push_str("Use `explain_keywords <keyword>` for detailed information about a specific keyword.\n");

    output
}

fn explain_single_keyword(keyword: &str) -> String {
    let kw_lower = keyword.to_lowercase();

    match kw_lower.as_str() {
        "rush" => format_keyword_detail(
            "Rush",
            "Can attack the turn it's played",
            "Rush allows a creature to ignore summoning sickness. Normally, creatures cannot \
            attack on the turn they are played. Rush creatures can attack immediately.",
            &["Great for aggressive decks that want to apply early pressure",
              "Synergizes well with attack-triggered effects",
              "Countered by Guard creatures"],
            "Symbiote",
        ),
        "guard" => format_keyword_detail(
            "Guard",
            "Enemies must attack this creature first",
            "Guard forces enemy creatures to target this creature before they can attack other \
            creatures or your commander. Multiple Guard creatures all must be defeated before \
            non-Guard targets become available.",
            &["Essential for protecting valuable creatures",
              "Countered by Ranged (which ignores Guard)",
              "Stack multiple Guards to create a wall"],
            "Argentum",
        ),
        "ranged" => format_keyword_detail(
            "Ranged",
            "Can attack past Guard creatures",
            "Ranged allows a creature to ignore Guard and attack any enemy creature or the \
            commander directly. This makes Ranged creatures excellent for picking off key targets.",
            &["Use to snipe high-value targets behind Guards",
              "Combines well with Lethal for guaranteed kills",
              "Neutral faction specialty"],
            "Neutral",
        ),
        "piercing" => format_keyword_detail(
            "Piercing",
            "Excess damage hits the enemy commander",
            "When a Piercing creature deals more damage than needed to kill an enemy creature, \
            the excess damage is dealt to the enemy commander. Attacking the commander directly \
            with Piercing has no special effect.",
            &["High attack + Piercing = massive face damage",
              "Excellent against low-health blockers",
              "Synergizes with attack buffs"],
            "Argentum",
        ),
        "lifesteal" => format_keyword_detail(
            "Lifesteal",
            "Heals your commander equal to damage dealt",
            "When a Lifesteal creature deals damage (to creatures or commander), your commander \
            heals for the same amount. This healing can exceed your starting life total.",
            &["Sustain through aggressive matchups",
              "High attack + Lifesteal = massive healing",
              "The Blood Sovereign commander gives all creatures Lifesteal"],
            "Obsidion",
        ),
        "lethal" => format_keyword_detail(
            "Lethal",
            "Destroys any creature it damages",
            "A creature with Lethal kills any creature it deals damage to, regardless of how \
            much health that creature has. Even 1 damage from a Lethal creature is fatal.",
            &["Perfect for killing large creatures efficiently",
              "Countered by Shield (blocks the damage entirely)",
              "The Deathmaster commander gives Lethal creatures Quick"],
            "Symbiote/Obsidion",
        ),
        "shield" => format_keyword_detail(
            "Shield",
            "Blocks the first instance of damage",
            "Shield prevents the first source of damage that would be dealt to this creature, \
            then is removed. It blocks all damage from that source, regardless of amount.",
            &["Protects against Lethal (no damage = no kill)",
              "Only blocks once, then gone",
              "Refreshed by some abilities"],
            "Argentum",
        ),
        "quick" => format_keyword_detail(
            "Quick",
            "Strikes first in combat",
            "A creature with Quick deals its damage before the defender in combat. If the Quick \
            creature kills its target, it takes no damage in return. Both creatures having Quick \
            results in simultaneous damage.",
            &["Excellent for trading up against larger creatures",
              "Void Archon commander gives all creatures Quick",
              "Deathmaster gives Quick to creatures with Lethal"],
            "Obsidion",
        ),
        "ephemeral" => format_keyword_detail(
            "Ephemeral",
            "Dies at the end of your turn",
            "Ephemeral creatures are destroyed at the end of the turn they're played (or the turn \
            they gain Ephemeral). They're typically very cost-efficient but temporary.",
            &["Use for burst damage or emergency blockers",
              "Can still be sacrificed or bounced before dying",
              "Common on token creatures"],
            "Various",
        ),
        "regenerate" => format_keyword_detail(
            "Regenerate",
            "Heals to full health at the start of your turn",
            "At the start of your turn, a creature with Regenerate heals all damage and returns \
            to its maximum health. It must survive until your turn starts to benefit.",
            &["Great on high-health creatures",
              "Makes creatures very hard to remove without one-shotting",
              "Synergizes with Fortify (can't die to chip damage)"],
            "Symbiote",
        ),
        "stealth" => format_keyword_detail(
            "Stealth",
            "Cannot be targeted until it attacks",
            "A creature with Stealth cannot be targeted by attacks or abilities. Once it attacks, \
            Stealth is removed permanently. AoE effects can still hit Stealth creatures.",
            &["Guarantees at least one attack",
              "Shadow Weaver commander gives all creatures Stealth",
              "Breaks Stealth: attacking, being targeted by AoE"],
            "Obsidion",
        ),
        "charge" => format_keyword_detail(
            "Charge",
            "+2 Attack on the turn played",
            "When a creature with Charge enters the battlefield, it gains +2 Attack until end of \
            turn. This bonus is lost at the end of the turn.",
            &["Great for burst damage",
              "Combines with Rush for immediate impact",
              "The bonus applies only on the entry turn"],
            "Neutral",
        ),
        "frenzy" => format_keyword_detail(
            "Frenzy",
            "+1 Attack per attack this turn",
            "Each time a creature with Frenzy attacks, it gains +1 Attack. This stacks with \
            multiple attacks (if enabled by other effects) and resets at end of turn.",
            &["Becomes stronger with each attack",
              "Alpha of the Hunt commander triggers on attacks",
              "Stacks reset at end of turn"],
            "Symbiote",
        ),
        "volatile" => format_keyword_detail(
            "Volatile",
            "Deals damage to adjacent creatures on death",
            "When a Volatile creature dies, it deals 2 damage to creatures in adjacent slots \
            (both friendly and enemy). Position matters!",
            &["Can trigger chain reactions with other Volatile creatures",
              "Be careful of friendly fire",
              "Place strategically next to enemy creatures"],
            "Various",
        ),
        "fortify" => format_keyword_detail(
            "Fortify",
            "Cannot be reduced below 1 HP by damage",
            "A creature with Fortify cannot die from damage alone - it will always survive with \
            at least 1 HP. It can still be destroyed by Lethal or destroy effects.",
            &["Nearly unkillable by combat alone",
              "Grand Architect commander gives all creatures Fortify",
              "Countered by Lethal keyword"],
            "Argentum",
        ),
        "ward" => format_keyword_detail(
            "Ward",
            "Immune to spells and abilities",
            "A creature with Ward cannot be targeted by spells or abilities (friendly or enemy). \
            It can still be attacked normally and affected by untargeted effects.",
            &["Protects against removal spells",
              "Sanctum Healer commander gives all creatures Ward",
              "Can still be attacked and affected by AoE"],
            "Argentum",
        ),
        _ => format!(
            "Unknown keyword: '{}'\n\nUse `explain_keywords` to see all available keywords.",
            keyword
        ),
    }
}

fn format_keyword_detail(
    name: &str,
    brief: &str,
    detailed: &str,
    tips: &[&str],
    faction: &str,
) -> String {
    let mut output = String::new();
    output.push_str(&format!("# {}\n\n", name));
    output.push_str(&format!("**{}**\n\n", brief));
    output.push_str(&format!("{}\n\n", detailed));

    output.push_str("## Tips\n\n");
    for tip in tips {
        output.push_str(&format!("- {}\n", tip));
    }
    output.push_str("\n");

    output.push_str(&format!("**Primary Faction**: {}\n", faction));

    output
}

/// Explain game rules by topic.
///
/// If no topic is specified, returns a game overview.
/// Topics: "overview", "turn", "combat", "essence", "victory", "commanders", "cards"
pub fn explain_rules(topic: Option<&str>) -> String {
    match topic {
        None | Some("overview") => explain_overview(),
        Some("turn") => explain_turn_structure(),
        Some("combat") => explain_combat(),
        Some("essence") => explain_essence(),
        Some("victory") | Some("win") => explain_victory(),
        Some("commanders") | Some("commander") => explain_commanders(),
        Some("cards") | Some("card") => explain_card_types(),
        Some("actions") | Some("action") => explain_actions(),
        Some(other) => format!(
            "Unknown topic: '{}'\n\n\
            Available topics: overview, turn, combat, essence, victory, commanders, cards, actions\n\n\
            Use `explain_rules` for a general overview.",
            other
        ),
    }
}

fn explain_overview() -> String {
    r#"# Essence Wars - Game Overview

Essence Wars is a strategic card game where two commanders battle for control of essence, the magical energy that powers the world.

## Core Concepts

- **Commanders**: Each player has a commander with 30 life and a unique ability
- **Essence**: Resource used to play cards (gain 1 per turn, start with 1)
- **Action Points**: 3 per turn, used for playing cards and attacking
- **Board**: 5 creature slots + 2 support slots per player

## Win Conditions

1. **Tactical Victory**: Reduce enemy commander's life to 0
2. **Essence Extraction**: Deal 50 cumulative damage to the enemy commander
3. **Turn Limit**: After 30 turns, highest essence extracted wins

## Card Types

- **Creatures**: Attack and defend (cost AP to play)
- **Spells**: One-time effects (cost AP to play)
- **Supports**: Persistent effects with durability

## Factions

| Faction | Playstyle | Key Keywords |
|---------|-----------|--------------|
| **Argentum Combine** | Defensive, outlast | Guard, Shield, Fortify |
| **Symbiote Circles** | Aggressive tempo | Rush, Lethal, Regenerate |
| **Obsidion Syndicate** | Burst damage | Lifesteal, Stealth, Quick |
| **Neutral** | Utility | Ranged, Charge |

---
Use `explain_rules <topic>` for details: turn, combat, essence, victory, commanders, cards, actions
"#.to_string()
}

fn explain_turn_structure() -> String {
    r#"# Turn Structure

Each turn follows this sequence:

## 1. Start of Turn
- Gain 1 maximum essence (up to 10)
- Refill essence to maximum
- Gain 3 Action Points
- Creatures with **Regenerate** heal to full
- Support durability decreases by 1 (removed at 0)
- **Triggered abilities** with "Start of Turn" activate

## 2. Main Phase
You can take actions in any order:
- **Play cards** (costs AP and essence)
- **Attack** with creatures (costs 1 AP each)
- **Use abilities** (costs vary)
- **Commander's Insight** (draw 1 card for 4 essence, once per turn)

## 3. End of Turn
- Creatures with **Ephemeral** die
- **Frenzy** stacks reset
- **Triggered abilities** with "End of Turn" activate
- Turn passes to opponent

## Action Points (AP)
- Start with 3 AP per turn
- Playing a card: 1 AP
- Attacking: 1 AP
- Using an ability: varies (usually 0-1 AP)
- Commander's Insight: 0 AP (just essence cost)

## Summoning Sickness
Creatures cannot attack the turn they're played, unless they have **Rush**.
"#.to_string()
}

fn explain_combat() -> String {
    r#"# Combat System

## Basic Combat
1. Declare attacker (your creature) and defender (enemy creature or commander)
2. Both creatures deal damage equal to their Attack
3. Creatures with 0 or less health die

## Combat Keywords (in order of resolution)

### Before Damage
- **Stealth**: Can't be targeted (removed when attacking)
- **Guard**: Must be attacked before non-Guard targets
- **Ranged**: Ignores Guard, can attack any target

### During Damage
- **Quick**: Strikes first; if kill, takes no damage
- **Shield**: Blocks first damage instance
- **Fortify**: Can't be reduced below 1 HP

### After Damage
- **Lethal**: Kills any creature it damages
- **Lifesteal**: Heals your commander
- **Piercing**: Excess damage hits enemy commander

### Stacking Keywords
- Two Quick creatures = simultaneous damage
- Lethal + any damage = kill (unless Shield blocks)
- Shield blocks Lethal (no damage = no kill)

## Face Attacks
Attacking the enemy commander directly:
- Only possible if no Guard creatures present (or you have Ranged)
- Damage counts toward Essence Extraction victory
- Lifesteal heals, Piercing has no extra effect
"#.to_string()
}

fn explain_essence() -> String {
    r#"# Essence System

Essence is the resource used to play cards.

## Gaining Essence
- **Start of game**: 1 maximum essence
- **Each turn**: +1 maximum essence (caps at 10)
- **Turn start**: Essence refills to maximum

## Essence Curve
| Turn | Max Essence |
|------|-------------|
| 1 | 1 |
| 2 | 2 |
| 3 | 3 |
| ... | ... |
| 10+ | 10 |

## Spending Essence
- Each card has an **essence cost** (top-left corner)
- You also need **Action Points** to play cards
- Unused essence is lost at end of turn (refills next turn)

## Commander's Insight
- Cost: 4 essence (no AP cost)
- Effect: Draw 1 card
- Limit: Once per turn
- Purpose: Catch-up mechanic when behind

## Essence Extraction (Victory Condition)
- Track: Total damage dealt to enemy commander
- Threshold: 50 essence extracted = victory
- Applies in Essence War mode (default)
"#.to_string()
}

fn explain_victory() -> String {
    r#"# Victory Conditions

Essence Wars has three ways to win:

## 1. Tactical Victory (Life = 0)
Reduce the enemy commander's life to 0.
- Commanders start with 30 life
- Damage from creatures attacking face
- Damage from spells and abilities
- If BOTH reach 0 simultaneously = DRAW

## 2. Essence Extraction (50 damage dealt)
Deal 50 cumulative damage to the enemy commander.
- Tracks total face damage over the game
- Doesn't matter if they heal
- First to 50 wins instantly

## 3. Turn Limit (30 turns)
If neither player wins by turn 30:
- **Essence War mode**: Higher essence extracted wins
- **Attrition mode**: Higher life total wins
- Tie: Player 1 wins

## Game Modes

| Mode | Default | Turn Limit Tiebreaker |
|------|---------|----------------------|
| **Essence War** | Yes | Higher essence extracted |
| **Attrition** | No | Higher life total |

## Strategic Implications
- Aggressive decks aim for Tactical Victory or Essence Extraction
- Control decks may win via Turn Limit with life advantage
- Track both life AND damage dealt when planning
"#.to_string()
}

fn explain_commanders() -> String {
    r#"# Commander System

Each player has a commander that defines their deck's identity.

## Commander Basics
- **Location**: Command Zone (not on battlefield)
- **Life**: 30 (this IS your life total)
- **Ability**: Passive or Triggered effect
- **Targeting**: Cannot be targeted by attacks or spells directly

## Ability Types

### Passive Abilities
Always active. Examples:
- "Your creatures have +1 Attack"
- "Your creatures have Rush"
- "Your creatures have Ward"

### Triggered Abilities
Activate on specific events. Examples:
- "Start of Turn: Summon a 2/2 token"
- "When a creature dies: Deal 2 damage"
- "When a creature attacks: Give +1/+0"

## Commanders by Faction

### Argentum Combine
| Commander | Ability |
|-----------|---------|
| High Artificer | Start of Turn: Summon 2/2 Brass Cog |
| Sanctum Healer | Creatures have Ward and +0/+2 |
| Siege Marshal Vex | Creatures have +2 Attack |
| Grand Architect | Creatures have Fortify and +0/+2 |

### Symbiote Circles
| Commander | Ability |
|-----------|---------|
| Broodmother | All creatures have Rush |
| Plague Sovereign | On ally death: 2 damage to enemy |
| Alpha of the Hunt | On attack: All creatures get +1/+0 |
| Eternal Grove | Start of Turn: All creatures get +1/+1 |

### Obsidion Syndicate
| Commander | Ability |
|-----------|---------|
| Blood Sovereign | Creatures have Lifesteal and +0/+1 |
| Deathmaster | Creatures with Lethal have Quick |
| Shadow Weaver | Creatures have Stealth |
| Void Archon | Creatures have Quick |
"#.to_string()
}

fn explain_card_types() -> String {
    r#"# Card Types

## Creatures
Units that attack and defend.

**Stats**: Attack / Health
**Cost**: Essence + 1 Action Point
**Keywords**: Special abilities (Rush, Guard, etc.)

Creatures have **summoning sickness** - they can't attack the turn played (unless Rush).

## Spells
One-time effects that resolve immediately.

**Cost**: Essence + 1 Action Point
**Targeting**: May require a target (creature, player, etc.)

Spell types:
- **Damage**: Deal damage to targets
- **Buff**: Give stats or keywords
- **Removal**: Destroy or transform creatures
- **Draw**: Draw cards
- **Healing**: Restore commander life

## Supports
Persistent effects that last multiple turns.

**Cost**: Essence + 1 Action Point
**Durability**: How many turns it lasts
**Effects**: Passive bonuses or triggered abilities

Support durability decreases by 1 at the start of each of your turns.
When durability reaches 0, the support is destroyed.

## Card Rarity
| Rarity | Cards per Deck |
|--------|----------------|
| Common | Unlimited |
| Uncommon | Up to 3 |
| Rare | Up to 2 |
| Legendary | Up to 1 |
"#.to_string()
}

fn explain_actions() -> String {
    r#"# Game Actions

## Playing Cards
- **Cost**: 1 AP + card's essence cost
- **Target**: Creature -> empty slot; Spell -> per card
- **Timing**: Main phase only

## Attacking
- **Cost**: 1 AP per attack
- **Requirements**: Creature must not have summoning sickness
- **Targets**: Enemy creature or commander (respecting Guard)

## Using Abilities
- **Cost**: Varies (usually 0-1 AP)
- **Source**: Creature activated abilities
- **Targets**: Per ability description

## Commander's Insight
- **Cost**: 4 essence (0 AP)
- **Effect**: Draw 1 card
- **Limit**: Once per turn
- **When**: Any time during your turn

## End Turn
- **Cost**: 0 AP
- **Effect**: Passes turn to opponent
- **Triggers**: End of turn effects (Ephemeral dies, etc.)

## Action Economy Tips
- 3 AP = typically 3 actions per turn
- Playing + attacking with a Rush creature = 2 AP
- Save AP for responding to opponent next turn? No - no instant-speed actions!
- Use all your AP and essence each turn for maximum efficiency
"#.to_string()
}

/// Explain a specific card by ID.
pub fn explain_card(manager: &SessionManager, card_id: u16) -> String {
    let card_db = manager.card_db();

    // Try as regular card first
    if let Some(card) = card_db.get(CardId(card_id)) {
        return format_card_explanation(card);
    }

    // Try as commander
    if let Some(commander) = card_db.get_commander(CardId(card_id)) {
        return format_commander_explanation(commander);
    }

    format!(
        "Card not found: ID {}\n\n\
        Card ID ranges:\n\
        - Argentum: 1000-1074\n\
        - Symbiote: 2000-2074\n\
        - Obsidion: 3000-3074\n\
        - Neutral: 4000-4074\n\
        - Commanders: 5000-5011",
        card_id
    )
}

fn format_card_explanation(card: &cardgame::core::cards::CardDefinition) -> String {
    let mut output = String::new();

    output.push_str(&format!("# {}\n\n", card.name));
    output.push_str(&format!("**Cost**: {} essence\n", card.cost));
    output.push_str(&format!("**Rarity**: {:?}\n", card.rarity));

    match &card.card_type {
        CardType::Creature { attack, health, keywords, abilities, .. } => {
            output.push_str("**Type**: Creature\n");
            output.push_str(&format!("**Stats**: {}/{}\n", attack, health));

            if !keywords.is_empty() {
                output.push_str(&format!("**Keywords**: {}\n", keywords.join(", ")));
            }

            if !abilities.is_empty() {
                output.push_str("\n## Abilities\n\n");
                for ability in abilities {
                    output.push_str(&format!("- **{:?}** ({}): {}\n",
                        ability.trigger,
                        format_ability_cost(ability),
                        format_effects(&ability.effects)
                    ));
                }
            }
        }
        CardType::Spell { targeting, effects, .. } => {
            output.push_str("**Type**: Spell\n");
            output.push_str(&format!("**Targeting**: {:?}\n", targeting));
            output.push_str("\n## Effect\n\n");
            output.push_str(&format!("{}\n", format_effects(effects)));
        }
        CardType::Support { durability, passive_effects, triggered_effects } => {
            output.push_str("**Type**: Support\n");
            output.push_str(&format!("**Durability**: {}\n", durability));

            if !passive_effects.is_empty() {
                output.push_str("\n## Passive Effects\n\n");
                for effect in passive_effects {
                    output.push_str(&format!("- {}\n", format_passive_modifier(&effect.modifier)));
                }
            }

            if !triggered_effects.is_empty() {
                output.push_str("\n## Triggered Effects\n\n");
                for triggered in triggered_effects {
                    output.push_str(&format!("- **{:?}**: {}\n",
                        triggered.trigger,
                        format_effects(&triggered.effects)
                    ));
                }
            }
        }
    }

    if !card.tags.is_empty() {
        output.push_str(&format!("\n**Tags**: {}\n", card.tags.join(", ")));
    }

    output
}

fn format_commander_explanation(commander: &cardgame::core::cards::CommanderDefinition) -> String {
    let mut output = String::new();

    output.push_str(&format!("# {} (Commander)\n\n", commander.name));
    output.push_str(&format!("**Faction**: {:?}\n", commander.faction));
    output.push_str("**Life**: 30\n");

    if let Some(passive) = commander.passive_ability() {
        output.push_str("\n## Passive Ability\n\n");
        output.push_str(&format!("{}\n\n", passive.description));
        output.push_str(&format!("Effect: {:?}\n", passive.effect));
    }

    if let Some(triggered) = commander.triggered_ability() {
        output.push_str("\n## Triggered Ability\n\n");
        output.push_str(&format!("{}\n\n", triggered.description));
        output.push_str(&format!("**Trigger**: {:?}\n", triggered.trigger));
        output.push_str(&format!("**Effect**: {}\n", format_effects(&triggered.effects)));
    }

    if let Some(ref flavor) = commander.flavor {
        output.push_str(&format!("\n---\n*{}*\n", flavor));
    }

    output
}

fn format_passive_modifier(modifier: &PassiveModifier) -> String {
    match modifier {
        PassiveModifier::AttackBonus(n) => format!("+{} Attack to friendly creatures", n),
        PassiveModifier::HealthBonus(n) => format!("+{} Health to friendly creatures", n),
        PassiveModifier::GrantKeyword(kw) => format!("Grant {} to friendly creatures", kw),
    }
}

fn format_ability_cost(ability: &cardgame::core::cards::AbilityDefinition) -> String {
    if ability.essence_cost > 0 {
        format!("{} essence", ability.essence_cost)
    } else {
        "free".to_string()
    }
}

fn format_effects(effects: &[EffectDefinition]) -> String {
    effects.iter()
        .map(format_effect)
        .collect::<Vec<_>>()
        .join("; ")
}

fn format_effect(effect: &EffectDefinition) -> String {
    match effect {
        EffectDefinition::Damage { amount, .. } => format!("Deal {} damage", amount),
        EffectDefinition::Heal { amount, .. } => format!("Heal {}", amount),
        EffectDefinition::Draw { count } => format!("Draw {} card(s)", count),
        EffectDefinition::BuffStats { attack, health, .. } => format!("+{}/+{}", attack, health),
        EffectDefinition::GrantKeyword { keyword, .. } => format!("Grant {}", keyword),
        EffectDefinition::RemoveKeyword { keyword, .. } => format!("Remove {}", keyword),
        EffectDefinition::Destroy { .. } => "Destroy".to_string(),
        EffectDefinition::Silence { .. } => "Silence".to_string(),
        EffectDefinition::GainEssence { amount } => format!("Gain {} essence", amount),
        EffectDefinition::RefreshCreature => "Refresh creature".to_string(),
        EffectDefinition::Bounce { .. } => "Return to hand".to_string(),
        EffectDefinition::SummonToken { token } => format!("Summon {}", token.name),
        EffectDefinition::Transform { into } => format!("Transform into {}", into.name),
        EffectDefinition::Copy => "Create a copy".to_string(),
    }
}
