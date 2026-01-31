//! Swiss tournament system for deck competition.
//!
//! Provides infrastructure for running Swiss-system tournaments between decks,
//! with score-based pairing, tiebreakers, and ELO integration.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::arena::{run_match_parallel, EloTracker, MatchConfig, MatchStats};
use crate::bots::{AlphaBetaConfig, BotType, BotWeights, MctsConfig};
use crate::cards::CardDatabase;
use crate::decks::{DeckDefinition, DeckRegistry, Faction};
use crate::types::CardId;

/// Configuration for a Swiss tournament.
#[derive(Clone, Debug)]
pub struct SwissConfig {
    /// Number of rounds
    pub rounds: usize,
    /// Number of games per match
    pub games_per_match: usize,
    /// Bot type for all participants
    pub bot_type: BotType,
    /// Optional weights for bots
    pub weights: Option<BotWeights>,
    /// MCTS configuration
    pub mcts_config: MctsConfig,
    /// Alpha-Beta configuration
    pub alphabeta_config: AlphaBetaConfig,
    /// Base seed for reproducibility
    pub seed: u64,
    /// Show progress during matches
    pub show_progress: bool,
    /// Optional faction filter
    pub faction_filter: Option<Faction>,
}

impl SwissConfig {
    /// Create a new Swiss tournament configuration.
    pub fn new(rounds: usize, games_per_match: usize, bot_type: BotType, seed: u64) -> Self {
        Self {
            rounds,
            games_per_match,
            bot_type,
            weights: None,
            mcts_config: MctsConfig::default(),
            alphabeta_config: AlphaBetaConfig::default(),
            seed,
            show_progress: false,
            faction_filter: None,
        }
    }

    /// Set custom weights.
    pub fn with_weights(mut self, weights: Option<BotWeights>) -> Self {
        self.weights = weights;
        self
    }

    /// Set MCTS configuration.
    pub fn with_mcts_config(mut self, config: MctsConfig) -> Self {
        self.mcts_config = config;
        self
    }

    /// Set Alpha-Beta configuration.
    pub fn with_alphabeta_config(mut self, config: AlphaBetaConfig) -> Self {
        self.alphabeta_config = config;
        self
    }

    /// Enable progress display.
    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }

    /// Filter to a specific faction.
    pub fn with_faction(mut self, faction: Option<Faction>) -> Self {
        self.faction_filter = faction;
        self
    }

    /// Calculate recommended rounds for participant count.
    pub fn recommended_rounds(participant_count: usize) -> usize {
        if participant_count <= 1 {
            return 0;
        }
        // ceil(log2(n)) + 1 for proper Swiss separation
        let log2 = (participant_count as f64).log2().ceil() as usize;
        log2 + 1
    }
}

/// Match result from one player's perspective.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TournamentMatchResult {
    Win,
    Loss,
    Draw,
}

impl TournamentMatchResult {
    /// Get the inverse result.
    pub fn inverse(&self) -> Self {
        match self {
            TournamentMatchResult::Win => TournamentMatchResult::Loss,
            TournamentMatchResult::Loss => TournamentMatchResult::Win,
            TournamentMatchResult::Draw => TournamentMatchResult::Draw,
        }
    }
}

/// Outcome of a single match in the tournament.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchOutcome {
    /// Match result from this player's perspective
    pub result: TournamentMatchResult,
    /// Games won
    pub wins: usize,
    /// Games lost
    pub losses: usize,
    /// Games drawn
    pub draws: usize,
    /// Total games played
    pub games: usize,
    /// Win rate (0.0 to 1.0)
    pub win_rate: f64,
}

impl MatchOutcome {
    /// Create from MatchStats.
    pub fn from_stats(stats: &MatchStats, is_player1: bool) -> Self {
        let (wins, losses) = if is_player1 {
            (stats.overall.bot1_wins, stats.overall.bot2_wins)
        } else {
            (stats.overall.bot2_wins, stats.overall.bot1_wins)
        };

        let result = if wins > losses {
            TournamentMatchResult::Win
        } else if losses > wins {
            TournamentMatchResult::Loss
        } else {
            TournamentMatchResult::Draw
        };

        Self {
            result,
            wins,
            losses,
            draws: stats.overall.draws,
            games: stats.overall.games,
            win_rate: if stats.overall.games > 0 {
                wins as f64 / stats.overall.games as f64
            } else {
                0.5
            },
        }
    }

    /// Create the inverse outcome (for opponent's perspective).
    pub fn inverse(&self) -> Self {
        Self {
            result: self.result.inverse(),
            wins: self.losses,
            losses: self.wins,
            draws: self.draws,
            games: self.games,
            win_rate: if self.games > 0 {
                self.losses as f64 / self.games as f64
            } else {
                0.5
            },
        }
    }
}

/// A participant in the Swiss tournament.
#[derive(Clone, Debug)]
pub struct Participant {
    /// Unique identifier (deck ID)
    pub deck_id: String,
    /// Full deck definition
    pub deck: DeckDefinition,
    /// Current tournament score (match points)
    pub score: u32,
    /// Opponent deck IDs faced
    pub opponents: Vec<String>,
    /// Match history (deck_id -> MatchOutcome)
    pub match_history: HashMap<String, MatchOutcome>,
}

impl Participant {
    /// Create a new participant from a deck definition.
    pub fn new(deck: DeckDefinition) -> Self {
        Self {
            deck_id: deck.id.clone(),
            deck,
            score: 0,
            opponents: Vec::new(),
            match_history: HashMap::new(),
        }
    }

    /// Record a match result against an opponent.
    pub fn record_match(&mut self, opponent_id: &str, outcome: MatchOutcome) {
        self.opponents.push(opponent_id.to_string());
        match outcome.result {
            TournamentMatchResult::Win => self.score += 3,
            TournamentMatchResult::Draw => self.score += 1,
            TournamentMatchResult::Loss => {} // 0 points
        }
        self.match_history.insert(opponent_id.to_string(), outcome);
    }

    /// Check if this participant has already played against another.
    pub fn has_played(&self, opponent_id: &str) -> bool {
        self.opponents.contains(&opponent_id.to_string())
    }
}

/// A pairing for a single round.
#[derive(Clone, Debug)]
pub struct Pairing {
    /// First participant deck ID
    pub player1_id: String,
    /// Second participant deck ID (None if bye)
    pub player2_id: Option<String>,
}

impl Pairing {
    /// Create a normal pairing.
    pub fn new(player1_id: String, player2_id: String) -> Self {
        Self {
            player1_id,
            player2_id: Some(player2_id),
        }
    }

    /// Create a bye pairing.
    pub fn bye(player_id: String) -> Self {
        Self {
            player1_id: player_id,
            player2_id: None,
        }
    }

    /// Check if this is a bye.
    pub fn is_bye(&self) -> bool {
        self.player2_id.is_none()
    }
}

/// Results of a single round.
#[derive(Clone, Debug)]
pub struct RoundResult {
    /// Round number (1-indexed)
    pub round_number: usize,
    /// Pairings for this round
    pub pairings: Vec<Pairing>,
    /// Match results (player1_id -> MatchStats)
    pub results: HashMap<String, MatchStats>,
    /// Total duration of the round
    pub duration: Duration,
}

impl RoundResult {
    /// Create a new round result.
    pub fn new(round_number: usize, pairings: Vec<Pairing>) -> Self {
        Self {
            round_number,
            pairings,
            results: HashMap::new(),
            duration: Duration::ZERO,
        }
    }
}

/// Tournament standings entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Standing {
    /// Rank (1-indexed)
    pub rank: usize,
    /// Deck ID
    pub deck_id: String,
    /// Deck name
    pub deck_name: String,
    /// Commander name
    pub commander_name: String,
    /// Faction
    pub faction: String,
    /// Tournament score (match points)
    pub score: u32,
    /// Buchholz score (sum of opponents' scores)
    pub buchholz: u32,
    /// Total game win rate across all matches
    pub win_rate: f64,
    /// Games won
    pub games_won: usize,
    /// Games lost
    pub games_lost: usize,
    /// Games drawn
    pub games_drawn: usize,
    /// Match wins
    pub match_wins: usize,
    /// Match losses
    pub match_losses: usize,
    /// Match draws
    pub match_draws: usize,
}

/// Complete tournament results.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TournamentResult {
    /// Timestamp
    pub timestamp: String,
    /// Number of participants
    pub participants: usize,
    /// Number of rounds
    pub rounds: usize,
    /// Games per match
    pub games_per_match: usize,
    /// Bot type used
    pub bot_type: String,
    /// Base seed
    pub seed: u64,
    /// Total duration in seconds
    pub total_duration_secs: f64,
    /// Final standings
    pub standings: Vec<Standing>,
}

impl TournamentResult {
    /// Export to JSON.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

/// Swiss tournament state and execution.
pub struct SwissTournament {
    /// Tournament configuration
    config: SwissConfig,
    /// All participants
    participants: HashMap<String, Participant>,
    /// Participant order (for consistent iteration)
    participant_order: Vec<String>,
    /// Round results
    rounds: Vec<RoundResult>,
    /// Participants who have received a bye
    bye_recipients: HashSet<String>,
    /// ELO tracker for rating updates
    elo_tracker: EloTracker,
}

impl SwissTournament {
    /// Create a new Swiss tournament from deck registry.
    pub fn new(registry: &DeckRegistry, config: SwissConfig) -> Self {
        let decks: Vec<&DeckDefinition> = match config.faction_filter {
            Some(faction) => registry.decks_for_faction(faction),
            None => registry.decks().collect(),
        };

        let mut participants = HashMap::new();
        let mut participant_order = Vec::new();

        // Sort by deck ID for consistent ordering
        let mut sorted_decks: Vec<_> = decks.into_iter().collect();
        sorted_decks.sort_by(|a, b| a.id.cmp(&b.id));

        for deck in sorted_decks {
            let participant = Participant::new(deck.clone());
            participant_order.push(deck.id.clone());
            participants.insert(deck.id.clone(), participant);
        }

        // Load ELO tracker
        let elo_path = PathBuf::from("data/ratings/deck_elo.json");
        let elo_tracker = EloTracker::load_or_create(&elo_path);

        Self {
            config,
            participants,
            participant_order,
            rounds: Vec::new(),
            bye_recipients: HashSet::new(),
            elo_tracker,
        }
    }

    /// Get the number of participants.
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Run the complete tournament.
    pub fn run(&mut self, card_db: &CardDatabase) -> TournamentResult {
        let start_time = Instant::now();

        println!();
        println!("{}", "=".repeat(60));
        println!("SWISS TOURNAMENT");
        println!("{}", "=".repeat(60));
        println!("Participants: {}", self.participants.len());
        println!("Rounds: {}", self.config.rounds);
        println!("Games per match: {}", self.config.games_per_match);
        println!("Bot type: {}", self.config.bot_type.name());
        println!("Base seed: {}", self.config.seed);
        if let Some(faction) = self.config.faction_filter {
            println!("Faction filter: {:?}", faction);
        }
        println!("{}", "=".repeat(60));

        for round_num in 1..=self.config.rounds {
            self.run_round(card_db, round_num);
            self.print_standings_after_round(card_db, round_num);
        }

        let total_duration = start_time.elapsed();

        // Save ELO ratings
        if let Err(e) = self.elo_tracker.save() {
            eprintln!("Warning: Failed to save ELO ratings: {}", e);
        }

        TournamentResult {
            timestamp: chrono::Utc::now().to_rfc3339(),
            participants: self.participants.len(),
            rounds: self.config.rounds,
            games_per_match: self.config.games_per_match,
            bot_type: self.config.bot_type.name().to_string(),
            seed: self.config.seed,
            total_duration_secs: total_duration.as_secs_f64(),
            standings: self.compute_standings(card_db),
        }
    }

    /// Generate pairings for the current round using Swiss system.
    fn generate_pairings(&self) -> Vec<Pairing> {
        let mut pairings = Vec::new();
        let mut paired: HashSet<String> = HashSet::new();

        // Sort participants by score (descending), then by Buchholz, then by win rate
        let mut sorted_ids = self.participant_order.clone();
        sorted_ids.sort_by(|a, b| {
            let pa = &self.participants[a];
            let pb = &self.participants[b];

            // Primary: score descending
            pb.score
                .cmp(&pa.score)
                // Secondary: Buchholz descending
                .then_with(|| self.buchholz_score(b).cmp(&self.buchholz_score(a)))
                // Tertiary: total win rate descending
                .then_with(|| {
                    let wr_a = self.total_win_rate(a);
                    let wr_b = self.total_win_rate(b);
                    wr_b.partial_cmp(&wr_a).unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        // Handle bye if odd number of participants
        if sorted_ids.len() % 2 == 1 {
            // Find lowest-ranked player who hasn't had a bye
            let mut bye_player = None;
            for id in sorted_ids.iter().rev() {
                if !self.bye_recipients.contains(id) {
                    bye_player = Some(id.clone());
                    break;
                }
            }
            // If all have had byes, give to lowest ranked
            let bye_player = bye_player.unwrap_or_else(|| sorted_ids.last().unwrap().clone());
            pairings.push(Pairing::bye(bye_player.clone()));
            paired.insert(bye_player);
        }

        // Pair remaining players
        for id in &sorted_ids {
            if paired.contains(id) {
                continue;
            }

            let participant = &self.participants[id];

            // Find best available opponent (similar score, not previously faced)
            let mut found = false;
            for opponent_id in &sorted_ids {
                if opponent_id == id || paired.contains(opponent_id) {
                    continue;
                }

                // Prefer opponents not previously faced
                if !participant.has_played(opponent_id) {
                    pairings.push(Pairing::new(id.clone(), opponent_id.clone()));
                    paired.insert(id.clone());
                    paired.insert(opponent_id.clone());
                    found = true;
                    break;
                }
            }

            // If no unfaced opponent, pair with any available
            if !found && !paired.contains(id) {
                for opponent_id in &sorted_ids {
                    if opponent_id != id && !paired.contains(opponent_id) {
                        pairings.push(Pairing::new(id.clone(), opponent_id.clone()));
                        paired.insert(id.clone());
                        paired.insert(opponent_id.clone());
                        break;
                    }
                }
            }
        }

        pairings
    }

    /// Calculate Buchholz score (sum of opponents' scores).
    fn buchholz_score(&self, deck_id: &str) -> u32 {
        self.participants
            .get(deck_id)
            .map(|p| {
                p.opponents
                    .iter()
                    .map(|opp_id| self.participants.get(opp_id).map(|opp| opp.score).unwrap_or(0))
                    .sum()
            })
            .unwrap_or(0)
    }

    /// Calculate total win rate across all matches.
    fn total_win_rate(&self, deck_id: &str) -> f64 {
        self.participants
            .get(deck_id)
            .map(|p| {
                let total_games: usize = p.match_history.values().map(|o| o.games).sum();
                let total_wins: usize = p.match_history.values().map(|o| o.wins).sum();
                if total_games == 0 {
                    0.5
                } else {
                    total_wins as f64 / total_games as f64
                }
            })
            .unwrap_or(0.5)
    }

    /// Run a single round.
    fn run_round(&mut self, card_db: &CardDatabase, round_num: usize) {
        println!();
        println!("{}", "-".repeat(50));
        println!("ROUND {}", round_num);
        println!("{}", "-".repeat(50));
        println!();

        let pairings = self.generate_pairings();
        let mut round_result = RoundResult::new(round_num, pairings.clone());
        let round_start = Instant::now();

        for pairing in &pairings {
            if pairing.is_bye() {
                // Award bye (equivalent to a match win)
                let deck_name = &self.participants[&pairing.player1_id].deck.name;
                println!("  {} receives BYE (+3 pts)", deck_name);

                if let Some(participant) = self.participants.get_mut(&pairing.player1_id) {
                    participant.score += 3;
                }
                self.bye_recipients.insert(pairing.player1_id.clone());
            } else {
                let player2_id = pairing.player2_id.as_ref().unwrap();
                let deck1 = self.participants[&pairing.player1_id].deck.clone();
                let deck2 = self.participants[player2_id].deck.clone();

                print!("  {} vs {} ... ", deck1.name, deck2.name);

                // Run the match
                let match_seed = self
                    .config
                    .seed
                    .wrapping_add(round_num as u64 * 1_000_000);
                let match_config = MatchConfig::new(
                    self.config.bot_type.clone(),
                    self.config.bot_type.clone(),
                    deck1.clone(),
                    deck2.clone(),
                    self.config.games_per_match,
                    match_seed,
                )
                .with_weights1(self.config.weights.clone())
                .with_weights2(self.config.weights.clone())
                .with_mcts_config(self.config.mcts_config.clone())
                .with_alphabeta_config(self.config.alphabeta_config.clone())
                .with_progress(false); // Don't show per-match progress

                let stats = match run_match_parallel(card_db, &match_config) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error running match: {}", e);
                        continue;
                    }
                };

                // Record results
                let outcome1 = MatchOutcome::from_stats(&stats, true);
                let outcome2 = outcome1.inverse();

                let result_str = match outcome1.result {
                    TournamentMatchResult::Win => {
                        format!("{} wins ({}-{}-{})", deck1.name, outcome1.wins, outcome1.losses, outcome1.draws)
                    }
                    TournamentMatchResult::Loss => {
                        format!("{} wins ({}-{}-{})", deck2.name, outcome2.wins, outcome2.losses, outcome2.draws)
                    }
                    TournamentMatchResult::Draw => {
                        format!("Draw ({}-{}-{})", outcome1.wins, outcome1.losses, outcome1.draws)
                    }
                };
                println!("{}", result_str);

                // Update ELO ratings
                let (delta1, delta2) =
                    self.elo_tracker
                        .update_match(&pairing.player1_id, player2_id, &stats.overall);
                if delta1 != 0 || delta2 != 0 {
                    println!(
                        "    ELO: {} {:+}, {} {:+}",
                        pairing.player1_id, delta1, player2_id, delta2
                    );
                }

                // Update participants
                if let Some(p1) = self.participants.get_mut(&pairing.player1_id) {
                    p1.record_match(player2_id, outcome1);
                }
                if let Some(p2) = self.participants.get_mut(player2_id) {
                    p2.record_match(&pairing.player1_id, outcome2);
                }

                round_result.results.insert(pairing.player1_id.clone(), stats);
            }
        }

        round_result.duration = round_start.elapsed();
        self.rounds.push(round_result);
    }

    /// Compute final standings with all tiebreakers.
    fn compute_standings(&self, card_db: &CardDatabase) -> Vec<Standing> {
        let mut standings: Vec<Standing> = self
            .participants
            .values()
            .map(|p| {
                let total_games: usize = p.match_history.values().map(|o| o.games).sum();
                let games_won: usize = p.match_history.values().map(|o| o.wins).sum();
                let games_lost: usize = p.match_history.values().map(|o| o.losses).sum();
                let games_drawn: usize = p.match_history.values().map(|o| o.draws).sum();

                let match_wins = p
                    .match_history
                    .values()
                    .filter(|o| o.result == TournamentMatchResult::Win)
                    .count();
                let match_losses = p
                    .match_history
                    .values()
                    .filter(|o| o.result == TournamentMatchResult::Loss)
                    .count();
                let match_draws = p
                    .match_history
                    .values()
                    .filter(|o| o.result == TournamentMatchResult::Draw)
                    .count();

                let commander_name = card_db
                    .get_commander(CardId(p.deck.commander))
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                let faction = p
                    .deck
                    .faction()
                    .map(|f| format!("{:?}", f))
                    .unwrap_or_else(|| "Unknown".to_string());

                Standing {
                    rank: 0, // Set after sorting
                    deck_id: p.deck_id.clone(),
                    deck_name: p.deck.name.clone(),
                    commander_name,
                    faction,
                    score: p.score,
                    buchholz: self.buchholz_score(&p.deck_id),
                    win_rate: if total_games > 0 {
                        games_won as f64 / total_games as f64
                    } else {
                        0.5
                    },
                    games_won,
                    games_lost,
                    games_drawn,
                    match_wins,
                    match_losses,
                    match_draws,
                }
            })
            .collect();

        // Sort by: score desc, buchholz desc, win rate desc
        standings.sort_by(|a, b| {
            b.score.cmp(&a.score).then_with(|| b.buchholz.cmp(&a.buchholz)).then_with(|| {
                b.win_rate
                    .partial_cmp(&a.win_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        // Assign ranks
        for (i, standing) in standings.iter_mut().enumerate() {
            standing.rank = i + 1;
        }

        standings
    }

    /// Print standings after a round.
    fn print_standings_after_round(&self, card_db: &CardDatabase, round_num: usize) {
        let standings = self.compute_standings(card_db);

        println!();
        println!("Standings after Round {}:", round_num);
        println!("{}", "─".repeat(90));
        println!(
            "{:>3} │ {:<22} │ {:<18} │ {:>5} │ {:>5} │ {:>7} │ {:>7}",
            "#", "Deck", "Commander", "Score", "Buch.", "W-L-D", "Win%"
        );
        println!("{}", "─".repeat(90));

        for standing in &standings {
            let faction_marker = match standing.faction.as_str() {
                "Argentum" => "A",
                "Symbiote" => "S",
                "Obsidion" => "O",
                "Neutral" => "N",
                _ => "-",
            };

            let wld = format!(
                "{}-{}-{}",
                standing.match_wins, standing.match_losses, standing.match_draws
            );

            println!(
                "{:>3} │ {:<20} {} │ {:<18} │ {:>5} │ {:>5} │ {:>7} │ {:>6.1}%",
                standing.rank,
                truncate(&standing.deck_name, 20),
                faction_marker,
                truncate(&standing.commander_name, 18),
                standing.score,
                standing.buchholz,
                wld,
                standing.win_rate * 100.0
            );
        }
        println!("{}", "─".repeat(90));
    }
}

/// Truncate string to max length with ellipsis.
fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(2)).collect();
        format!("{}..", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommended_rounds() {
        assert_eq!(SwissConfig::recommended_rounds(1), 0);
        assert_eq!(SwissConfig::recommended_rounds(2), 2); // log2(2) + 1 = 2
        assert_eq!(SwissConfig::recommended_rounds(4), 3); // log2(4) + 1 = 3
        assert_eq!(SwissConfig::recommended_rounds(8), 4); // log2(8) + 1 = 4
        assert_eq!(SwissConfig::recommended_rounds(12), 5); // ceil(log2(12)) + 1 = 5
    }

    #[test]
    fn test_match_outcome_inverse() {
        let outcome = MatchOutcome {
            result: TournamentMatchResult::Win,
            wins: 7,
            losses: 3,
            draws: 0,
            games: 10,
            win_rate: 0.7,
        };

        let inverse = outcome.inverse();
        assert_eq!(inverse.result, TournamentMatchResult::Loss);
        assert_eq!(inverse.wins, 3);
        assert_eq!(inverse.losses, 7);
        assert_eq!(inverse.games, 10);
    }

    #[test]
    fn test_participant_record_match() {
        let deck = DeckDefinition {
            id: "test_deck".to_string(),
            name: "Test Deck".to_string(),
            description: "Test deck for unit tests".to_string(),
            commander: 1000,
            cards: vec![],
            tags: vec![],
            playstyle: "midrange".to_string(),
        };

        let mut participant = Participant::new(deck);
        assert_eq!(participant.score, 0);

        // Record a win
        let outcome = MatchOutcome {
            result: TournamentMatchResult::Win,
            wins: 7,
            losses: 3,
            draws: 0,
            games: 10,
            win_rate: 0.7,
        };
        participant.record_match("opponent1", outcome);
        assert_eq!(participant.score, 3); // Win = 3 points
        assert!(participant.has_played("opponent1"));

        // Record a draw
        let outcome2 = MatchOutcome {
            result: TournamentMatchResult::Draw,
            wins: 5,
            losses: 5,
            draws: 0,
            games: 10,
            win_rate: 0.5,
        };
        participant.record_match("opponent2", outcome2);
        assert_eq!(participant.score, 4); // 3 + 1 = 4 points

        // Record a loss
        let outcome3 = MatchOutcome {
            result: TournamentMatchResult::Loss,
            wins: 3,
            losses: 7,
            draws: 0,
            games: 10,
            win_rate: 0.3,
        };
        participant.record_match("opponent3", outcome3);
        assert_eq!(participant.score, 4); // No points for loss
    }
}
