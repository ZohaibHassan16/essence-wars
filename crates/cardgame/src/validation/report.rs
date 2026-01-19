//! Validation report formatting and output.
//!
//! Provides utilities for displaying validation results.

use std::path::{Path, PathBuf};
use std::time::Duration;

use super::types::ValidationResults;

/// Print formatted results to stdout.
pub fn print_results(results: &ValidationResults, total_time: Duration) {
    println!("=== Matchup Results ===");

    for m in &results.matchups {
        let f1_name = capitalize(&m.faction1);
        let f2_name = capitalize(&m.faction2);

        println!("\n{} vs {}:", f1_name, f2_name);
        println!(
            "  {} P1: {}/{} ({:.1}%)  |  {} P2: {}/{} ({:.1}%)",
            f1_name,
            m.f1_as_p1_wins,
            m.f1_as_p1_games,
            m.f1_as_p1_wins as f64 / m.f1_as_p1_games as f64 * 100.0,
            f1_name,
            m.f1_as_p2_wins,
            m.f1_as_p2_games,
            m.f1_as_p2_wins as f64 / m.f1_as_p2_games as f64 * 100.0,
        );
        println!(
            "  Total: {} {:.1}% / {} {:.1}%  (draws: {})",
            f1_name,
            m.faction1_win_rate * 100.0,
            f2_name,
            m.faction2_win_rate * 100.0,
            m.draws,
        );
        println!(
            "  Avg turns: {:.1}, Time: {:.1}s",
            m.avg_turns, m.total_time_secs
        );
    }

    println!("\n=== Summary ===");
    println!(
        "P1 Win Rate: {:.1}% [{}]",
        results.summary.p1_win_rate * 100.0,
        results.summary.p1_status
    );

    println!("\nFaction Win Rates:");
    for (faction, rate) in &results.summary.faction_win_rates {
        println!("  {}: {:.1}%", capitalize(faction), rate * 100.0);
    }
    println!(
        "Max Delta: {:.1}% [{}]",
        results.summary.max_faction_delta * 100.0,
        results.summary.faction_status
    );

    if !results.summary.warnings.is_empty() {
        println!("\nWarnings:");
        for w in &results.summary.warnings {
            println!("  - {}", w);
        }
    }

    println!("\nOverall Status: {}", results.summary.overall_status);
    println!("Total Time: {:.1}s", total_time.as_secs_f64());
}

/// Export results to JSON file.
pub fn export_json(results: &ValidationResults, path: &Path) -> Result<(), ExportError> {
    let json = serde_json::to_string_pretty(results).map_err(ExportError::Serialization)?;
    std::fs::write(path, json).map_err(ExportError::Io)?;
    println!("\n💾 Results saved to: {:?}", path);
    Ok(())
}

/// Save comprehensive validation results to a timestamped directory.
///
/// Creates a directory structure like `experiments/validation/YYYY-MM-DD_HHMM/`
/// containing:
/// - `results.json` - Full JSON data
/// - `summary.txt` - Human-readable summary
/// - `config.toml` - Run configuration
///
/// Returns the path to the created directory.
pub fn save_validation_results(
    results: &ValidationResults,
    total_time: Duration,
    run_id: Option<&str>,
) -> Result<PathBuf, ExportError> {
    // Generate run_id if not provided
    let run_id = run_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d_%H%M").to_string());

    // Create validation directory
    let validation_dir = PathBuf::from("experiments")
        .join("validation")
        .join(&run_id);
    std::fs::create_dir_all(&validation_dir).map_err(ExportError::Io)?;

    // Save JSON results
    let results_file = validation_dir.join("results.json");
    let json = serde_json::to_string_pretty(results).map_err(ExportError::Serialization)?;
    std::fs::write(&results_file, json).map_err(ExportError::Io)?;

    // Save human-readable summary
    let summary_file = validation_dir.join("summary.txt");
    let summary_content = generate_summary_text(results, total_time, &run_id);
    std::fs::write(&summary_file, summary_content).map_err(ExportError::Io)?;

    // Save config TOML
    let config_file = validation_dir.join("config.toml");
    let config_content = generate_config_toml(results, &run_id);
    std::fs::write(&config_file, config_content).map_err(ExportError::Io)?;

    println!("\n💾 Validation results saved to: {}", validation_dir.display());
    println!("   📄 results.json - Full JSON data");
    println!("   📝 summary.txt  - Human-readable summary");
    println!("   ⚙️  config.toml  - Run configuration");

    Ok(validation_dir)
}

/// Generate human-readable summary text.
fn generate_summary_text(
    results: &ValidationResults,
    total_time: Duration,
    run_id: &str,
) -> String {
    let mut lines = Vec::new();

    lines.push("=".repeat(60));
    lines.push(format!("VALIDATION SUMMARY - {}", run_id));
    lines.push("=".repeat(60));
    lines.push(String::new());

    lines.push(format!("Timestamp: {}", results.timestamp));
    lines.push("Parallel Execution: false".to_string());
    lines.push(String::new());

    lines.push("--- Balance Status ---".to_string());
    lines.push(format!(
        "Overall Status: {}",
        results.summary.overall_status
    ));
    lines.push(format!(
        "P1 Win Rate: {:.1}%",
        results.summary.p1_win_rate * 100.0
    ));
    lines.push(format!(
        "Max Faction Delta: {:.1}%",
        results.summary.max_faction_delta * 100.0
    ));
    lines.push(String::new());

    lines.push("--- Faction Win Rates ---".to_string());
    for (faction, rate) in &results.summary.faction_win_rates {
        lines.push(format!("  {}: {:.1}%", capitalize(faction), rate * 100.0));
    }

    if !results.summary.warnings.is_empty() {
        lines.push(String::new());
        lines.push("--- Warnings ---".to_string());
        for w in &results.summary.warnings {
            lines.push(format!("  - {}", w));
        }
    }

    lines.push(String::new());
    lines.push("--- Timing ---".to_string());
    lines.push(format!("Wall Time: {:.1}s", total_time.as_secs_f64()));

    lines.push(String::new());
    lines.push("--- Matchup Details ---".to_string());
    for m in &results.matchups {
        let f1 = capitalize(&m.faction1);
        let f2 = capitalize(&m.faction2);
        let f1_wr = m.faction1_win_rate * 100.0;
        let f2_wr = m.faction2_win_rate * 100.0;
        lines.push(format!("{} vs {}: {:.1}% / {:.1}%", f1, f2, f1_wr, f2_wr));
    }

    lines.push(String::new());
    lines.push("=".repeat(60));

    lines.join("\n")
}

/// Generate configuration TOML content.
fn generate_config_toml(results: &ValidationResults, run_id: &str) -> String {
    let mut lines = Vec::new();

    lines.push("# Validation Run Configuration".to_string());
    lines.push(format!("run_id = \"{}\"", run_id));
    lines.push(format!("timestamp = \"{}\"", results.timestamp));
    lines.push("parallel_execution = false".to_string());
    lines.push(String::new());

    lines.push("[config]".to_string());
    lines.push(format!(
        "games_per_matchup = {}",
        results.config.games_per_matchup
    ));
    lines.push(format!(
        "mcts_simulations = {}",
        results.config.mcts_simulations
    ));
    lines.push(format!("seed = {}", results.config.seed));
    lines.push(format!("threads = {}", results.config.threads));
    if let Some(ref filter) = results.config.matchup_filter {
        lines.push(format!("matchup_filter = \"{}\"", filter));
    }
    lines.push(String::new());

    lines.push("[summary]".to_string());
    lines.push(format!(
        "overall_status = \"{}\"",
        match results.summary.overall_status {
            super::types::BalanceStatus::Balanced => "balanced",
            super::types::BalanceStatus::Warning => "warning",
            super::types::BalanceStatus::Imbalanced => "imbalanced",
        }
    ));
    lines.push(format!(
        "p1_win_rate = {:.4}",
        results.summary.p1_win_rate
    ));
    lines.push(format!(
        "max_faction_delta = {:.4}",
        results.summary.max_faction_delta
    ));

    lines.join("\n")
}

/// Errors that can occur during export.
#[derive(Debug)]
pub enum ExportError {
    /// JSON serialization failed.
    Serialization(serde_json::Error),
    /// File I/O failed.
    Io(std::io::Error),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Serialization(e) => write!(f, "JSON serialization error: {}", e),
            ExportError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExportError::Serialization(e) => Some(e),
            ExportError::Io(e) => Some(e),
        }
    }
}

/// Capitalize first letter of a string.
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("argentum"), "Argentum");
        assert_eq!(capitalize("SYMBIOTE"), "SYMBIOTE");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("a"), "A");
    }
}
