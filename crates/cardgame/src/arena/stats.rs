//! Statistics collection for matches and matchups.

use std::time::Duration;

use crate::types::PlayerId;

/// Statistics for a single matchup (deck pair).
#[derive(Clone, Debug, Default)]
pub struct MatchupStats {
    /// Number of games played
    pub games: usize,
    /// Wins for player 1 (bot 1)
    pub bot1_wins: usize,
    /// Wins for player 2 (bot 2)
    pub bot2_wins: usize,
    /// Number of draws
    pub draws: usize,
    /// Total turns across all games
    pub total_turns: usize,
    /// Total time spent
    pub total_time: Duration,
}

impl MatchupStats {
    /// Create new empty stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a game result.
    pub fn record_game(&mut self, winner: Option<PlayerId>, turns: u32, time: Duration) {
        self.games += 1;
        self.total_turns += turns as usize;
        self.total_time += time;

        match winner {
            Some(PlayerId::PLAYER_ONE) => self.bot1_wins += 1,
            Some(PlayerId::PLAYER_TWO) => self.bot2_wins += 1,
            None => self.draws += 1,
            _ => {}
        }
    }

    /// Win rate for bot 1 (0.0 to 1.0).
    pub fn bot1_win_rate(&self) -> f64 {
        if self.games == 0 {
            0.5
        } else {
            self.bot1_wins as f64 / self.games as f64
        }
    }

    /// Win rate for bot 2 (0.0 to 1.0).
    pub fn bot2_win_rate(&self) -> f64 {
        if self.games == 0 {
            0.5
        } else {
            self.bot2_wins as f64 / self.games as f64
        }
    }

    /// Draw rate (0.0 to 1.0).
    pub fn draw_rate(&self) -> f64 {
        if self.games == 0 {
            0.0
        } else {
            self.draws as f64 / self.games as f64
        }
    }

    /// Average turns per game.
    pub fn avg_turns(&self) -> f64 {
        if self.games == 0 {
            0.0
        } else {
            self.total_turns as f64 / self.games as f64
        }
    }

    /// Average time per game in milliseconds.
    pub fn avg_time_ms(&self) -> f64 {
        if self.games == 0 {
            0.0
        } else {
            self.total_time.as_millis() as f64 / self.games as f64
        }
    }

    /// Merge another stats object into this one.
    pub fn merge(&mut self, other: &MatchupStats) {
        self.games += other.games;
        self.bot1_wins += other.bot1_wins;
        self.bot2_wins += other.bot2_wins;
        self.draws += other.draws;
        self.total_turns += other.total_turns;
        self.total_time += other.total_time;
    }
}

/// Statistics for a match (possibly multiple matchups).
#[derive(Clone, Debug, Default)]
pub struct MatchStats {
    /// Overall statistics (aggregated across all matchups)
    pub overall: MatchupStats,
    /// Bot 1 name
    pub bot1_name: String,
    /// Bot 2 name
    pub bot2_name: String,
    /// Wall-clock time for the entire match (set separately for parallel execution)
    pub wall_clock_time: Option<Duration>,
}

impl MatchStats {
    /// Create new match stats.
    pub fn new(bot1_name: String, bot2_name: String) -> Self {
        Self {
            overall: MatchupStats::new(),
            bot1_name,
            bot2_name,
            wall_clock_time: None,
        }
    }

    /// Set the wall-clock time for the match (for parallel execution).
    pub fn set_wall_clock_time(&mut self, time: Duration) {
        self.wall_clock_time = Some(time);
    }

    /// Record a game result.
    pub fn record_game(&mut self, winner: Option<PlayerId>, turns: u32, time: Duration) {
        self.overall.record_game(winner, turns, time);
    }

    /// Print a summary of the match.
    pub fn summary(&self) -> String {
        let time_info = match self.wall_clock_time {
            Some(wall_clock) => {
                let speedup = self.overall.total_time.as_secs_f64() / wall_clock.as_secs_f64().max(0.001);
                format!(
                    "Wall-clock time: {:.2}s ({:.1}x speedup)\n\
                     CPU time: {:.2}s",
                    wall_clock.as_secs_f64(),
                    speedup,
                    self.overall.total_time.as_secs_f64()
                )
            }
            None => format!("Total time: {:.2}s", self.overall.total_time.as_secs_f64()),
        };

        format!(
            "Match: {} vs {}\n\
             Games: {}\n\
             Bot 1 ({}) wins: {} ({:.1}%)\n\
             Bot 2 ({}) wins: {} ({:.1}%)\n\
             Draws: {} ({:.1}%)\n\
             Avg turns: {:.1}\n\
             Avg time: {:.2}ms/game\n\
             {}",
            self.bot1_name,
            self.bot2_name,
            self.overall.games,
            self.bot1_name,
            self.overall.bot1_wins,
            self.overall.bot1_win_rate() * 100.0,
            self.bot2_name,
            self.overall.bot2_wins,
            self.overall.bot2_win_rate() * 100.0,
            self.overall.draws,
            self.overall.draw_rate() * 100.0,
            self.overall.avg_turns(),
            self.overall.avg_time_ms(),
            time_info,
        )
    }
}
