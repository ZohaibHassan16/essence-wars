//! Matchup generation for balance testing.
//!
//! Provides utilities for generating all possible deck matchups at various
//! granularities (all decks, faction pairs, filtered subsets).

use crate::decks::{DeckDefinition, DeckRegistry, Faction};

use super::matchup::UnifiedMatchup;

/// Builder for generating matchups at various granularities.
///
/// # Example
///
/// ```ignore
/// let builder = MatchupBuilder::new(&deck_registry);
///
/// // Generate all 66 unique matchups (12 decks, no mirrors)
/// let all_matchups = builder.build_all_matchups();
///
/// // Generate only Argentum vs Symbiote matchups
/// let faction_matchups = builder.build_faction_matchups(Faction::Argentum, Faction::Symbiote);
///
/// // Filter to specific matchups
/// let filtered = MatchupBuilder::filter_matchups(all_matchups, "broodmother");
/// ```
pub struct MatchupBuilder<'a> {
    registry: &'a DeckRegistry,
}

impl<'a> MatchupBuilder<'a> {
    /// Create a new matchup builder for the given deck registry.
    pub fn new(registry: &'a DeckRegistry) -> Self {
        Self { registry }
    }

    /// Build all deck-vs-deck matchups (excluding same-deck mirrors).
    ///
    /// With 12 decks, this produces 12×11/2 = 66 unique matchups.
    ///
    /// Matchups are ordered consistently:
    /// - By faction pair (Argentum-Symbiote, Argentum-Obsidion, Symbiote-Obsidion)
    /// - Then by deck within faction (alphabetically by deck ID)
    pub fn build_all_matchups(&self) -> Vec<UnifiedMatchup> {
        let all_decks: Vec<&DeckDefinition> = self.registry.decks().collect();
        let mut matchups = Vec::new();

        for (i, deck1) in all_decks.iter().enumerate() {
            for deck2 in all_decks.iter().skip(i + 1) {
                // Skip same-deck mirrors
                if deck1.id == deck2.id {
                    continue;
                }
                matchups.push(UnifiedMatchup::new((*deck1).clone(), (*deck2).clone()));
            }
        }

        matchups
    }

    /// Build matchups for a specific faction pair.
    ///
    /// If the factions are the same, generates intra-faction matchups
    /// (excluding mirrors).
    pub fn build_faction_matchups(&self, f1: Faction, f2: Faction) -> Vec<UnifiedMatchup> {
        let decks1: Vec<_> = self.registry.decks_for_faction(f1);
        let decks2: Vec<_> = self.registry.decks_for_faction(f2);

        let mut matchups = Vec::new();

        if f1 == f2 {
            // Intra-faction: avoid duplicates and mirrors
            for (i, deck1) in decks1.iter().enumerate() {
                for deck2 in decks1.iter().skip(i + 1) {
                    if deck1.id != deck2.id {
                        matchups.push(UnifiedMatchup::new((*deck1).clone(), (*deck2).clone()));
                    }
                }
            }
        } else {
            // Inter-faction: all combinations
            for deck1 in &decks1 {
                for deck2 in &decks2 {
                    matchups.push(UnifiedMatchup::new((*deck1).clone(), (*deck2).clone()));
                }
            }
        }

        matchups
    }

    /// Build all inter-faction matchups (excludes same-faction matchups).
    ///
    /// This is useful for focusing on cross-faction balance without
    /// intra-faction matchups.
    pub fn build_inter_faction_matchups(&self) -> Vec<UnifiedMatchup> {
        let factions = Faction::all_factions();
        let mut matchups = Vec::new();

        for (i, &f1) in factions.iter().enumerate() {
            for &f2 in factions.iter().skip(i + 1) {
                matchups.extend(self.build_faction_matchups(f1, f2));
            }
        }

        matchups
    }

    /// Build all intra-faction matchups (same-faction only, excluding mirrors).
    ///
    /// With 4 decks per faction, produces 4×3/2 = 6 matchups per faction,
    /// for a total of 18 intra-faction matchups.
    pub fn build_intra_faction_matchups(&self) -> Vec<UnifiedMatchup> {
        let mut matchups = Vec::new();

        for &faction in Faction::all_factions() {
            matchups.extend(self.build_faction_matchups(faction, faction));
        }

        matchups
    }

    /// Build all matchups where a specific deck is player 1.
    ///
    /// With 12 decks, this produces 11 matchups (the deck vs all others).
    /// Used for parallelizing benchmark jobs across decks.
    pub fn build_matchups_for_deck1(&self, deck_id: &str) -> Vec<UnifiedMatchup> {
        let all_decks: Vec<&DeckDefinition> = self.registry.decks().collect();
        let mut matchups = Vec::new();

        // Find the deck1
        let Some(deck1) = all_decks.iter().find(|d| d.id == deck_id) else {
            return matchups;
        };

        // Create matchups against all other decks
        for deck2 in &all_decks {
            if deck2.id != deck1.id {
                matchups.push(UnifiedMatchup::new((*deck1).clone(), (*deck2).clone()));
            }
        }

        matchups
    }

    /// Filter matchups by pattern.
    ///
    /// Supports matching by:
    /// - Deck ID (e.g., "broodmother" matches "broodmother_pack")
    /// - Faction pair (e.g., "argentum-symbiote" or "symbiote-argentum")
    /// - Single faction (e.g., "argentum" matches all matchups involving Argentum)
    ///
    /// Pattern matching is case-insensitive.
    pub fn filter_matchups(matchups: Vec<UnifiedMatchup>, pattern: &str) -> Vec<UnifiedMatchup> {
        let pattern_lower = pattern.to_lowercase();

        matchups
            .into_iter()
            .filter(|m| {
                let id_lower = m.id.to_lowercase();
                let f1 = m.faction1_or_neutral().as_tag();
                let f2 = m.faction2_or_neutral().as_tag();

                // Match by deck ID (partial match)
                if id_lower.contains(&pattern_lower) {
                    return true;
                }

                // Match by faction pair (either order)
                let faction_pair = format!("{}-{}", f1, f2);
                let faction_pair_rev = format!("{}-{}", f2, f1);
                if faction_pair.contains(&pattern_lower)
                    || faction_pair_rev.contains(&pattern_lower)
                {
                    return true;
                }

                // Match by single faction
                if f1 == pattern_lower || f2 == pattern_lower {
                    return true;
                }

                false
            })
            .collect()
    }

    /// Get a summary of matchup counts.
    pub fn matchup_summary(&self) -> MatchupSummary {
        let all = self.build_all_matchups();
        let inter = self.build_inter_faction_matchups();
        let intra = self.build_intra_faction_matchups();

        MatchupSummary {
            total_decks: self.registry.len(),
            total_matchups: all.len(),
            inter_faction_matchups: inter.len(),
            intra_faction_matchups: intra.len(),
        }
    }
}

/// Summary of matchup counts for reporting.
#[derive(Debug, Clone)]
pub struct MatchupSummary {
    /// Total number of decks in the registry.
    pub total_decks: usize,
    /// Total unique matchups (excluding mirrors).
    pub total_matchups: usize,
    /// Matchups between different factions.
    pub inter_faction_matchups: usize,
    /// Matchups within the same faction.
    pub intra_faction_matchups: usize,
}

impl std::fmt::Display for MatchupSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} decks, {} matchups ({} inter-faction, {} intra-faction)",
            self.total_decks,
            self.total_matchups,
            self.inter_faction_matchups,
            self.intra_faction_matchups
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::GameData;

    fn load_test_data() -> GameData {
        GameData::load_default(true).expect("Failed to load test data")
    }

    #[test]
    fn test_build_all_matchups() {
        let data = load_test_data();
        let builder = MatchupBuilder::new(&data.deck_registry);

        let matchups = builder.build_all_matchups();

        // With 12 decks: 12 * 11 / 2 = 66 matchups
        assert_eq!(matchups.len(), 66, "Expected 66 unique matchups");

        // Verify no mirrors
        for matchup in &matchups {
            assert!(
                !matchup.is_mirror(),
                "Found mirror match: {}",
                matchup.id
            );
        }

        // Verify no duplicates
        let mut ids: Vec<_> = matchups.iter().map(|m| &m.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), matchups.len(), "Found duplicate matchups");
    }

    #[test]
    fn test_build_inter_faction_matchups() {
        let data = load_test_data();
        let builder = MatchupBuilder::new(&data.deck_registry);

        let matchups = builder.build_inter_faction_matchups();

        // 3 faction pairs × 4×4 decks = 48 matchups
        assert_eq!(matchups.len(), 48, "Expected 48 inter-faction matchups");

        // All matchups should be between different factions
        for matchup in &matchups {
            assert!(
                !matchup.is_same_faction(),
                "Found same-faction match in inter-faction: {}",
                matchup.id
            );
        }
    }

    #[test]
    fn test_build_intra_faction_matchups() {
        let data = load_test_data();
        let builder = MatchupBuilder::new(&data.deck_registry);

        let matchups = builder.build_intra_faction_matchups();

        // 3 factions × (4×3/2) = 3 × 6 = 18 matchups
        assert_eq!(matchups.len(), 18, "Expected 18 intra-faction matchups");

        // All matchups should be within same faction
        for matchup in &matchups {
            assert!(
                matchup.is_same_faction(),
                "Found cross-faction match in intra-faction: {}",
                matchup.id
            );
        }
    }

    #[test]
    fn test_build_faction_matchups() {
        let data = load_test_data();
        let builder = MatchupBuilder::new(&data.deck_registry);

        // Cross-faction: 4 × 4 = 16 matchups
        let cross = builder.build_faction_matchups(Faction::Argentum, Faction::Symbiote);
        assert_eq!(cross.len(), 16, "Expected 16 Argentum vs Symbiote matchups");

        // Same-faction: 4 × 3 / 2 = 6 matchups (no mirrors)
        let same = builder.build_faction_matchups(Faction::Argentum, Faction::Argentum);
        assert_eq!(same.len(), 6, "Expected 6 Argentum vs Argentum matchups");

        // Verify no mirrors in same-faction
        for matchup in &same {
            assert!(!matchup.is_mirror(), "Found mirror in same-faction matchups");
        }
    }

    #[test]
    fn test_filter_by_deck_id() {
        let data = load_test_data();
        let builder = MatchupBuilder::new(&data.deck_registry);
        let all = builder.build_all_matchups();

        // Filter by partial deck name
        let filtered = MatchupBuilder::filter_matchups(all.clone(), "broodmother");

        // Should find all matchups involving broodmother_pack
        assert!(!filtered.is_empty(), "Expected to find broodmother matchups");
        for matchup in &filtered {
            let has_brood = matchup.deck1.id.contains("broodmother")
                || matchup.deck2.id.contains("broodmother");
            assert!(has_brood, "Filtered matchup should contain broodmother");
        }
    }

    #[test]
    fn test_filter_by_faction_pair() {
        let data = load_test_data();
        let builder = MatchupBuilder::new(&data.deck_registry);
        let all = builder.build_all_matchups();

        // Filter by faction pair
        let filtered = MatchupBuilder::filter_matchups(all.clone(), "argentum-symbiote");

        // Should find exactly the cross-faction matchups
        assert_eq!(filtered.len(), 16, "Expected 16 Argentum-Symbiote matchups");

        for matchup in &filtered {
            let factions = (matchup.faction1(), matchup.faction2());
            let valid = factions == (Some(Faction::Argentum), Some(Faction::Symbiote))
                || factions == (Some(Faction::Symbiote), Some(Faction::Argentum));
            assert!(valid, "Filtered matchup should be Argentum vs Symbiote");
        }
    }

    #[test]
    fn test_filter_by_single_faction() {
        let data = load_test_data();
        let builder = MatchupBuilder::new(&data.deck_registry);
        let all = builder.build_all_matchups();

        // Filter by single faction
        let filtered = MatchupBuilder::filter_matchups(all.clone(), "argentum");

        // Should include all matchups involving Argentum
        // Argentum vs Argentum: 6 + Argentum vs others: 4×4×2 = 32, but we only count unique
        // Actually: Argentum vs Symbiote (16) + Argentum vs Obsidion (16) + Argentum vs Argentum (6) = 38
        assert!(!filtered.is_empty());
        for matchup in &filtered {
            let has_argentum = matchup.faction1() == Some(Faction::Argentum)
                || matchup.faction2() == Some(Faction::Argentum);
            assert!(has_argentum, "Filtered matchup should involve Argentum");
        }
    }

    #[test]
    fn test_matchup_summary() {
        let data = load_test_data();
        let builder = MatchupBuilder::new(&data.deck_registry);

        let summary = builder.matchup_summary();

        assert_eq!(summary.total_decks, 12);
        assert_eq!(summary.total_matchups, 66);
        assert_eq!(summary.inter_faction_matchups, 48);
        assert_eq!(summary.intra_faction_matchups, 18);
    }

    #[test]
    fn test_case_insensitive_filter() {
        let data = load_test_data();
        let builder = MatchupBuilder::new(&data.deck_registry);
        let all = builder.build_all_matchups();

        let filtered_lower = MatchupBuilder::filter_matchups(all.clone(), "argentum");
        let filtered_upper = MatchupBuilder::filter_matchups(all.clone(), "ARGENTUM");
        let filtered_mixed = MatchupBuilder::filter_matchups(all.clone(), "ArGenTuM");

        assert_eq!(filtered_lower.len(), filtered_upper.len());
        assert_eq!(filtered_lower.len(), filtered_mixed.len());
    }
}
