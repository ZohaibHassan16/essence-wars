//! Validation report formatting and output.
//!
//! Provides utilities for displaying validation results.

use std::path::{Path, PathBuf};
use std::time::Duration;

use super::types::ValidationResults;

/// Print formatted results to stdout.
pub fn print_results(results: &ValidationResults, total_time: Duration) {
    // Print detailed deck matchups (commander vs commander)
    println!("=== Commander Matchup Results ===");
    println!(
        "{:28} vs {:28} | {:7} | {:7} | {:6}",
        "Commander 1", "Commander 2", "Overall", "As P1", "As P2"
    );
    println!("{}", "-".repeat(95));

    for m in &results.matchups {
        let p1_rate = if m.f1_as_p1_games > 0 {
            m.f1_as_p1_wins as f64 / m.f1_as_p1_games as f64 * 100.0
        } else {
            0.0
        };
        let p2_rate = if m.f1_as_p2_games > 0 {
            m.f1_as_p2_wins as f64 / m.f1_as_p2_games as f64 * 100.0
        } else {
            0.0
        };

        // Determine visual indicator
        let indicator = if m.faction1_win_rate >= 0.60 {
            "▲"
        } else if m.faction1_win_rate <= 0.40 {
            "▼"
        } else {
            " "
        };

        println!(
            "{:28} vs {:28} | {:5.1}% {} | {:5.1}% | {:5.1}%",
            truncate_name(&m.commander1_name, 28),
            truncate_name(&m.commander2_name, 28),
            m.faction1_win_rate * 100.0,
            indicator,
            p1_rate,
            p2_rate
        );
    }
    println!("\nLegend: ▲ = C1 wins ≥60%, ▼ = C1 wins ≤40%");

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

    // Deck performance section
    if !results.summary.deck_stats.is_empty() {
        println!("\n=== Deck Performance ===");
        println!(
            "{:28} {:10} {:>7}  {:>12}  {:22}",
            "Commander", "Faction", "WinRate", "95% CI", "Worst Matchup"
        );
        println!("{}", "-".repeat(85));

        for ds in &results.summary.deck_stats {
            let status = if ds.win_rate > 0.60 {
                "▲"
            } else if ds.win_rate < 0.40 {
                "▼"
            } else {
                " "
            };

            let worst = ds
                .worst_matchup
                .as_ref()
                .map(|(id, rate)| format!("{} ({:.0}%)", id, rate * 100.0))
                .unwrap_or_else(|| "-".to_string());

            println!(
                "{:28} {:10} {:6.1}% {} [{:4.1}-{:4.1}%]  {}",
                ds.commander_name,
                capitalize(&ds.faction),
                ds.win_rate * 100.0,
                status,
                ds.win_rate_ci_lower * 100.0,
                ds.win_rate_ci_upper * 100.0,
                worst
            );
        }
        println!("\nLegend: ▲ = Above 60% (strong), ▼ = Below 40% (weak)");
    }

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

/// Print a matchup matrix showing deck vs deck win rates.
pub fn print_matchup_matrix(matchups: &[super::types::MatchupResult]) {
    use std::collections::{BTreeSet, HashMap};

    // Collect unique deck IDs (sorted for consistent order)
    let mut deck_ids: BTreeSet<&str> = BTreeSet::new();
    for m in matchups {
        deck_ids.insert(&m.deck1_id);
        deck_ids.insert(&m.deck2_id);
    }
    let deck_ids: Vec<&str> = deck_ids.into_iter().collect();

    if deck_ids.is_empty() {
        return;
    }

    // Build lookup map: (deck1_id, deck2_id) -> win_rate for deck1
    let mut matrix: HashMap<(&str, &str), f64> = HashMap::new();
    for m in matchups {
        matrix.insert((&m.deck1_id, &m.deck2_id), m.faction1_win_rate);
        // The inverse matchup (if we have deck2 as player 1)
        matrix.insert((&m.deck2_id, &m.deck1_id), m.faction2_win_rate);
    }

    // Column width for deck names
    let col_width = 7;

    // Print header
    println!();
    println!("=== Matchup Matrix (Row Deck Win Rate) ===");
    print!("{:20}", "");
    for deck_id in &deck_ids {
        print!(" {:>width$}", abbreviate_deck_name(deck_id, col_width), width = col_width);
    }
    println!();
    println!("{}", "-".repeat(20 + (col_width + 1) * deck_ids.len()));

    // Print each row
    for row_deck in &deck_ids {
        print!("{:20}", abbreviate_deck_name(row_deck, 20));
        for col_deck in &deck_ids {
            if row_deck == col_deck {
                // Diagonal - self-matchup
                print!(" {:>width$}", "-", width = col_width);
            } else if let Some(&win_rate) = matrix.get(&(*row_deck, *col_deck)) {
                print!(" {:>5.1}%", win_rate * 100.0);
            } else {
                // No data for this matchup
                print!(" {:>width$}", "n/a", width = col_width);
            }
        }
        println!();
    }
    println!();
}

/// Export matchup matrix to CSV file.
pub fn export_matrix_csv(
    matchups: &[super::types::MatchupResult],
    path: &Path,
) -> Result<(), ExportError> {
    use std::collections::{BTreeSet, HashMap};
    use std::io::Write;

    // Collect unique deck IDs (sorted for consistent order)
    let mut deck_ids: BTreeSet<&str> = BTreeSet::new();
    for m in matchups {
        deck_ids.insert(&m.deck1_id);
        deck_ids.insert(&m.deck2_id);
    }
    let deck_ids: Vec<&str> = deck_ids.into_iter().collect();

    // Build lookup map
    let mut matrix: HashMap<(&str, &str), f64> = HashMap::new();
    for m in matchups {
        matrix.insert((&m.deck1_id, &m.deck2_id), m.faction1_win_rate);
        matrix.insert((&m.deck2_id, &m.deck1_id), m.faction2_win_rate);
    }

    // Write CSV
    let mut file = std::fs::File::create(path).map_err(ExportError::Io)?;

    // Header row
    write!(file, "deck").map_err(ExportError::Io)?;
    for deck_id in &deck_ids {
        write!(file, ",{}", deck_id).map_err(ExportError::Io)?;
    }
    writeln!(file).map_err(ExportError::Io)?;

    // Data rows
    for row_deck in &deck_ids {
        write!(file, "{}", row_deck).map_err(ExportError::Io)?;
        for col_deck in &deck_ids {
            if row_deck == col_deck {
                write!(file, ",").map_err(ExportError::Io)?;
            } else if let Some(&win_rate) = matrix.get(&(*row_deck, *col_deck)) {
                write!(file, ",{:.3}", win_rate).map_err(ExportError::Io)?;
            } else {
                write!(file, ",").map_err(ExportError::Io)?;
            }
        }
        writeln!(file).map_err(ExportError::Io)?;
    }

    Ok(())
}

/// Abbreviate deck name/ID for compact display.
fn abbreviate_deck_name(name: &str, max_len: usize) -> String {
    // Try to use the first word, capitalize first letter
    let first_word = name.split(|c: char| c == '_' || c == ' ').next().unwrap_or(name);
    let abbreviated = if first_word.len() > max_len {
        &first_word[..max_len]
    } else {
        first_word
    };
    // Capitalize first letter
    let mut chars = abbreviated.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
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
    let parallel = results.config.threads > 1;
    lines.push(format!("Parallel Execution: {} ({} threads)", parallel, results.config.threads));
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

    // Deck performance section
    if !results.summary.deck_stats.is_empty() {
        lines.push(String::new());
        lines.push("--- Deck Performance ---".to_string());
        lines.push(format!(
            "{:28} {:10} {:>7}  {:>12}  {}",
            "Commander", "Faction", "WinRate", "95% CI", "Worst Matchup"
        ));
        lines.push("-".repeat(85));

        for ds in &results.summary.deck_stats {
            let status = if ds.win_rate > 0.60 {
                "▲"
            } else if ds.win_rate < 0.40 {
                "▼"
            } else {
                " "
            };

            let worst = ds
                .worst_matchup
                .as_ref()
                .map(|(id, rate)| format!("{} ({:.0}%)", id, rate * 100.0))
                .unwrap_or_else(|| "-".to_string());

            lines.push(format!(
                "{:28} {:10} {:6.1}% {} [{:4.1}-{:4.1}%]  {}",
                ds.commander_name,
                capitalize(&ds.faction),
                ds.win_rate * 100.0,
                status,
                ds.win_rate_ci_lower * 100.0,
                ds.win_rate_ci_upper * 100.0,
                worst
            ));
        }
        lines.push(String::new());
        lines.push("Legend: ▲ = Above 60% (strong), ▼ = Below 40% (weak)".to_string());
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

/// Truncate a string to a maximum width, adding "..." if truncated.
fn truncate_name(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        format!("{:width$}", s, width = max_width)
    } else if max_width >= 3 {
        format!("{:width$}", format!("{}...", &s[..max_width - 3]), width = max_width)
    } else {
        format!("{:width$}", s, width = max_width)
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
