//! Unit tests for arena stats and card synergy tracking.

use std::collections::HashSet;
use std::time::Duration;

use cardgame::arena::{MatchStats, MatchupStats};
use cardgame::stats::{CardPair, CardPairStats, CardStatsCollector, SynergyCollector};
use cardgame::types::{CardId, PlayerId};

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

// ============================================================================
// Card Synergy Tests
// ============================================================================

#[test]
fn test_card_pair_canonical_ordering() {
    let pair1 = CardPair::new(CardId(1000), CardId(2000));
    let pair2 = CardPair::new(CardId(2000), CardId(1000));

    // Same pair regardless of order
    assert_eq!(pair1, pair2);

    // Smaller ID always first
    assert_eq!(pair1.first(), CardId(1000));
    assert_eq!(pair1.second(), CardId(2000));
}

#[test]
fn test_card_pair_stats_win_rate() {
    let stats = CardPairStats {
        games_together: 20,
        wins_together: 14, // 70% win rate
    };

    assert!((stats.pair_win_rate() - 0.70).abs() < 0.001);
}

#[test]
fn test_card_pair_stats_synergy_bonus() {
    let stats = CardPairStats {
        games_together: 20,
        wins_together: 14, // 70% pair win rate
    };

    let card1_wr = 0.55; // 55%
    let card2_wr = 0.50; // 50%
    // Expected: (55% + 50%) / 2 = 52.5%
    // Synergy bonus: 70% - 52.5% = 17.5%

    let bonus = stats.synergy_bonus(card1_wr, card2_wr);
    assert!((bonus - 0.175).abs() < 0.001);
}

#[test]
fn test_synergy_collector_aggregation() {
    let mut collector = SynergyCollector::new();

    let mut cards: HashSet<CardId> = HashSet::new();
    cards.insert(CardId(1000));
    cards.insert(CardId(1001));
    cards.insert(CardId(1002));

    // Record 2 wins, 1 loss
    collector.aggregate_player_pairs(&cards, true);
    collector.aggregate_player_pairs(&cards, true);
    collector.aggregate_player_pairs(&cards, false);

    let pair = CardPair::new(CardId(1000), CardId(1001));
    let stats = collector.pair_stats.get(&pair).unwrap();
    assert_eq!(stats.games_together, 3);
    assert_eq!(stats.wins_together, 2);
    assert!((stats.pair_win_rate() - 0.667).abs() < 0.01);
}

#[test]
fn test_synergy_pair_count() {
    // n cards = n*(n-1)/2 pairs
    // 5 cards = 10 pairs
    let mut collector = SynergyCollector::new();
    let cards: HashSet<CardId> = (1000..1005).map(CardId).collect();
    collector.aggregate_player_pairs(&cards, true);
    assert_eq!(collector.pair_stats.len(), 10);
}

#[test]
fn test_synergy_collector_merge() {
    let mut c1 = SynergyCollector::new();
    let mut c2 = SynergyCollector::new();

    let cards: HashSet<CardId> = vec![CardId(1000), CardId(1001)].into_iter().collect();
    c1.aggregate_player_pairs(&cards, true);
    c2.aggregate_player_pairs(&cards, false);

    c1.merge(&c2);

    let pair = CardPair::new(CardId(1000), CardId(1001));
    let stats = c1.pair_stats.get(&pair).unwrap();
    assert_eq!(stats.games_together, 2);
    assert_eq!(stats.wins_together, 1);
}

#[test]
fn test_card_stats_collector_with_synergy() {
    let collector = CardStatsCollector::with_synergy(true);
    assert!(collector.synergy.is_some());

    let collector = CardStatsCollector::with_synergy(false);
    assert!(collector.synergy.is_none());

    let collector = CardStatsCollector::new();
    assert!(collector.synergy.is_none());
}

#[test]
fn test_synergy_filter_by_min_cooccur() {
    let mut collector = SynergyCollector::new();

    let cards1: HashSet<CardId> = vec![CardId(1000), CardId(1001)].into_iter().collect();
    let cards2: HashSet<CardId> = vec![CardId(2000), CardId(2001)].into_iter().collect();

    // Record pair1 5 times, pair2 2 times
    for _ in 0..5 {
        collector.aggregate_player_pairs(&cards1, true);
    }
    for _ in 0..2 {
        collector.aggregate_player_pairs(&cards2, true);
    }

    // Filter with min_cooccur=3
    let filtered = collector.filter_by_min_cooccur(3);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].0, CardPair::new(CardId(1000), CardId(1001)));
}
