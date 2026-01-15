//! Validation report formatting and output.
//!
//! Provides utilities for displaying validation results.

use std::path::Path;
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
    println!("\nResults saved to: {:?}", path);
    Ok(())
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
