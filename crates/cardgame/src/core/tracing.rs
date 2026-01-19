//! Tracing infrastructure for debugging combat and effect resolution.
//!
//! This module provides:
//! - CombatTracer: Step-by-step combat keyword resolution tracing
//! - EffectTracer: Effect queue processing tracing
//!
//! These tracers are optional and can be enabled via CLI flags for debugging.

use crate::core::effects::{Effect, EffectSource, EffectTarget};
use crate::core::keywords::Keywords;
use crate::core::types::{PlayerId, Slot};

// ============================================================================
// Combat Tracing
// ============================================================================

/// Phase of combat resolution
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CombatPhase {
    /// Initial combat setup
    Setup,
    /// Checking for Quick keyword
    QuickCheck,
    /// Quick strike damage (first striker attacks)
    QuickStrike,
    /// Checking for Shield keyword
    ShieldCheck,
    /// Shield absorption
    ShieldAbsorption,
    /// Main damage calculation
    DamageCalculation,
    /// Counter-attack damage
    CounterAttack,
    /// Checking for Lethal keyword
    LethalCheck,
    /// Lethal instant kill triggered
    LethalTrigger,
    /// Checking for Piercing keyword
    PiercingCheck,
    /// Piercing overflow damage to face
    PiercingOverflow,
    /// Checking for Lifesteal keyword
    LifestealCheck,
    /// Lifesteal healing applied
    LifestealHeal,
    /// Ranged attack (no counter)
    RangedAttack,
    /// Face attack (empty defender slot)
    FaceAttack,
    /// Combat complete
    Complete,
}

impl std::fmt::Display for CombatPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CombatPhase::Setup => write!(f, "SETUP"),
            CombatPhase::QuickCheck => write!(f, "QUICK_CHECK"),
            CombatPhase::QuickStrike => write!(f, "QUICK_STRIKE"),
            CombatPhase::ShieldCheck => write!(f, "SHIELD_CHECK"),
            CombatPhase::ShieldAbsorption => write!(f, "SHIELD_ABSORB"),
            CombatPhase::DamageCalculation => write!(f, "DAMAGE_CALC"),
            CombatPhase::CounterAttack => write!(f, "COUNTER"),
            CombatPhase::LethalCheck => write!(f, "LETHAL_CHECK"),
            CombatPhase::LethalTrigger => write!(f, "LETHAL_KILL"),
            CombatPhase::PiercingCheck => write!(f, "PIERCE_CHECK"),
            CombatPhase::PiercingOverflow => write!(f, "PIERCE_OVERFLOW"),
            CombatPhase::LifestealCheck => write!(f, "LIFESTEAL_CHECK"),
            CombatPhase::LifestealHeal => write!(f, "LIFESTEAL_HEAL"),
            CombatPhase::RangedAttack => write!(f, "RANGED"),
            CombatPhase::FaceAttack => write!(f, "FACE_ATTACK"),
            CombatPhase::Complete => write!(f, "COMPLETE"),
        }
    }
}

/// A single step in combat resolution
#[derive(Clone, Debug)]
pub struct CombatStep {
    /// Phase of this step
    pub phase: CombatPhase,
    /// Description of what happened
    pub description: String,
    /// Attacker's keywords at this step
    pub attacker_keywords: Keywords,
    /// Defender's keywords at this step (if applicable)
    pub defender_keywords: Option<Keywords>,
    /// Damage dealt in this step
    pub damage_dealt: u8,
    /// Healing applied in this step
    pub healing: u8,
    /// Whether a creature died
    pub death: Option<String>,
}

impl CombatStep {
    /// Create a new combat step
    pub fn new(phase: CombatPhase, description: impl Into<String>) -> Self {
        Self {
            phase,
            description: description.into(),
            attacker_keywords: Keywords::none(),
            defender_keywords: None,
            damage_dealt: 0,
            healing: 0,
            death: None,
        }
    }

    /// Add keyword info
    pub fn with_keywords(mut self, attacker: Keywords, defender: Option<Keywords>) -> Self {
        self.attacker_keywords = attacker;
        self.defender_keywords = defender;
        self
    }

    /// Add damage info
    pub fn with_damage(mut self, damage: u8) -> Self {
        self.damage_dealt = damage;
        self
    }

    /// Add healing info
    pub fn with_healing(mut self, healing: u8) -> Self {
        self.healing = healing;
        self
    }

    /// Add death info
    pub fn with_death(mut self, who: impl Into<String>) -> Self {
        self.death = Some(who.into());
        self
    }
}

/// Traces a complete combat resolution
#[derive(Clone, Debug)]
pub struct CombatTrace {
    /// Attacker info
    pub attacker_owner: PlayerId,
    pub attacker_slot: Slot,
    pub attacker_attack: i8,
    pub attacker_health: i8,
    pub attacker_keywords: Keywords,

    /// Defender info (if not face attack)
    pub defender_owner: Option<PlayerId>,
    pub defender_slot: Slot,
    pub defender_attack: Option<i8>,
    pub defender_health: Option<i8>,
    pub defender_keywords: Option<Keywords>,

    /// Is this a face attack?
    pub is_face_attack: bool,

    /// Steps in the combat resolution
    pub steps: Vec<CombatStep>,

    /// Final results
    pub attacker_died: bool,
    pub defender_died: bool,
    pub face_damage: u8,
    pub attacker_healed: u8,
}

impl CombatTrace {
    /// Create a new combat trace for a creature vs creature fight
    #[allow(clippy::too_many_arguments)]
    pub fn new_creature_combat(
        attacker_owner: PlayerId,
        attacker_slot: Slot,
        attacker_attack: i8,
        attacker_health: i8,
        attacker_keywords: Keywords,
        defender_owner: PlayerId,
        defender_slot: Slot,
        defender_attack: i8,
        defender_health: i8,
        defender_keywords: Keywords,
    ) -> Self {
        let mut trace = Self {
            attacker_owner,
            attacker_slot,
            attacker_attack,
            attacker_health,
            attacker_keywords,
            defender_owner: Some(defender_owner),
            defender_slot,
            defender_attack: Some(defender_attack),
            defender_health: Some(defender_health),
            defender_keywords: Some(defender_keywords),
            is_face_attack: false,
            steps: Vec::new(),
            attacker_died: false,
            defender_died: false,
            face_damage: 0,
            attacker_healed: 0,
        };

        // Add setup step
        trace.add_step(CombatStep::new(
            CombatPhase::Setup,
            format!(
                "Combat: P{} Slot {} ({}/{} {:?}) vs P{} Slot {} ({}/{} {:?})",
                attacker_owner.index() + 1,
                attacker_slot.0,
                attacker_attack,
                attacker_health,
                attacker_keywords.to_names(),
                defender_owner.index() + 1,
                defender_slot.0,
                defender_attack,
                defender_health,
                defender_keywords.to_names()
            ),
        ).with_keywords(attacker_keywords, Some(defender_keywords)));

        trace
    }

    /// Create a new combat trace for a face attack
    pub fn new_face_attack(
        attacker_owner: PlayerId,
        attacker_slot: Slot,
        attacker_attack: i8,
        attacker_health: i8,
        attacker_keywords: Keywords,
        defender_slot: Slot,
    ) -> Self {
        let mut trace = Self {
            attacker_owner,
            attacker_slot,
            attacker_attack,
            attacker_health,
            attacker_keywords,
            defender_owner: None,
            defender_slot,
            defender_attack: None,
            defender_health: None,
            defender_keywords: None,
            is_face_attack: true,
            steps: Vec::new(),
            attacker_died: false,
            defender_died: false,
            face_damage: 0,
            attacker_healed: 0,
        };

        // Add setup step
        trace.add_step(CombatStep::new(
            CombatPhase::Setup,
            format!(
                "Face Attack: P{} Slot {} ({}/{} {:?}) -> enemy face",
                attacker_owner.index() + 1,
                attacker_slot.0,
                attacker_attack,
                attacker_health,
                attacker_keywords.to_names()
            ),
        ).with_keywords(attacker_keywords, None));

        trace
    }

    /// Add a step to the trace
    pub fn add_step(&mut self, step: CombatStep) {
        self.steps.push(step);
    }

    /// Log Quick keyword check
    pub fn log_quick_check(&mut self, attacker_has_quick: bool, defender_has_quick: bool) {
        let desc = match (attacker_has_quick, defender_has_quick) {
            (true, true) => "Both have Quick - simultaneous strikes".to_string(),
            (true, false) => "Attacker has Quick - strikes first".to_string(),
            (false, true) => "Defender has Quick - counter-attacks first".to_string(),
            (false, false) => "Neither has Quick - normal combat".to_string(),
        };
        self.add_step(CombatStep::new(CombatPhase::QuickCheck, desc));
    }

    /// Log Quick strike damage
    pub fn log_quick_strike(&mut self, striker: &str, damage: u8, target_died: bool) {
        let mut step = CombatStep::new(
            CombatPhase::QuickStrike,
            format!("{} Quick strike deals {} damage", striker, damage),
        ).with_damage(damage);

        if target_died {
            step = step.with_death(if striker == "Attacker" { "Defender" } else { "Attacker" });
        }

        self.add_step(step);
    }

    /// Log Shield check
    pub fn log_shield_check(&mut self, who: &str, has_shield: bool) {
        if has_shield {
            self.add_step(CombatStep::new(
                CombatPhase::ShieldCheck,
                format!("{} has Shield - will absorb first hit", who),
            ));
        }
    }

    /// Log Shield absorption
    pub fn log_shield_absorption(&mut self, who: &str, damage_blocked: u8) {
        self.add_step(CombatStep::new(
            CombatPhase::ShieldAbsorption,
            format!("{}'s Shield absorbs {} damage", who, damage_blocked),
        ));
    }

    /// Log main damage calculation
    pub fn log_damage(&mut self, attacker_damage: u8, defender_damage: u8) {
        self.add_step(CombatStep::new(
            CombatPhase::DamageCalculation,
            format!(
                "Damage: Attacker deals {}, Defender deals {}",
                attacker_damage, defender_damage
            ),
        ).with_damage(attacker_damage));
    }

    /// Log counter-attack
    pub fn log_counter_attack(&mut self, damage: u8, attacker_died: bool) {
        let mut step = CombatStep::new(
            CombatPhase::CounterAttack,
            format!("Counter-attack deals {} damage", damage),
        ).with_damage(damage);

        if attacker_died {
            step = step.with_death("Attacker");
        }

        self.add_step(step);
    }

    /// Log Lethal trigger
    pub fn log_lethal(&mut self, who: &str, damage_was: u8) {
        self.add_step(CombatStep::new(
            CombatPhase::LethalTrigger,
            format!("{} Lethal triggers with {} damage - instant kill", who, damage_was),
        ).with_death(if who == "Attacker" { "Defender" } else { "Attacker" }));
    }

    /// Log Piercing overflow
    pub fn log_piercing(&mut self, overflow_damage: u8) {
        self.add_step(CombatStep::new(
            CombatPhase::PiercingOverflow,
            format!("Piercing deals {} overflow damage to face", overflow_damage),
        ).with_damage(overflow_damage));
    }

    /// Log Lifesteal healing
    pub fn log_lifesteal(&mut self, healed: u8) {
        self.add_step(CombatStep::new(
            CombatPhase::LifestealHeal,
            format!("Lifesteal heals attacker for {}", healed),
        ).with_healing(healed));
        self.attacker_healed = healed;
    }

    /// Log Ranged (no counter)
    pub fn log_ranged(&mut self) {
        self.add_step(CombatStep::new(
            CombatPhase::RangedAttack,
            "Ranged attack - no counter-attack".to_string(),
        ));
    }

    /// Log face damage
    pub fn log_face_damage(&mut self, damage: u8) {
        self.add_step(CombatStep::new(
            CombatPhase::FaceAttack,
            format!("Face attack deals {} damage to enemy player", damage),
        ).with_damage(damage));
        self.face_damage = damage;
    }

    /// Log completion
    pub fn log_complete(&mut self, attacker_died: bool, defender_died: bool) {
        self.attacker_died = attacker_died;
        self.defender_died = defender_died;

        let result = match (attacker_died, defender_died) {
            (false, false) => "Both survive",
            (true, false) => "Attacker died",
            (false, true) => "Defender died",
            (true, true) => "Both died",
        };

        self.add_step(CombatStep::new(
            CombatPhase::Complete,
            format!("Combat complete: {}", result),
        ));
    }

    /// Format the trace as a string for logging
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        output.push_str("║                     COMBAT TRACE                              ║\n");
        output.push_str("╠══════════════════════════════════════════════════════════════╣\n");

        for (i, step) in self.steps.iter().enumerate() {
            output.push_str(&format!(
                "║ {:2}. [{:14}] {}\n",
                i + 1,
                step.phase.to_string(),
                step.description
            ));

            if step.damage_dealt > 0 {
                output.push_str(&format!("║     └─ Damage: {}\n", step.damage_dealt));
            }
            if step.healing > 0 {
                output.push_str(&format!("║     └─ Healing: {}\n", step.healing));
            }
            if let Some(ref who) = step.death {
                output.push_str(&format!("║     └─ Death: {}\n", who));
            }
        }

        output.push_str("╠══════════════════════════════════════════════════════════════╣\n");
        output.push_str(&format!(
            "║ Result: Attacker {} | Defender {} | Face: {} | Healed: {}\n",
            if self.attacker_died { "DEAD" } else { "alive" },
            if self.defender_died { "DEAD" } else { "alive" },
            self.face_damage,
            self.attacker_healed
        ));
        output.push_str("╚══════════════════════════════════════════════════════════════╝\n");

        output
    }
}

/// Combat tracer that collects all combat traces for a game
#[derive(Clone, Debug, Default)]
pub struct CombatTracer {
    /// All combat traces for the game
    pub traces: Vec<CombatTrace>,
    /// Whether tracing is enabled
    pub enabled: bool,
}

impl CombatTracer {
    /// Create a new combat tracer
    pub fn new(enabled: bool) -> Self {
        Self {
            traces: Vec::new(),
            enabled,
        }
    }

    /// Check if tracing is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Add a completed trace
    pub fn add_trace(&mut self, trace: CombatTrace) {
        if self.enabled {
            self.traces.push(trace);
        }
    }

    /// Get all traces
    pub fn traces(&self) -> &[CombatTrace] {
        &self.traces
    }

    /// Clear all traces
    pub fn clear(&mut self) {
        self.traces.clear();
    }

    /// Format all traces as a string
    pub fn format_all(&self) -> String {
        self.traces
            .iter()
            .map(|t| t.format())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// ============================================================================
// Effect Queue Tracing
// ============================================================================

/// Type of effect event being traced
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectEventType {
    /// Effect added to queue
    Queued,
    /// Effect execution started
    ExecutionStart,
    /// Effect execution completed
    ExecutionComplete,
    /// Death detected
    DeathDetected,
    /// Death trigger fired
    DeathTrigger,
    /// Queue processing started
    QueueStart,
    /// Queue processing completed
    QueueComplete,
}

impl std::fmt::Display for EffectEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectEventType::Queued => write!(f, "QUEUED"),
            EffectEventType::ExecutionStart => write!(f, "EXEC_START"),
            EffectEventType::ExecutionComplete => write!(f, "EXEC_DONE"),
            EffectEventType::DeathDetected => write!(f, "DEATH"),
            EffectEventType::DeathTrigger => write!(f, "DEATH_TRIGGER"),
            EffectEventType::QueueStart => write!(f, "QUEUE_START"),
            EffectEventType::QueueComplete => write!(f, "QUEUE_DONE"),
        }
    }
}

/// A single event in effect queue processing
#[derive(Clone, Debug)]
pub struct EffectEvent {
    /// Type of event
    pub event_type: EffectEventType,
    /// Description of the event
    pub description: String,
    /// Queue depth at this point
    pub queue_depth: usize,
    /// Cascade depth (how many death triggers deep)
    pub cascade_depth: usize,
    /// State changes (if any)
    pub state_changes: Vec<String>,
}

impl EffectEvent {
    /// Create a new effect event
    pub fn new(event_type: EffectEventType, description: impl Into<String>) -> Self {
        Self {
            event_type,
            description: description.into(),
            queue_depth: 0,
            cascade_depth: 0,
            state_changes: Vec::new(),
        }
    }

    /// Set queue depth
    pub fn with_queue_depth(mut self, depth: usize) -> Self {
        self.queue_depth = depth;
        self
    }

    /// Set cascade depth
    pub fn with_cascade_depth(mut self, depth: usize) -> Self {
        self.cascade_depth = depth;
        self
    }

    /// Add a state change
    pub fn with_state_change(mut self, change: impl Into<String>) -> Self {
        self.state_changes.push(change.into());
        self
    }
}

/// Format an effect for display
fn format_effect(effect: &Effect) -> String {
    match effect {
        Effect::Damage { target, amount, .. } => {
            format!("Damage({}, {})", format_target(target), amount)
        }
        Effect::Heal { target, amount, .. } => {
            format!("Heal({}, {})", format_target(target), amount)
        }
        Effect::BuffStats { target, attack, health, .. } => {
            format!("BuffStats({}, {:+}/{:+})", format_target(target), attack, health)
        }
        Effect::SetStats { target, attack, health } => {
            format!("SetStats({}, {}/{})", format_target(target), attack, health)
        }
        Effect::GrantKeyword { target, keyword, .. } => {
            format!("GrantKeyword({}, 0x{:02X})", format_target(target), keyword)
        }
        Effect::RemoveKeyword { target, keyword, .. } => {
            format!("RemoveKeyword({}, 0x{:02X})", format_target(target), keyword)
        }
        Effect::Destroy { target, .. } => {
            format!("Destroy({})", format_target(target))
        }
        Effect::Silence { target, .. } => {
            format!("Silence({})", format_target(target))
        }
        Effect::Draw { player, count } => {
            format!("Draw(P{}, {})", player.index() + 1, count)
        }
        Effect::GainEssence { player, amount } => {
            format!("GainEssence(P{}, {})", player.index() + 1, amount)
        }
        Effect::RefreshCreature { target } => {
            format!("Refresh({})", format_target(target))
        }
        Effect::Summon { owner, card_id, slot } => {
            let slot_str = slot.map_or("auto".to_string(), |s| format!("{}", s.0));
            format!("Summon(P{}, card={}, slot={})", owner.index() + 1, card_id.0, slot_str)
        }
        Effect::Bounce { target, .. } => {
            format!("Bounce({})", format_target(target))
        }
        Effect::SummonToken { owner, token, slot } => {
            let slot_str = slot.map_or("auto".to_string(), |s| format!("{}", s.0));
            format!("SummonToken(P{}, '{}' {}/{}, slot={})", owner.index() + 1, token.name, token.attack, token.health, slot_str)
        }
        Effect::Transform { target, into } => {
            format!("Transform({} -> '{}' {}/{})", format_target(target), into.name, into.attack, into.health)
        }
        Effect::Copy { target, owner } => {
            format!("Copy({} for P{})", format_target(target), owner.index() + 1)
        }
    }
}

/// Format an effect target for display
fn format_target(target: &EffectTarget) -> String {
    match target {
        EffectTarget::Creature { owner, slot } => {
            format!("P{}_Slot{}", owner.index() + 1, slot.0)
        }
        EffectTarget::Player(player) => {
            format!("P{}", player.index() + 1)
        }
        EffectTarget::AllAllyCreatures(player) => {
            format!("AllP{}Creatures", player.index() + 1)
        }
        EffectTarget::AllEnemyCreatures(player) => {
            format!("AllEnemyOf_P{}", player.index() + 1)
        }
        EffectTarget::AllCreatures => "AllCreatures".to_string(),
        EffectTarget::TriggerSource => "TriggerSource".to_string(),
        EffectTarget::None => "None".to_string(),
    }
}

/// Format an effect source for display
fn format_source(source: &EffectSource) -> String {
    match source {
        EffectSource::Card(card_id) => {
            format!("Card(id={})", card_id.0)
        }
        EffectSource::Creature { owner, slot } => {
            format!("Creature(P{}, Slot{})", owner.index() + 1, slot.0)
        }
        EffectSource::Support { owner, slot } => {
            format!("Support(P{}, Slot{})", owner.index() + 1, slot.0)
        }
        EffectSource::System => "System".to_string(),
    }
}

/// Traces effect queue processing for a game
#[derive(Clone, Debug, Default)]
pub struct EffectTracer {
    /// All effect events
    pub events: Vec<EffectEvent>,
    /// Whether tracing is enabled
    pub enabled: bool,
    /// Current cascade depth
    cascade_depth: usize,
}

impl EffectTracer {
    /// Create a new effect tracer
    pub fn new(enabled: bool) -> Self {
        Self {
            events: Vec::new(),
            enabled,
            cascade_depth: 0,
        }
    }

    /// Check if tracing is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Log queue processing start
    pub fn log_queue_start(&mut self, queue_size: usize) {
        if !self.enabled {
            return;
        }
        self.events.push(
            EffectEvent::new(
                EffectEventType::QueueStart,
                format!("Effect queue processing started ({} effects)", queue_size),
            )
            .with_queue_depth(queue_size),
        );
    }

    /// Log queue processing complete
    pub fn log_queue_complete(&mut self, effects_processed: usize) {
        if !self.enabled {
            return;
        }
        self.events.push(EffectEvent::new(
            EffectEventType::QueueComplete,
            format!(
                "Effect queue processing complete ({} effects processed)",
                effects_processed
            ),
        ));
    }

    /// Log effect being queued
    pub fn log_effect_queued(&mut self, effect: &Effect, source: &EffectSource, queue_size: usize) {
        if !self.enabled {
            return;
        }
        self.events.push(
            EffectEvent::new(
                EffectEventType::Queued,
                format!(
                    "{} queued from {}",
                    format_effect(effect),
                    format_source(source)
                ),
            )
            .with_queue_depth(queue_size)
            .with_cascade_depth(self.cascade_depth),
        );
    }

    /// Log effect execution starting
    pub fn log_effect_start(&mut self, effect: &Effect, queue_remaining: usize) {
        if !self.enabled {
            return;
        }
        self.events.push(
            EffectEvent::new(
                EffectEventType::ExecutionStart,
                format!("Executing: {}", format_effect(effect)),
            )
            .with_queue_depth(queue_remaining)
            .with_cascade_depth(self.cascade_depth),
        );
    }

    /// Log effect execution complete with state changes
    pub fn log_effect_complete(&mut self, effect: &Effect, changes: Vec<String>) {
        if !self.enabled {
            return;
        }
        let mut event = EffectEvent::new(
            EffectEventType::ExecutionComplete,
            format!("Completed: {}", format_effect(effect)),
        )
        .with_cascade_depth(self.cascade_depth);

        for change in changes {
            event = event.with_state_change(change);
        }

        self.events.push(event);
    }

    /// Log death detection
    pub fn log_death(&mut self, owner: PlayerId, slot: u8, creature_name: &str) {
        if !self.enabled {
            return;
        }
        self.events.push(
            EffectEvent::new(
                EffectEventType::DeathDetected,
                format!(
                    "Death: {} (P{} Slot {})",
                    creature_name,
                    owner.index() + 1,
                    slot
                ),
            )
            .with_cascade_depth(self.cascade_depth),
        );
    }

    /// Log death trigger firing
    pub fn log_death_trigger(&mut self, trigger_type: &str, source_owner: PlayerId, source_slot: u8) {
        if !self.enabled {
            return;
        }
        self.cascade_depth += 1;
        self.events.push(
            EffectEvent::new(
                EffectEventType::DeathTrigger,
                format!(
                    "{} triggered from P{} Slot {}",
                    trigger_type,
                    source_owner.index() + 1,
                    source_slot
                ),
            )
            .with_cascade_depth(self.cascade_depth),
        );
    }

    /// End death trigger cascade level
    pub fn end_death_trigger(&mut self) {
        if self.cascade_depth > 0 {
            self.cascade_depth -= 1;
        }
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.cascade_depth = 0;
    }

    /// Format all events as a string
    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str("╔══════════════════════════════════════════════════════════════════╗\n");
        output.push_str("║                      EFFECT QUEUE TRACE                          ║\n");
        output.push_str("╠══════════════════════════════════════════════════════════════════╣\n");

        for event in &self.events {
            let indent = "  ".repeat(event.cascade_depth);
            output.push_str(&format!(
                "║ {}[{:12}] {}\n",
                indent, event.event_type, event.description
            ));

            if event.queue_depth > 0 {
                output.push_str(&format!("║ {}    └─ Queue: {} remaining\n", indent, event.queue_depth));
            }

            for change in &event.state_changes {
                output.push_str(&format!("║ {}    └─ {}\n", indent, change));
            }
        }

        output.push_str("╚══════════════════════════════════════════════════════════════════╝\n");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_trace_creature_combat() {
        let mut trace = CombatTrace::new_creature_combat(
            PlayerId::PLAYER_ONE,
            Slot(2),
            3,
            4,
            Keywords::none().with_quick(),
            PlayerId::PLAYER_TWO,
            Slot(1),
            2,
            3,
            Keywords::none().with_shield(),
        );

        trace.log_quick_check(true, false);
        trace.log_quick_strike("Attacker", 3, false);
        trace.log_shield_check("Defender", true);
        trace.log_shield_absorption("Defender", 3);
        trace.log_complete(false, false);

        // 1 setup + 5 logged steps = 6 total
        assert_eq!(trace.steps.len(), 6);
        assert!(!trace.attacker_died);
        assert!(!trace.defender_died);

        let formatted = trace.format();
        assert!(formatted.contains("COMBAT TRACE"));
        assert!(formatted.contains("Quick"));
    }

    #[test]
    fn test_combat_trace_face_attack() {
        let mut trace = CombatTrace::new_face_attack(
            PlayerId::PLAYER_ONE,
            Slot(0),
            5,
            3,
            Keywords::none().with_lifesteal(),
            Slot(2),
        );

        trace.log_face_damage(5);
        trace.log_lifesteal(5);
        trace.log_complete(false, false);

        assert!(trace.is_face_attack);
        assert_eq!(trace.face_damage, 5);

        let formatted = trace.format();
        assert!(formatted.contains("Face Attack"));
    }

    #[test]
    fn test_combat_tracer() {
        let mut tracer = CombatTracer::new(true);

        let trace1 = CombatTrace::new_face_attack(
            PlayerId::PLAYER_ONE,
            Slot(0),
            3,
            3,
            Keywords::none(),
            Slot(0),
        );
        tracer.add_trace(trace1);

        let trace2 = CombatTrace::new_creature_combat(
            PlayerId::PLAYER_ONE,
            Slot(1),
            2,
            2,
            Keywords::none(),
            PlayerId::PLAYER_TWO,
            Slot(1),
            2,
            2,
            Keywords::none(),
        );
        tracer.add_trace(trace2);

        assert_eq!(tracer.traces().len(), 2);

        let formatted = tracer.format_all();
        assert!(formatted.contains("COMBAT TRACE"));
    }

    #[test]
    fn test_combat_tracer_disabled() {
        let mut tracer = CombatTracer::new(false);

        let trace = CombatTrace::new_face_attack(
            PlayerId::PLAYER_ONE,
            Slot(0),
            3,
            3,
            Keywords::none(),
            Slot(0),
        );
        tracer.add_trace(trace);

        // Should be empty when disabled
        assert!(tracer.traces().is_empty());
    }

    #[test]
    fn test_effect_tracer_basic() {
        let mut tracer = EffectTracer::new(true);

        tracer.log_queue_start(3);

        let effect = Effect::Damage {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(0),
            },
            amount: 5,
            filter: None,
        };
        tracer.log_effect_queued(&effect, &EffectSource::System, 3);
        tracer.log_effect_start(&effect, 2);
        tracer.log_effect_complete(&effect, vec!["P1 Slot 0 health: 5 -> 0".to_string()]);

        tracer.log_death(PlayerId::PLAYER_ONE, 0, "Test Creature");
        tracer.log_queue_complete(1);

        assert_eq!(tracer.events.len(), 6);

        let formatted = tracer.format();
        assert!(formatted.contains("EFFECT QUEUE TRACE"));
        assert!(formatted.contains("Damage"));
        assert!(formatted.contains("DEATH"));
    }

    #[test]
    fn test_effect_tracer_cascade() {
        let mut tracer = EffectTracer::new(true);

        tracer.log_queue_start(1);
        tracer.log_death(PlayerId::PLAYER_ONE, 0, "Creature A");
        tracer.log_death_trigger("OnDeath", PlayerId::PLAYER_ONE, 0);

        // Cascade depth should be 1
        assert_eq!(tracer.cascade_depth, 1);

        tracer.log_death(PlayerId::PLAYER_TWO, 0, "Creature B");
        tracer.log_death_trigger("OnDeath", PlayerId::PLAYER_TWO, 0);

        // Cascade depth should be 2
        assert_eq!(tracer.cascade_depth, 2);

        tracer.end_death_trigger();
        tracer.end_death_trigger();

        assert_eq!(tracer.cascade_depth, 0);
    }

    #[test]
    fn test_effect_tracer_disabled() {
        let mut tracer = EffectTracer::new(false);

        tracer.log_queue_start(3);
        tracer.log_death(PlayerId::PLAYER_ONE, 0, "Test");

        // Should be empty when disabled
        assert!(tracer.events.is_empty());
    }

    #[test]
    fn test_format_effect() {
        let damage = Effect::Damage {
            target: EffectTarget::Player(PlayerId::PLAYER_TWO),
            amount: 10,
            filter: None,
        };
        assert_eq!(format_effect(&damage), "Damage(P2, 10)");

        let heal = Effect::Heal {
            target: EffectTarget::Creature {
                owner: PlayerId::PLAYER_ONE,
                slot: Slot(2),
            },
            amount: 3,
            filter: None,
        };
        assert_eq!(format_effect(&heal), "Heal(P1_Slot2, 3)");

        let draw = Effect::Draw {
            player: PlayerId::PLAYER_ONE,
            count: 2,
        };
        assert_eq!(format_effect(&draw), "Draw(P1, 2)");
    }
}
