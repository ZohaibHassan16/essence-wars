//! Unit tests for arena stats.

use std::time::Duration;

use cardgame::arena::{MatchStats, MatchupStats};
use cardgame::types::PlayerId;

#[test]
fn test_matchup_stats_recording() {
    let mut stats = MatchupStats::new();

    stats.record_game(Some(PlayerId::PLAYER_ONE), 10, Duration::from_millis(100));
    stats.record_game(Some(PlayerId::PLAYER_TWO), 15, Duration::from_millis(150));
    stats.record_game(Some(PlayerId::PLAYER_ONE), 12, Duration::from_millis(120));
    stats.record_game(None, 30, Duration::from_millis(300));

    assert_eq!(stats.games, 4);
    assert_eq!(stats.bot1_wins, 2);
    assert_eq!(stats.bot2_wins, 1);
    assert_eq!(stats.draws, 1);
    assert_eq!(stats.total_turns, 67);
    assert_eq!(stats.total_time, Duration::from_millis(670));
}

#[test]
fn test_win_rates() {
    let mut stats = MatchupStats::new();
    stats.record_game(Some(PlayerId::PLAYER_ONE), 10, Duration::from_millis(100));
    stats.record_game(Some(PlayerId::PLAYER_ONE), 10, Duration::from_millis(100));
    stats.record_game(Some(PlayerId::PLAYER_TWO), 10, Duration::from_millis(100));
    stats.record_game(None, 10, Duration::from_millis(100));

    assert!((stats.bot1_win_rate() - 0.5).abs() < 0.001);
    assert!((stats.bot2_win_rate() - 0.25).abs() < 0.001);
    assert!((stats.draw_rate() - 0.25).abs() < 0.001);
}

#[test]
fn test_stats_merge() {
    let mut stats1 = MatchupStats::new();
    stats1.record_game(Some(PlayerId::PLAYER_ONE), 10, Duration::from_millis(100));

    let mut stats2 = MatchupStats::new();
    stats2.record_game(Some(PlayerId::PLAYER_TWO), 15, Duration::from_millis(150));
    stats2.record_game(Some(PlayerId::PLAYER_TWO), 12, Duration::from_millis(120));

    stats1.merge(&stats2);

    assert_eq!(stats1.games, 3);
    assert_eq!(stats1.bot1_wins, 1);
    assert_eq!(stats1.bot2_wins, 2);
}

#[test]
fn test_match_stats_summary() {
    let mut stats = MatchStats::new("GreedyBot".to_string(), "RandomBot".to_string());
    stats.record_game(Some(PlayerId::PLAYER_ONE), 10, Duration::from_millis(100));

    let summary = stats.summary();
    assert!(summary.contains("GreedyBot"));
    assert!(summary.contains("RandomBot"));
    assert!(summary.contains("Games: 1"));
}
