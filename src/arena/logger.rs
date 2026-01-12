//! Action logging for game traceability and debugging.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use crate::actions::Action;
use crate::state::GameState;
use crate::types::PlayerId;

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

/// Logger for recording game actions with optional verbosity.
pub struct ActionLogger {
    output: LogOutput,
    verbose: bool,
    game_start: Option<Instant>,
    action_count: usize,
}

impl ActionLogger {
    /// Create a new logger.
    pub fn new(output: LogOutput, verbose: bool) -> Self {
        Self {
            output,
            verbose,
            game_start: None,
            action_count: 0,
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
            "\n=== GAME END ===\nResult: {}\nFinal turn: {}\nFinal life: P1={}, P2={}\nActions: {}\nTime: {}ms\n",
            winner_str, final_turn, p1_life, p2_life, self.action_count, elapsed
        ))
    }

    /// Check if verbose mode is enabled.
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_memory_output() {
        let mut logger = ActionLogger::new(LogOutput::Memory(Vec::new()), false);

        logger.log_game_start(12345, "Bot1", "Bot2").unwrap();

        let record = ActionRecord {
            turn: 1,
            player: PlayerId::PLAYER_ONE,
            action: Action::EndTurn,
            thinking_time_us: 100,
            state_snapshot: None,
        };
        logger.log_action(&record).unwrap();
        logger.log_game_end(Some(PlayerId::PLAYER_ONE), 10, 15, 0).unwrap();

        if let LogOutput::Memory(lines) = &logger.output {
            assert_eq!(lines.len(), 3);
            assert!(lines[0].contains("12345"));
            assert!(lines[1].contains("EndTurn"));
            assert!(lines[2].contains("Player 1 wins"));
        }
    }

    #[test]
    fn test_state_snapshot() {
        let mut state = GameState::new();
        state.players[0].life = 25;
        state.players[1].life = 30;
        state.players[0].action_points = 3;

        let snapshot = StateSnapshot::from_state(&state);

        assert_eq!(snapshot.p1_life, 25);
        assert_eq!(snapshot.p2_life, 30);
        assert_eq!(snapshot.p1_ap, 3);
    }
}
