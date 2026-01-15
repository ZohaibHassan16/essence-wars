//! Export functionality for diagnostic data.
//!
//! Provides CSV and JSON export for diagnostic results, enabling
//! integration with Python analysis tools and external processing.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use serde::Serialize;

use super::analyzer::AggregatedStats;
use super::collector::GameDiagnostics;

/// Export format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// CSV format (creates multiple files).
    Csv,
    /// JSON format (single file with nested structure).
    Json,
    /// Both CSV and JSON.
    All,
}

impl ExportFormat {
    /// Parse from string argument.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Some(Self::Csv),
            "json" => Some(Self::Json),
            "all" => Some(Self::All),
            _ => None,
        }
    }
}

/// Per-game metadata for export.
#[derive(Debug, Clone, Serialize)]
pub struct GameMetadataExport {
    /// Game index.
    pub game_id: usize,
    /// Random seed.
    pub seed: u64,
    /// Winner (1 = P1, 2 = P2, 0 = draw).
    pub winner: u8,
    /// Total turns.
    pub total_turns: u32,
    /// Turn when P1 first took damage.
    pub first_damage_to_p1: Option<u32>,
    /// Turn when P2 first took damage.
    pub first_damage_to_p2: Option<u32>,
    /// Turn when first creature died.
    pub first_creature_death: Option<u32>,
    /// P1 actions.
    pub p1_actions: usize,
    /// P2 actions.
    pub p2_actions: usize,
    /// Average board advantage (positive = P1 ahead).
    pub avg_board_advantage: f64,
    /// Turns P1 was ahead.
    pub turns_p1_ahead: u32,
    /// Turns P2 was ahead.
    pub turns_p2_ahead: u32,
    /// P1 essence spent.
    pub p1_essence_spent: u32,
    /// P2 essence spent.
    pub p2_essence_spent: u32,
    /// P1 face damage dealt.
    pub p1_face_damage: u32,
    /// P2 face damage dealt.
    pub p2_face_damage: u32,
    /// P1 creatures killed.
    pub p1_creatures_killed: u32,
    /// P2 creatures killed.
    pub p2_creatures_killed: u32,
}

impl GameMetadataExport {
    /// Create from GameDiagnostics.
    pub fn from_diagnostics(idx: usize, diag: &GameDiagnostics) -> Self {
        let winner = match diag.winner {
            Some(crate::types::PlayerId::PLAYER_ONE) => 1,
            Some(crate::types::PlayerId::PLAYER_TWO) => 2,
            _ => 0,
        };

        Self {
            game_id: idx,
            seed: diag.seed,
            winner,
            total_turns: diag.total_turns,
            first_damage_to_p1: diag.first_damage_to_p1_turn,
            first_damage_to_p2: diag.first_damage_to_p2_turn,
            first_creature_death: diag.first_creature_death_turn,
            p1_actions: diag.p1_actions,
            p2_actions: diag.p2_actions,
            avg_board_advantage: diag.metrics.avg_board_advantage,
            turns_p1_ahead: diag.metrics.turns_p1_ahead,
            turns_p2_ahead: diag.metrics.turns_p2_ahead,
            p1_essence_spent: diag.metrics.resource_efficiency.p1_essence_spent,
            p2_essence_spent: diag.metrics.resource_efficiency.p2_essence_spent,
            p1_face_damage: diag.metrics.combat_efficiency.p1_face_damage,
            p2_face_damage: diag.metrics.combat_efficiency.p2_face_damage,
            p1_creatures_killed: diag.metrics.combat_efficiency.p1_creatures_killed,
            p2_creatures_killed: diag.metrics.combat_efficiency.p2_creatures_killed,
        }
    }
}

/// Per-turn data for export.
#[derive(Debug, Clone, Serialize)]
pub struct TurnDataExport {
    /// Game index.
    pub game_id: usize,
    /// Turn number.
    pub turn: u32,
    /// P1 life.
    pub p1_life: i32,
    /// P2 life.
    pub p2_life: i32,
    /// P1 creature count.
    pub p1_creatures: usize,
    /// P2 creature count.
    pub p2_creatures: usize,
    /// P1 total attack.
    pub p1_attack: i32,
    /// P2 total attack.
    pub p2_attack: i32,
    /// P1 total health.
    pub p1_health: i32,
    /// P2 total health.
    pub p2_health: i32,
    /// P1 hand size.
    pub p1_hand: usize,
    /// P2 hand size.
    pub p2_hand: usize,
    /// P1 current essence.
    pub p1_essence: u8,
    /// P2 current essence.
    pub p2_essence: u8,
    /// Board advantage score.
    pub board_advantage: f64,
}

/// Aggregate statistics for export.
#[derive(Debug, Clone, Serialize)]
pub struct AggregateStatsExport {
    /// Total games.
    pub total_games: usize,
    /// P1 wins.
    pub p1_wins: usize,
    /// P2 wins.
    pub p2_wins: usize,
    /// Draws.
    pub draws: usize,
    /// P1 win rate.
    pub p1_win_rate: f64,
    /// P1 win rate 95% CI lower.
    pub p1_win_rate_ci_lower: f64,
    /// P1 win rate 95% CI upper.
    pub p1_win_rate_ci_upper: f64,
    /// Chi-square statistic.
    pub chi_square: f64,
    /// P-value.
    pub p_value: f64,
    /// P1 first blood count.
    pub p1_first_blood: usize,
    /// P2 first blood count.
    pub p2_first_blood: usize,
    /// P1 first creature count.
    pub p1_first_creature: usize,
    /// P2 first creature count.
    pub p2_first_creature: usize,
    /// Average board advantage.
    pub avg_board_advantage: f64,
    /// Total turns P1 ahead.
    pub total_turns_p1_ahead: u32,
    /// Total turns P2 ahead.
    pub total_turns_p2_ahead: u32,
    /// P1 average essence spent.
    pub p1_avg_essence_spent: f64,
    /// P2 average essence spent.
    pub p2_avg_essence_spent: f64,
    /// P1 resource efficiency.
    pub p1_resource_efficiency: f64,
    /// P2 resource efficiency.
    pub p2_resource_efficiency: f64,
    /// P1 average face damage.
    pub p1_avg_face_damage: f64,
    /// P2 average face damage.
    pub p2_avg_face_damage: f64,
    /// P1 trade ratio.
    pub p1_trade_ratio: f64,
    /// P2 trade ratio.
    pub p2_trade_ratio: f64,
}

impl AggregateStatsExport {
    /// Create from AggregatedStats.
    pub fn from_stats(stats: &AggregatedStats) -> Self {
        let p1_stats = stats.p1_win_rate_stats();

        Self {
            total_games: stats.total_games,
            p1_wins: stats.p1_wins,
            p2_wins: stats.p2_wins,
            draws: stats.draws,
            p1_win_rate: p1_stats.proportion,
            p1_win_rate_ci_lower: p1_stats.ci_lower,
            p1_win_rate_ci_upper: p1_stats.ci_upper,
            chi_square: p1_stats.chi_square,
            p_value: p1_stats.p_value,
            p1_first_blood: stats.p1_first_blood,
            p2_first_blood: stats.p2_first_blood,
            p1_first_creature: stats.p1_first_creature,
            p2_first_creature: stats.p2_first_creature,
            avg_board_advantage: stats.avg_board_advantage(),
            total_turns_p1_ahead: stats.total_turns_p1_ahead,
            total_turns_p2_ahead: stats.total_turns_p2_ahead,
            p1_avg_essence_spent: stats.p1_avg_essence_spent(),
            p2_avg_essence_spent: stats.p2_avg_essence_spent(),
            p1_resource_efficiency: stats.p1_resource_efficiency(),
            p2_resource_efficiency: stats.p2_resource_efficiency(),
            p1_avg_face_damage: stats.p1_avg_face_damage(),
            p2_avg_face_damage: stats.p2_avg_face_damage(),
            p1_trade_ratio: stats.p1_trade_ratio(),
            p2_trade_ratio: stats.p2_trade_ratio(),
        }
    }
}

/// Full diagnostic export structure (for JSON).
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticExport {
    /// Aggregate statistics.
    pub aggregate: AggregateStatsExport,
    /// Per-game metadata.
    pub games: Vec<GameMetadataExport>,
    /// Per-turn data (optional, can be large).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turns: Option<Vec<TurnDataExport>>,
}

/// Export diagnostic results to CSV files.
///
/// Creates the following files in the output directory:
/// - `game_metadata.csv` - Per-game results
/// - `aggregate_stats.csv` - Summary statistics
/// - `per_turn_data.csv` - Per-turn metrics (optional)
pub fn export_csv(
    output_dir: &Path,
    stats: &AggregatedStats,
    games: &[GameDiagnostics],
    include_turns: bool,
) -> io::Result<()> {
    std::fs::create_dir_all(output_dir)?;

    // Export aggregate stats
    let agg_path = output_dir.join("aggregate_stats.csv");
    export_aggregate_csv(&agg_path, stats)?;

    // Export game metadata
    let games_path = output_dir.join("game_metadata.csv");
    export_games_csv(&games_path, games)?;

    // Export per-turn data (optional)
    if include_turns {
        let turns_path = output_dir.join("per_turn_data.csv");
        export_turns_csv(&turns_path, games)?;
    }

    Ok(())
}

/// Export aggregate statistics to CSV.
fn export_aggregate_csv(path: &Path, stats: &AggregatedStats) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let export = AggregateStatsExport::from_stats(stats);

    // Write header
    writeln!(
        writer,
        "metric,value"
    )?;

    // Write rows
    writeln!(writer, "total_games,{}", export.total_games)?;
    writeln!(writer, "p1_wins,{}", export.p1_wins)?;
    writeln!(writer, "p2_wins,{}", export.p2_wins)?;
    writeln!(writer, "draws,{}", export.draws)?;
    writeln!(writer, "p1_win_rate,{:.4}", export.p1_win_rate)?;
    writeln!(writer, "p1_win_rate_ci_lower,{:.4}", export.p1_win_rate_ci_lower)?;
    writeln!(writer, "p1_win_rate_ci_upper,{:.4}", export.p1_win_rate_ci_upper)?;
    writeln!(writer, "chi_square,{:.4}", export.chi_square)?;
    writeln!(writer, "p_value,{:.6}", export.p_value)?;
    writeln!(writer, "p1_first_blood,{}", export.p1_first_blood)?;
    writeln!(writer, "p2_first_blood,{}", export.p2_first_blood)?;
    writeln!(writer, "p1_first_creature,{}", export.p1_first_creature)?;
    writeln!(writer, "p2_first_creature,{}", export.p2_first_creature)?;
    writeln!(writer, "avg_board_advantage,{:.4}", export.avg_board_advantage)?;
    writeln!(writer, "total_turns_p1_ahead,{}", export.total_turns_p1_ahead)?;
    writeln!(writer, "total_turns_p2_ahead,{}", export.total_turns_p2_ahead)?;
    writeln!(writer, "p1_avg_essence_spent,{:.2}", export.p1_avg_essence_spent)?;
    writeln!(writer, "p2_avg_essence_spent,{:.2}", export.p2_avg_essence_spent)?;
    writeln!(writer, "p1_resource_efficiency,{:.4}", export.p1_resource_efficiency)?;
    writeln!(writer, "p2_resource_efficiency,{:.4}", export.p2_resource_efficiency)?;
    writeln!(writer, "p1_avg_face_damage,{:.2}", export.p1_avg_face_damage)?;
    writeln!(writer, "p2_avg_face_damage,{:.2}", export.p2_avg_face_damage)?;
    writeln!(writer, "p1_trade_ratio,{:.4}", export.p1_trade_ratio)?;
    writeln!(writer, "p2_trade_ratio,{:.4}", export.p2_trade_ratio)?;

    writer.flush()
}

/// Export game metadata to CSV.
fn export_games_csv(path: &Path, games: &[GameDiagnostics]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(
        writer,
        "game_id,seed,winner,total_turns,first_damage_p1,first_damage_p2,first_creature_death,p1_actions,p2_actions,avg_board_advantage,turns_p1_ahead,turns_p2_ahead,p1_essence_spent,p2_essence_spent,p1_face_damage,p2_face_damage,p1_creatures_killed,p2_creatures_killed"
    )?;

    // Write rows
    for (idx, game) in games.iter().enumerate() {
        let export = GameMetadataExport::from_diagnostics(idx, game);
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{},{:.2},{},{},{},{},{},{},{},{}",
            export.game_id,
            export.seed,
            export.winner,
            export.total_turns,
            export.first_damage_to_p1.map_or(String::new(), |v| v.to_string()),
            export.first_damage_to_p2.map_or(String::new(), |v| v.to_string()),
            export.first_creature_death.map_or(String::new(), |v| v.to_string()),
            export.p1_actions,
            export.p2_actions,
            export.avg_board_advantage,
            export.turns_p1_ahead,
            export.turns_p2_ahead,
            export.p1_essence_spent,
            export.p2_essence_spent,
            export.p1_face_damage,
            export.p2_face_damage,
            export.p1_creatures_killed,
            export.p2_creatures_killed,
        )?;
    }

    writer.flush()
}

/// Export per-turn data to CSV.
fn export_turns_csv(path: &Path, games: &[GameDiagnostics]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(
        writer,
        "game_id,turn,p1_life,p2_life,p1_creatures,p2_creatures,p1_attack,p2_attack,p1_health,p2_health,p1_hand,p2_hand,p1_essence,p2_essence,board_advantage"
    )?;

    // Write rows
    for (game_idx, game) in games.iter().enumerate() {
        for snapshot in &game.snapshots {
            let board_advantage = (snapshot.p1_life - snapshot.p2_life) as f64 / 3.0
                + (snapshot.p1_total_attack - snapshot.p2_total_attack) as f64 / 3.0
                + (snapshot.p1_total_health - snapshot.p2_total_health) as f64 / 3.0;

            writeln!(
                writer,
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.2}",
                game_idx,
                snapshot.turn,
                snapshot.p1_life,
                snapshot.p2_life,
                snapshot.p1_creatures,
                snapshot.p2_creatures,
                snapshot.p1_total_attack,
                snapshot.p2_total_attack,
                snapshot.p1_total_health,
                snapshot.p2_total_health,
                snapshot.p1_hand_size,
                snapshot.p2_hand_size,
                snapshot.p1_essence,
                snapshot.p2_essence,
                board_advantage,
            )?;
        }
    }

    writer.flush()
}

/// Export diagnostic results to JSON.
///
/// Creates a single JSON file with nested structure.
pub fn export_json(
    path: &Path,
    stats: &AggregatedStats,
    games: &[GameDiagnostics],
    include_turns: bool,
) -> io::Result<()> {
    let aggregate = AggregateStatsExport::from_stats(stats);

    let game_exports: Vec<GameMetadataExport> = games
        .iter()
        .enumerate()
        .map(|(idx, g)| GameMetadataExport::from_diagnostics(idx, g))
        .collect();

    let turns = if include_turns {
        let mut all_turns = Vec::new();
        for (game_idx, game) in games.iter().enumerate() {
            for snapshot in &game.snapshots {
                let board_advantage = (snapshot.p1_life - snapshot.p2_life) as f64 / 3.0
                    + (snapshot.p1_total_attack - snapshot.p2_total_attack) as f64 / 3.0
                    + (snapshot.p1_total_health - snapshot.p2_total_health) as f64 / 3.0;

                all_turns.push(TurnDataExport {
                    game_id: game_idx,
                    turn: snapshot.turn,
                    p1_life: snapshot.p1_life,
                    p2_life: snapshot.p2_life,
                    p1_creatures: snapshot.p1_creatures,
                    p2_creatures: snapshot.p2_creatures,
                    p1_attack: snapshot.p1_total_attack,
                    p2_attack: snapshot.p2_total_attack,
                    p1_health: snapshot.p1_total_health,
                    p2_health: snapshot.p2_total_health,
                    p1_hand: snapshot.p1_hand_size,
                    p2_hand: snapshot.p2_hand_size,
                    p1_essence: snapshot.p1_essence,
                    p2_essence: snapshot.p2_essence,
                    board_advantage,
                });
            }
        }
        Some(all_turns)
    } else {
        None
    };

    let export = DiagnosticExport {
        aggregate,
        games: game_exports,
        turns,
    };

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &export).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_parse() {
        assert_eq!(ExportFormat::parse("csv"), Some(ExportFormat::Csv));
        assert_eq!(ExportFormat::parse("JSON"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::parse("all"), Some(ExportFormat::All));
        assert_eq!(ExportFormat::parse("invalid"), None);
    }

    #[test]
    fn test_aggregate_stats_export() {
        let stats = AggregatedStats::default();
        let export = AggregateStatsExport::from_stats(&stats);

        assert_eq!(export.total_games, 0);
        assert_eq!(export.p1_wins, 0);
    }
}
