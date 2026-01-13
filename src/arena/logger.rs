//! Action logging for game traceability and debugging.
//!
//! This module provides comprehensive logging for:
//! - Game start/end events
//! - Individual actions with timing
//! - Combat traces with full creature state snapshots
//! - Effect queue processing
//!
//! Use verbose mode for full state snapshots and combat traces.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use crate::actions::Action;
use crate::combat::CombatResult;
use crate::keywords::Keywords;
use crate::state::{Creature, GameState};
use crate::types::{CardId, PlayerId, Slot};

/// Where to output log messages.
pub enum LogOutput {
    /// Write to stdout
    Stdout,
    /// Write to a file
    File(BufWriter<File>),
    /// Collect in memory (for testing)
    Memory(Vec<String>),
}

impl LogOutput {
    /// Create a file output.
    pub fn file(path: &Path) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self::File(BufWriter::new(file)))
    }

    fn write_line(&mut self, line: &str) -> io::Result<()> {
        match self {
            LogOutput::Stdout => {
                println!("{}", line);
                Ok(())
            }
            LogOutput::File(writer) => {
                writeln!(writer, "{}", line)?;
                writer.flush()
            }
            LogOutput::Memory(lines) => {
                lines.push(line.to_string());
                Ok(())
            }
        }
    }
}

/// Record of a single action taken during a game.
#[derive(Clone, Debug)]
pub struct ActionRecord {
    /// Turn number when the action was taken
    pub turn: u32,
    /// Which player took the action
    pub player: PlayerId,
    /// The action that was taken
    pub action: Action,
    /// Time spent selecting this action (microseconds)
    pub thinking_time_us: u64,
    /// Game state before the action (only in verbose mode)
    pub state_snapshot: Option<StateSnapshot>,
}

/// Snapshot of relevant game state for debugging.
#[derive(Clone, Debug)]
pub struct StateSnapshot {
    pub p1_life: i16,
    pub p2_life: i16,
    pub p1_ap: u8,
    pub p2_ap: u8,
    pub p1_creatures: usize,
    pub p2_creatures: usize,
    pub p1_hand_size: usize,
    pub p2_hand_size: usize,
}

impl StateSnapshot {
    pub fn from_state(state: &GameState) -> Self {
        Self {
            p1_life: state.players[0].life,
            p2_life: state.players[1].life,
            p1_ap: state.players[0].action_points,
            p2_ap: state.players[1].action_points,
            p1_creatures: state.players[0].creatures.len(),
            p2_creatures: state.players[1].creatures.len(),
            p1_hand_size: state.players[0].hand.len(),
            p2_hand_size: state.players[1].hand.len(),
        }
    }
}

/// Snapshot of a creature's state at a point in time.
///
/// Used for combat traces to show creature stats before and after combat.
#[derive(Clone, Debug)]
pub struct CreatureSnapshot {
    /// The slot this creature occupies
    pub slot: Slot,
    /// The card definition ID
    pub card_id: CardId,
    /// Owner of the creature
    pub owner: PlayerId,
    /// Current attack value (may be modified by buffs)
    pub attack: i8,
    /// Current health
    pub health: i8,
    /// Maximum health
    pub max_health: i8,
    /// Active keywords (as bitfield)
    pub keywords: Keywords,
    /// Whether the creature is exhausted
    pub exhausted: bool,
    /// Whether the creature is silenced
    pub silenced: bool,
}

impl CreatureSnapshot {
    /// Create a snapshot from a creature reference.
    pub fn from_creature(creature: &Creature) -> Self {
        Self {
            slot: creature.slot,
            card_id: creature.card_id,
            owner: creature.owner,
            attack: creature.attack,
            health: creature.current_health,
            max_health: creature.max_health,
            keywords: creature.keywords,
            exhausted: creature.status.is_exhausted(),
            silenced: creature.status.is_silenced(),
        }
    }

    /// Format keywords as a human-readable string.
    pub fn keywords_str(&self) -> String {
        let mut kw = Vec::new();
        if self.keywords.has_rush() { kw.push("Rush"); }
        if self.keywords.has_ranged() { kw.push("Ranged"); }
        if self.keywords.has_piercing() { kw.push("Piercing"); }
        if self.keywords.has_guard() { kw.push("Guard"); }
        if self.keywords.has_lifesteal() { kw.push("Lifesteal"); }
        if self.keywords.has_lethal() { kw.push("Lethal"); }
        if self.keywords.has_shield() { kw.push("Shield"); }
        if self.keywords.has_quick() { kw.push("Quick"); }
        if kw.is_empty() {
            "none".to_string()
        } else {
            kw.join(", ")
        }
    }
}

impl std::fmt::Display for CreatureSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let player = if self.owner == PlayerId::PLAYER_ONE { "P1" } else { "P2" };
        write!(
            f,
            "{}[slot={}, card={}, {}/{}, kw=[{}]{}{}]",
            player,
            self.slot.0,
            self.card_id.0,
            self.attack,
            self.health,
            self.keywords_str(),
            if self.exhausted { ", exhausted" } else { "" },
            if self.silenced { ", silenced" } else { "" },
        )
    }
}

/// Detailed trace of a combat event.
///
/// Captures the full state of combat including:
/// - Attacker and defender snapshots before combat
/// - Combat result (damage, deaths, healing)
/// - Any triggered effects
#[derive(Clone, Debug)]
pub struct CombatTrace {
    /// Turn number when combat occurred
    pub turn: u16,
    /// Snapshot of the attacking creature before combat
    pub attacker: CreatureSnapshot,
    /// Snapshot of the defending creature before combat (None for face attacks)
    pub defender: Option<CreatureSnapshot>,
    /// The target slot being attacked
    pub target_slot: Slot,
    /// Result of combat resolution
    pub result: CombatResult,
    /// Effects that were triggered during combat
    pub triggered_effects: Vec<String>,
}

impl CombatTrace {
    /// Create a new combat trace.
    pub fn new(
        turn: u16,
        attacker: CreatureSnapshot,
        defender: Option<CreatureSnapshot>,
        target_slot: Slot,
        result: CombatResult,
    ) -> Self {
        Self {
            turn,
            attacker,
            defender,
            target_slot,
            result,
            triggered_effects: Vec::new(),
        }
    }

    /// Add a triggered effect description.
    pub fn add_triggered_effect(&mut self, effect: String) {
        self.triggered_effects.push(effect);
    }

    /// Format the combat trace as a human-readable string.
    pub fn format(&self) -> String {
        let mut lines = Vec::new();

        // Header
        lines.push(format!("=== COMBAT (Turn {}) ===", self.turn));

        // Attacker info
        lines.push(format!("Attacker: {}", self.attacker));

        // Defender or face attack
        if let Some(ref defender) = self.defender {
            lines.push(format!("Defender: {}", defender));
        } else {
            lines.push(format!("Target: Face attack (slot {})", self.target_slot.0));
        }

        // Combat resolution
        lines.push("--- Resolution ---".to_string());
        lines.push(format!(
            "Attacker dealt {} damage{}",
            self.result.attacker_damage_dealt,
            if self.result.defender_died { " (LETHAL)" } else { "" }
        ));

        if self.defender.is_some() {
            lines.push(format!(
                "Defender dealt {} damage{}",
                self.result.defender_damage_dealt,
                if self.result.attacker_died { " (LETHAL)" } else { "" }
            ));
        }

        if self.result.face_damage > 0 {
            lines.push(format!("Face damage: {}", self.result.face_damage));
        }

        if self.result.attacker_healed > 0 {
            lines.push(format!("Lifesteal heal: {}", self.result.attacker_healed));
        }

        // Outcome summary
        let outcome = match (self.result.attacker_died, self.result.defender_died) {
            (true, true) => "TRADE (both died)",
            (true, false) => "Attacker DIED",
            (false, true) => "Defender DIED",
            (false, false) => "Both survived",
        };
        lines.push(format!("Outcome: {}", outcome));

        // Triggered effects
        if !self.triggered_effects.is_empty() {
            lines.push("--- Triggered Effects ---".to_string());
            for effect in &self.triggered_effects {
                lines.push(format!("  - {}", effect));
            }
        }

        lines.join("\n")
    }
}

/// Logger for recording game actions with optional verbosity.
pub struct ActionLogger {
    /// The output destination for log messages.
    pub output: LogOutput,
    verbose: bool,
    game_start: Option<Instant>,
    action_count: usize,
    combat_count: usize,
}

impl ActionLogger {
    /// Create a new logger.
    pub fn new(output: LogOutput, verbose: bool) -> Self {
        Self {
            output,
            verbose,
            game_start: None,
            action_count: 0,
            combat_count: 0,
        }
    }

    /// Create a logger that writes to stdout.
    pub fn stdout(verbose: bool) -> Self {
        Self::new(LogOutput::Stdout, verbose)
    }

    /// Create a logger that writes to a file.
    pub fn to_file(path: &Path, verbose: bool) -> io::Result<Self> {
        Ok(Self::new(LogOutput::file(path)?, verbose))
    }

    /// Log the start of a new game.
    pub fn log_game_start(
        &mut self,
        seed: u64,
        bot1_name: &str,
        bot2_name: &str,
    ) -> io::Result<()> {
        self.game_start = Some(Instant::now());
        self.action_count = 0;
        self.combat_count = 0;

        self.output.write_line(&format!(
            "=== GAME START ===\nSeed: {}\nPlayer 1: {}\nPlayer 2: {}\n",
            seed, bot1_name, bot2_name
        ))
    }

    /// Log an action taken during the game.
    pub fn log_action(&mut self, record: &ActionRecord) -> io::Result<()> {
        self.action_count += 1;

        let player_str = match record.player {
            PlayerId::PLAYER_ONE => "P1",
            PlayerId::PLAYER_TWO => "P2",
            _ => "??",
        };

        let mut line = format!(
            "[Turn {:2}] {} {:?} ({}us)",
            record.turn, player_str, record.action, record.thinking_time_us
        );

        if self.verbose {
            if let Some(ref snap) = record.state_snapshot {
                line.push_str(&format!(
                    "\n         State: P1[{} life, {} AP, {} creatures, {} cards] P2[{} life, {} AP, {} creatures, {} cards]",
                    snap.p1_life, snap.p1_ap, snap.p1_creatures, snap.p1_hand_size,
                    snap.p2_life, snap.p2_ap, snap.p2_creatures, snap.p2_hand_size,
                ));
            }
        }

        self.output.write_line(&line)
    }

    /// Log a combat event with full trace (only in verbose mode).
    ///
    /// This provides detailed combat logging including:
    /// - Attacker and defender creature snapshots
    /// - Combat result (damage, deaths, healing)
    /// - Triggered effects
    pub fn log_combat(&mut self, trace: &CombatTrace) -> io::Result<()> {
        self.combat_count += 1;

        if self.verbose {
            self.output.write_line(&trace.format())
        } else {
            // In non-verbose mode, just log a summary line
            let defender_str = trace
                .defender
                .as_ref()
                .map(|d| format!("creature at slot {}", d.slot.0))
                .unwrap_or_else(|| "face".to_string());

            let outcome = match (trace.result.attacker_died, trace.result.defender_died) {
                (true, true) => "TRADE",
                (true, false) => "LOST",
                (false, true) => "WIN",
                (false, false) => "survived",
            };

            self.output.write_line(&format!(
                "         Combat: slot {} -> {} ({} dmg, {})",
                trace.attacker.slot.0,
                defender_str,
                trace.result.attacker_damage_dealt,
                outcome
            ))
        }
    }

    /// Log an effect that was triggered.
    pub fn log_effect(&mut self, turn: u16, effect_description: &str) -> io::Result<()> {
        if self.verbose {
            self.output.write_line(&format!(
                "         [Turn {}] Effect: {}",
                turn, effect_description
            ))
        } else {
            Ok(())
        }
    }

    /// Log the end of a game.
    pub fn log_game_end(
        &mut self,
        winner: Option<PlayerId>,
        final_turn: u32,
        p1_life: i16,
        p2_life: i16,
    ) -> io::Result<()> {
        let elapsed = self.game_start.map(|s| s.elapsed().as_millis()).unwrap_or(0);

        let winner_str = match winner {
            Some(PlayerId::PLAYER_ONE) => "Player 1 wins!",
            Some(PlayerId::PLAYER_TWO) => "Player 2 wins!",
            None => "Draw!",
            _ => "Unknown",
        };

        self.output.write_line(&format!(
            "\n=== GAME END ===\nResult: {}\nFinal turn: {}\nFinal life: P1={}, P2={}\nActions: {}\nCombats: {}\nTime: {}ms\n",
            winner_str, final_turn, p1_life, p2_life, self.action_count, self.combat_count, elapsed
        ))
    }

    /// Check if verbose mode is enabled.
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Get the number of combats logged.
    pub fn combat_count(&self) -> usize {
        self.combat_count
    }
}
