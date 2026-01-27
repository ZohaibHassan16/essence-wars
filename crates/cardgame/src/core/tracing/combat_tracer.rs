//! Combat tracing for step-by-step keyword resolution debugging.

use crate::core::keywords::Keywords;
use crate::core::types::{PlayerId, Slot};

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
