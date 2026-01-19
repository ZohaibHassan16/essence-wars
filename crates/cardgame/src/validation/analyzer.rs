//! Balance analysis for validation results.
//!
//! Analyzes matchup results to determine overall game balance.

use std::collections::HashMap;

use crate::diagnostics::statistics::{chi_square_test, wilson_score_interval};

use super::types::{BalanceStatus, BalanceSummary, MatchupP1Stats, MatchupResult, P1P2Summary};

/// Analyzer for balance validation results.
pub struct BalanceAnalyzer {
    /// Ideal P1 win rate range (lower bound).
    p1_ideal_low: f64,
    /// Ideal P1 win rate range (upper bound).
    p1_ideal_high: f64,
    /// Warning P1 win rate range (lower bound).
    p1_warning_low: f64,
    /// Warning P1 win rate range (upper bound).
    p1_warning_high: f64,
    /// Faction delta threshold for balanced status.
    faction_delta_balanced: f64,
    /// Faction delta threshold for warning status.
    faction_delta_warning: f64,
}

impl Default for BalanceAnalyzer {
    fn default() -> Self {
        Self {
            // P1 should win 50-55% ideally (slight first-player advantage acceptable)
            p1_ideal_low: 0.50,
            p1_ideal_high: 0.55,
            // Warning range: 45-60%
            p1_warning_low: 0.45,
            p1_warning_high: 0.60,
            // Faction delta < 10% is balanced
            faction_delta_balanced: 0.10,
            // Faction delta < 15% is warning
            faction_delta_warning: 0.15,
        }
    }
}

impl BalanceAnalyzer {
    /// Create a new balance analyzer with default thresholds.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a balance analyzer with custom P1 thresholds.
    pub fn with_p1_thresholds(
        mut self,
        ideal_low: f64,
        ideal_high: f64,
        warning_low: f64,
        warning_high: f64,
    ) -> Self {
        self.p1_ideal_low = ideal_low;
        self.p1_ideal_high = ideal_high;
        self.p1_warning_low = warning_low;
        self.p1_warning_high = warning_high;
        self
    }

    /// Create a balance analyzer with custom faction delta thresholds.
    pub fn with_faction_thresholds(mut self, balanced: f64, warning: f64) -> Self {
        self.faction_delta_balanced = balanced;
        self.faction_delta_warning = warning;
        self
    }

    /// Analyze matchup results and return a balance summary.
    pub fn analyze(&self, matchups: &[MatchupResult]) -> BalanceSummary {
        let mut warnings = Vec::new();

        // Calculate P1 win rate across all games
        let (p1_win_rate, _total_draws) = self.calculate_p1_win_rate(matchups);

        // Determine P1 balance status
        let p1_status = self.classify_p1_balance(p1_win_rate, &mut warnings);

        // Calculate faction win rates
        let faction_win_rates = self.calculate_faction_win_rates(matchups);

        // Calculate max faction delta
        let max_faction_delta = self.calculate_max_delta(&faction_win_rates);

        // Determine faction balance status
        let faction_status = self.classify_faction_balance(max_faction_delta, &mut warnings);

        // Overall status is the worse of P1 and faction
        let overall_status = match (p1_status, faction_status) {
            (BalanceStatus::Imbalanced, _) | (_, BalanceStatus::Imbalanced) => {
                BalanceStatus::Imbalanced
            }
            (BalanceStatus::Warning, _) | (_, BalanceStatus::Warning) => BalanceStatus::Warning,
            _ => BalanceStatus::Balanced,
        };

        // Build P1/P2 diagnostic summary
        let p1_p2_diagnostics = self.build_p1_p2_summary(matchups);

        BalanceSummary {
            p1_win_rate,
            p1_status,
            faction_win_rates,
            max_faction_delta,
            faction_status,
            overall_status,
            warnings,
            p1_p2_diagnostics,
        }
    }

    /// Build P1/P2 diagnostic summary from matchup results.
    fn build_p1_p2_summary(&self, matchups: &[MatchupResult]) -> P1P2Summary {
        // Calculate overall P1 wins
        let mut total_p1_wins = 0u32;
        let mut total_decisive = 0u32;
        let mut by_matchup = HashMap::new();

        for m in matchups {
            // P1 wins from both directions
            let matchup_p1_wins = m.f1_as_p1_wins + (m.total_games / 2 - m.f1_as_p2_wins - m.draws / 2);
            let matchup_decisive = m.total_games - m.draws;

            total_p1_wins += matchup_p1_wins;
            total_decisive += matchup_decisive;

            // Per-matchup stats from diagnostics
            let matchup_key = format!("{}-{}", m.faction1, m.faction2);
            by_matchup.insert(
                matchup_key,
                MatchupP1Stats {
                    p1_rate: m.diagnostics.p1_win_rate,
                    significance: m.diagnostics.p1_significance.clone(),
                },
            );
        }

        // Calculate overall statistics
        let overall_p1_win_rate = if total_decisive > 0 {
            total_p1_wins as f64 / total_decisive as f64
        } else {
            0.5
        };

        let (ci_lower, ci_upper) = if total_decisive > 0 {
            wilson_score_interval(total_p1_wins as usize, total_decisive as usize, 0.95)
        } else {
            (0.0, 1.0)
        };

        let (_, p_value) = if total_decisive > 0 {
            chi_square_test(total_p1_wins as usize, total_decisive as usize, 0.5)
        } else {
            (0.0, 1.0)
        };

        let significance = if p_value < 0.01 {
            "highly_significant".to_string()
        } else if p_value < 0.05 {
            "significant".to_string()
        } else if p_value < 0.10 {
            "marginal".to_string()
        } else {
            "not_significant".to_string()
        };

        // Determine assessment
        let assessment = if ci_upper < 0.48 {
            "p2_favored".to_string()
        } else if ci_lower > 0.52 && ci_lower <= 0.55 {
            "slight_p1_advantage".to_string()
        } else if ci_lower > 0.55 {
            "p1_favored".to_string()
        } else if (0.48..0.52).contains(&ci_upper) {
            "slight_p2_advantage".to_string()
        } else {
            "balanced".to_string()
        };

        P1P2Summary {
            overall_p1_win_rate,
            overall_p1_ci_lower: ci_lower,
            overall_p1_ci_upper: ci_upper,
            significance,
            assessment,
            by_matchup,
        }
    }

    /// Calculate P1 win rate across all matchups.
    fn calculate_p1_win_rate(&self, matchups: &[MatchupResult]) -> (f64, u32) {
        let mut total_p1_wins = 0u32;
        let mut total_games = 0u32;
        let mut total_draws = 0u32;

        for m in matchups {
            // F1 as P1 wins
            total_p1_wins += m.f1_as_p1_wins;
            // F2 as P1 wins (= games - F1 as P2 wins - draws/2)
            total_p1_wins += m.total_games / 2 - m.f1_as_p2_wins - m.draws / 2;
            total_games += m.total_games;
            total_draws += m.draws;
        }

        let decisive_games = total_games - total_draws;
        let p1_win_rate = if decisive_games > 0 {
            total_p1_wins as f64 / decisive_games as f64
        } else {
            0.5
        };

        (p1_win_rate, total_draws)
    }

    /// Classify P1 balance status.
    fn classify_p1_balance(&self, p1_win_rate: f64, warnings: &mut Vec<String>) -> BalanceStatus {
        if (self.p1_ideal_low..=self.p1_ideal_high).contains(&p1_win_rate) {
            BalanceStatus::Balanced
        } else if (self.p1_warning_low..=self.p1_warning_high).contains(&p1_win_rate) {
            warnings.push(format!(
                "P1 win rate {:.1}% is outside ideal range ({:.0}-{:.0}%)",
                p1_win_rate * 100.0,
                self.p1_ideal_low * 100.0,
                self.p1_ideal_high * 100.0
            ));
            BalanceStatus::Warning
        } else {
            warnings.push(format!(
                "P1 win rate {:.1}% is significantly imbalanced",
                p1_win_rate * 100.0
            ));
            BalanceStatus::Imbalanced
        }
    }

    /// Calculate win rates for each faction.
    fn calculate_faction_win_rates(&self, matchups: &[MatchupResult]) -> HashMap<String, f64> {
        let mut faction_wins: HashMap<String, u32> = HashMap::new();
        let mut faction_games: HashMap<String, u32> = HashMap::new();

        for m in matchups {
            *faction_wins.entry(m.faction1.clone()).or_insert(0) += m.faction1_total_wins;
            *faction_wins.entry(m.faction2.clone()).or_insert(0) += m.faction2_total_wins;
            *faction_games.entry(m.faction1.clone()).or_insert(0) += m.total_games - m.draws;
            *faction_games.entry(m.faction2.clone()).or_insert(0) += m.total_games - m.draws;
        }

        faction_wins
            .iter()
            .map(|(faction, wins)| {
                let games = faction_games.get(faction).copied().unwrap_or(1);
                (faction.clone(), *wins as f64 / games as f64)
            })
            .collect()
    }

    /// Calculate the maximum difference between faction win rates.
    fn calculate_max_delta(&self, faction_win_rates: &HashMap<String, f64>) -> f64 {
        let rates: Vec<f64> = faction_win_rates.values().copied().collect();
        if rates.len() >= 2 {
            let max = rates.iter().cloned().fold(f64::MIN, f64::max);
            let min = rates.iter().cloned().fold(f64::MAX, f64::min);
            max - min
        } else {
            0.0
        }
    }

    /// Classify faction balance status.
    fn classify_faction_balance(
        &self,
        max_delta: f64,
        warnings: &mut Vec<String>,
    ) -> BalanceStatus {
        if max_delta < self.faction_delta_balanced {
            BalanceStatus::Balanced
        } else if max_delta < self.faction_delta_warning {
            warnings.push(format!(
                "Faction delta {:.1}% is above target (<{:.0}%)",
                max_delta * 100.0,
                self.faction_delta_balanced * 100.0
            ));
            BalanceStatus::Warning
        } else {
            warnings.push(format!(
                "Faction delta {:.1}% indicates significant imbalance",
                max_delta * 100.0
            ));
            BalanceStatus::Imbalanced
        }
    }

    /// Check if a specific win rate indicates imbalance.
    pub fn is_imbalanced(&self, win_rate: f64) -> bool {
        win_rate < self.p1_warning_low || win_rate > self.p1_warning_high
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::MatchupDiagnostics;

    fn create_test_matchup(
        f1: &str,
        f2: &str,
        f1_wins: u32,
        f2_wins: u32,
        draws: u32,
    ) -> MatchupResult {
        let total = f1_wins + f2_wins + draws;
        let decisive = total - draws;
        MatchupResult {
            faction1: f1.to_string(),
            faction2: f2.to_string(),
            deck1_id: format!("{}_deck", f1),
            deck2_id: format!("{}_deck", f2),
            f1_as_p1_wins: f1_wins / 2,
            f1_as_p1_games: total / 2,
            f1_as_p2_wins: f1_wins - f1_wins / 2,
            f1_as_p2_games: total / 2,
            faction1_total_wins: f1_wins,
            faction2_total_wins: f2_wins,
            draws,
            total_games: total,
            faction1_win_rate: if decisive > 0 {
                f1_wins as f64 / decisive as f64
            } else {
                0.5
            },
            faction2_win_rate: if decisive > 0 {
                f2_wins as f64 / decisive as f64
            } else {
                0.5
            },
            avg_turns: 25.0,
            total_time_secs: 1.0,
            diagnostics: MatchupDiagnostics::default(),
        }
    }

    #[test]
    fn test_balanced_results() {
        let matchups = vec![
            create_test_matchup("argentum", "symbiote", 50, 50, 0),
            create_test_matchup("argentum", "obsidion", 50, 50, 0),
            create_test_matchup("symbiote", "obsidion", 50, 50, 0),
        ];

        let analyzer = BalanceAnalyzer::new();
        let summary = analyzer.analyze(&matchups);

        assert_eq!(summary.faction_status, BalanceStatus::Balanced);
        assert!(summary.max_faction_delta < 0.01);
    }

    #[test]
    fn test_imbalanced_faction() {
        let matchups = vec![
            create_test_matchup("argentum", "symbiote", 80, 20, 0),
            create_test_matchup("argentum", "obsidion", 80, 20, 0),
            create_test_matchup("symbiote", "obsidion", 50, 50, 0),
        ];

        let analyzer = BalanceAnalyzer::new();
        let summary = analyzer.analyze(&matchups);

        // Argentum should have much higher win rate
        assert!(summary.max_faction_delta > 0.15);
        assert_eq!(summary.faction_status, BalanceStatus::Imbalanced);
    }

    #[test]
    fn test_p1_advantage() {
        // Create matchups where P1 always wins
        let mut matchups = vec![create_test_matchup("argentum", "symbiote", 50, 50, 0)];
        // Manually set P1 wins high
        matchups[0].f1_as_p1_wins = 45; // 90% P1 win rate as F1
        matchups[0].f1_as_p2_wins = 5; // 10% P2 win rate as F1 (= 90% P1 win rate as F2)

        let analyzer = BalanceAnalyzer::new();
        let summary = analyzer.analyze(&matchups);

        // P1 win rate should be very high
        assert!(summary.p1_win_rate > 0.6 || summary.p1_win_rate < 0.4);
    }
}
