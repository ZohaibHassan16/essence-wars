//! Report generation for card statistics.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use serde::Serialize;

use crate::cards::CardDatabase;
use crate::types::CardId;

use super::collector::CardStatsCollector;
use super::types::CardPlayStats;

/// Card statistics with computed metrics for reporting.
#[derive(Debug, Clone, Serialize)]
pub struct CardReport {
    pub card_id: u16,
    pub name: String,
    pub cost: u8,
    pub card_type: String,
    pub faction: String,
    pub plays: u32,
    pub games_in: u32,
    pub wins: u32,
    pub win_rate: f64,
    pub delta: f64,
    pub impact: f64,
    pub early_pct: f64,
    pub mid_pct: f64,
    pub late_pct: f64,
}

/// Full report structure for JSON export.
#[derive(Debug, Clone, Serialize)]
pub struct FullReport {
    pub config: ReportConfig,
    pub summary: ReportSummary,
    pub cards: Vec<CardReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportConfig {
    pub mode: String,
    pub deck: Option<String>,
    pub deck1: Option<String>,
    pub deck2: Option<String>,
    pub games: u32,
    pub bot: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportSummary {
    pub total_games: u32,
    pub p1_wins: u32,
    pub p2_wins: u32,
    pub draws: u32,
    pub baseline_win_rate: f64,
}

/// Build a card report from stats.
pub fn build_card_report(
    card_id: CardId,
    stats: &CardPlayStats,
    card_db: &CardDatabase,
    baseline: f64,
    total_games: u32,
) -> CardReport {
    let (name, cost, card_type, faction) = if let Some(card) = card_db.get(card_id) {
        let card_type = if card.is_creature() {
            "Creature"
        } else if card.is_spell() {
            "Spell"
        } else {
            "Support"
        };
        let faction = faction_from_id(card_id);
        (card.name.clone(), card.cost, card_type.to_string(), faction)
    } else if let Some(cmd) = card_db.get_commander(card_id) {
        let faction = format!("{:?}", cmd.faction);
        (cmd.name.clone(), 0, "Commander".to_string(), faction)
    } else {
        (
            format!("Unknown ({})", card_id.0),
            0,
            "Unknown".to_string(),
            "Unknown".to_string(),
        )
    };

    CardReport {
        card_id: card_id.0,
        name,
        cost,
        card_type,
        faction,
        plays: stats.play_count,
        games_in: stats.games_played_in,
        wins: stats.wins_when_played,
        win_rate: stats.win_rate() * 100.0,
        delta: stats.win_delta(baseline) * 100.0,
        impact: stats.impact_score(baseline, total_games),
        early_pct: stats.early_pct(),
        mid_pct: stats.mid_pct(),
        late_pct: stats.late_pct(),
    }
}

/// Determine faction from card ID range.
fn faction_from_id(card_id: CardId) -> String {
    match card_id.0 {
        1000..=1999 => "Argentum".to_string(),
        2000..=2999 => "Symbiote".to_string(),
        3000..=3999 => "Obsidion".to_string(),
        4000..=4999 => "Neutral".to_string(),
        5000..=5999 => "Commander".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Print console report.
pub fn print_report(
    collector: &CardStatsCollector,
    card_db: &CardDatabase,
    mode: &str,
    deck_info: &str,
    min_plays: u32,
    top_n: usize,
) {
    let baseline = collector.baseline_win_rate();
    let total_games = collector.total_games;

    println!("=== Card Performance Analysis ===");
    println!("Mode: {}", mode);
    println!("Deck: {}", deck_info);
    println!(
        "Games: {} | P1 Wins: {} | P2 Wins: {} | Draws: {}",
        total_games, collector.p1_wins, collector.p2_wins, collector.draws
    );
    println!("Baseline Win Rate: {:.1}%", baseline * 100.0);
    println!();

    // Filter by min plays and sort by impact
    let mut cards: Vec<_> = collector
        .filter_by_min_plays(min_plays)
        .into_iter()
        .map(|(id, stats)| build_card_report(id, stats, card_db, baseline, total_games))
        .collect();

    cards.sort_by(|a, b| {
        b.impact
            .partial_cmp(&a.impact)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Top performers
    println!("=== Top {} Cards by Win Contribution ===", top_n);
    println!(
        "{:>4}  {:<24} {:>4}  {:>5}  {:>7}  {:>6}  {:>6}",
        "Rank", "Card Name", "Cost", "Plays", "WinRate", "Delta", "Impact"
    );
    println!("{}", "-".repeat(72));

    for (i, card) in cards.iter().take(top_n).enumerate() {
        println!(
            "{:>4}  {:<24} {:>4}  {:>5}  {:>6.1}%  {:>+5.1}%  {:>+6.1}",
            i + 1,
            truncate(&card.name, 24),
            card.cost,
            card.plays,
            card.win_rate,
            card.delta,
            card.impact
        );
    }
    println!();

    // Bottom performers
    println!("=== Bottom {} Cards by Win Contribution ===", top_n);
    println!(
        "{:>4}  {:<24} {:>4}  {:>5}  {:>7}  {:>6}  {:>6}",
        "Rank", "Card Name", "Cost", "Plays", "WinRate", "Delta", "Impact"
    );
    println!("{}", "-".repeat(72));

    let bottom: Vec<_> = cards.iter().rev().take(top_n).collect();
    for (i, card) in bottom.iter().enumerate() {
        println!(
            "{:>4}  {:<24} {:>4}  {:>5}  {:>6.1}%  {:>+5.1}%  {:>+6.1}",
            i + 1,
            truncate(&card.name, 24),
            card.cost,
            card.plays,
            card.win_rate,
            card.delta,
            card.impact
        );
    }
    println!();

    // Play timing for top cards
    println!("=== Play Timing Distribution (Top Cards) ===");
    println!(
        "{:<24} {:>6}  {:>6}  {:>6}",
        "Card Name", "Early", "Mid", "Late"
    );
    println!("{}", "-".repeat(48));

    for card in cards.iter().take(top_n) {
        println!(
            "{:<24} {:>5.1}%  {:>5.1}%  {:>5.1}%",
            truncate(&card.name, 24),
            card.early_pct,
            card.mid_pct,
            card.late_pct
        );
    }
}

/// Export to CSV.
pub fn export_csv(
    collector: &CardStatsCollector,
    card_db: &CardDatabase,
    path: &Path,
    min_plays: u32,
) -> std::io::Result<()> {
    let baseline = collector.baseline_win_rate();
    let total_games = collector.total_games;

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Header
    writeln!(
        writer,
        "card_id,name,cost,type,faction,plays,games_in,wins,win_rate,delta,impact,early_pct,mid_pct,late_pct"
    )?;

    // Data rows
    let cards: Vec<_> = collector
        .filter_by_min_plays(min_plays)
        .into_iter()
        .map(|(id, stats)| build_card_report(id, stats, card_db, baseline, total_games))
        .collect();

    for card in cards {
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{:.2},{:.2},{:.2},{:.1},{:.1},{:.1}",
            card.card_id,
            escape_csv(&card.name),
            card.cost,
            card.card_type,
            card.faction,
            card.plays,
            card.games_in,
            card.wins,
            card.win_rate,
            card.delta,
            card.impact,
            card.early_pct,
            card.mid_pct,
            card.late_pct
        )?;
    }

    writer.flush()?;
    Ok(())
}

/// Export to JSON.
pub fn export_json(
    collector: &CardStatsCollector,
    card_db: &CardDatabase,
    path: &Path,
    config: ReportConfig,
    min_plays: u32,
) -> std::io::Result<()> {
    let baseline = collector.baseline_win_rate();
    let total_games = collector.total_games;

    let cards: Vec<_> = collector
        .filter_by_min_plays(min_plays)
        .into_iter()
        .map(|(id, stats)| build_card_report(id, stats, card_db, baseline, total_games))
        .collect();

    let report = FullReport {
        config,
        summary: ReportSummary {
            total_games,
            p1_wins: collector.p1_wins,
            p2_wins: collector.p2_wins,
            draws: collector.draws,
            baseline_win_rate: baseline,
        },
        cards,
    };

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &report)?;

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
