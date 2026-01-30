//! Effect queue tracing for debugging effect resolution.

use crate::core::effects::{Effect, EffectSource, EffectTarget};
use crate::core::types::PlayerId;

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
        Effect::DestroySelf { owner, slot } => {
            format!("DestroySelf(P{}_Slot{})", owner.index() + 1, slot.0)
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
        EffectSource::Commander { owner } => {
            format!("Commander(P{})", owner.index() + 1)
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

