//! Transposition Table for Alpha-Beta search.
//!
//! Uses Zobrist hashing to efficiently detect when the same position has been
//! reached via different move orders, avoiding redundant search.
//!
//! # Architecture
//! - `ZobristHasher`: Generates incremental hash values for game states
//! - `TranspositionTable`: Caches search results with replacement strategy
//!
//! # Hash Components
//! For each position, we hash:
//! - Creatures: card_id, slot, attack, health, keywords, exhausted status
//! - Supports: card_id, slot, durability
//! - Resources: life, essence, action_points per player
//! - Active player and turn number

use std::sync::OnceLock;

use crate::actions::Action;
use crate::core::state::GameState;

/// Number of random values for Zobrist hashing.
/// Layout:
/// - [0..5000]: Creature card IDs (per slot, per player) = 2 * 5 * 500 potential cards
/// - [5000..5100]: Creature attack values (-50 to +50) per slot = 10 * 10
/// - [5100..5200]: Creature health values (0 to 100) per slot = 10 * 10
/// - [5200..5232]: Creature keywords (16 bits) per slot = 10 * 16 / 5
/// - [5232..5242]: Creature exhausted status per slot = 10
/// - [5242..5442]: Support card IDs per slot = 2 * 2 * 50
/// - [5442..5450]: Support durability per slot = 2 * 4
/// - [5450..5512]: Life values (0-30) per player = 2 * 31
/// - [5512..5532]: Essence values (0-10) per player = 2 * 10
/// - [5532..5540]: Action points (0-4) per player = 2 * 4
/// - [5540..5542]: Active player = 2
/// - [5542..5602]: Turn number (0-60) = 60
const ZOBRIST_TABLE_SIZE: usize = 6000;

/// Pre-computed random values for Zobrist hashing.
static ZOBRIST_TABLE: OnceLock<[u64; ZOBRIST_TABLE_SIZE]> = OnceLock::new();

fn get_zobrist_table() -> &'static [u64; ZOBRIST_TABLE_SIZE] {
    ZOBRIST_TABLE.get_or_init(|| {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut table = [0u64; ZOBRIST_TABLE_SIZE];

        // Use a deterministic seed for reproducibility
        let seed: u64 = 0xDEADBEEF_CAFEBABE;

        for (i, entry) in table.iter_mut().enumerate() {
            let mut hasher = DefaultHasher::new();
            seed.hash(&mut hasher);
            i.hash(&mut hasher);
            *entry = hasher.finish();
        }

        table
    })
}

/// Compute Zobrist hash for a game state.
///
/// This is a fast incremental hash that XORs together pre-computed random
/// values based on game features. The result is a 64-bit hash that uniquely
/// identifies the position with high probability.
pub fn zobrist_hash(state: &GameState) -> u64 {
    let table = get_zobrist_table();
    let mut hash: u64 = 0;

    // Hash creatures for both players
    for (player_idx, player_state) in state.players.iter().enumerate() {
        let player_offset = player_idx * 2500; // Half of creature range per player

        for creature in &player_state.creatures {
            let slot = creature.slot.0 as usize;
            let slot_offset = player_offset + slot * 500;

            // Card ID (mod 500 to fit in range)
            let card_idx = (creature.card_id.0 as usize) % 500;
            hash ^= table[slot_offset + card_idx];

            // Attack (offset by 50 to handle negatives)
            let attack_idx = 5000 + player_idx * 5 + slot;
            let attack_val = (creature.attack as i32 + 50).clamp(0, 99) as usize;
            hash ^= table[attack_idx].wrapping_add(attack_val as u64);

            // Health
            let health_idx = 5100 + player_idx * 5 + slot;
            let health_val = creature.current_health.clamp(0, 99) as usize;
            hash ^= table[health_idx].wrapping_add(health_val as u64);

            // Keywords (as bits)
            let kw_idx = 5200 + player_idx * 5 + slot;
            hash ^= table[kw_idx].wrapping_add(creature.keywords.0 as u64);

            // Exhausted status
            if creature.status.is_exhausted() {
                hash ^= table[5232 + player_idx * 5 + slot];
            }
        }

        // Hash supports
        for support in &player_state.supports {
            let slot = support.slot.0 as usize;
            let support_offset = 5242 + player_idx * 100 + slot * 50;
            let card_idx = (support.card_id.0 as usize) % 50;
            hash ^= table[support_offset + card_idx];

            // Durability
            let dur_idx = 5442 + player_idx * 4 + slot;
            hash ^= table[dur_idx].wrapping_add(support.current_durability as u64);
        }

        // Hash resources
        // Life (0-30)
        let life_idx = 5450 + player_idx * 31;
        let life_val = player_state.life.clamp(0, 30) as usize;
        hash ^= table[life_idx + life_val];

        // Essence (0-10)
        let essence_idx = 5512 + player_idx * 10;
        let essence_val = player_state.current_essence.min(9) as usize;
        hash ^= table[essence_idx + essence_val];

        // Action points (0-4)
        let ap_idx = 5532 + player_idx * 4;
        let ap_val = player_state.action_points.min(3) as usize;
        hash ^= table[ap_idx + ap_val];
    }

    // Active player
    hash ^= table[5540 + state.active_player.index()];

    // Turn number (mod 60)
    let turn_idx = 5542 + (state.current_turn as usize % 60);
    hash ^= table[turn_idx];

    hash
}

/// Type of bound stored in transposition table entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    /// Exact score (PV node)
    Exact,
    /// Lower bound (fail-high / beta cutoff)
    Lower,
    /// Upper bound (fail-low / alpha cutoff)
    Upper,
}

/// Entry in the transposition table.
#[derive(Clone, Debug)]
#[allow(dead_code)] // best_move stored for future move ordering enhancement
pub struct TTEntry {
    /// Zobrist hash of the position (for collision detection)
    pub hash: u64,
    /// Search depth at which this entry was computed
    pub depth: u32,
    /// Evaluation score
    pub score: f32,
    /// Type of bound
    pub bound: Bound,
    /// Best move found (for move ordering)
    pub best_move: Option<Action>,
    /// Age (generation) for replacement strategy
    pub age: u8,
}

impl TTEntry {
    /// Create a new entry.
    #[allow(dead_code)]
    pub fn new(hash: u64, depth: u32, score: f32, bound: Bound, best_move: Option<Action>) -> Self {
        Self {
            hash,
            depth,
            score,
            bound,
            best_move,
            age: 0,
        }
    }
}

/// Transposition table with configurable size.
///
/// Uses a simple replacement strategy: replace if new entry has greater depth
/// or if the existing entry is from an older search.
pub struct TranspositionTable {
    /// Table entries (power of 2 size for fast modulo)
    entries: Vec<Option<TTEntry>>,
    /// Mask for index calculation (size - 1)
    mask: usize,
    /// Current generation/age for replacement
    generation: u8,
    /// Statistics
    pub hits: u64,
    pub misses: u64,
    pub collisions: u64,
}

impl TranspositionTable {
    /// Create a new transposition table with the given size (in entries).
    /// Size will be rounded up to the next power of 2.
    pub fn new(size_hint: usize) -> Self {
        let size = size_hint.next_power_of_two().max(1024);
        Self {
            entries: vec![None; size],
            mask: size - 1,
            generation: 0,
            hits: 0,
            misses: 0,
            collisions: 0,
        }
    }

    /// Create a table with approximately the given memory budget (in MB).
    pub fn with_memory_mb(mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let entries = (mb * 1024 * 1024) / entry_size;
        Self::new(entries)
    }

    /// Increment the generation counter (call at start of each new search).
    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = None;
        }
        self.hits = 0;
        self.misses = 0;
        self.collisions = 0;
    }

    /// Probe the table for an entry.
    pub fn probe(&mut self, hash: u64) -> Option<&TTEntry> {
        let index = (hash as usize) & self.mask;

        if let Some(ref entry) = self.entries[index] {
            if entry.hash == hash {
                self.hits += 1;
                return Some(entry);
            } else {
                self.collisions += 1;
            }
        }

        self.misses += 1;
        None
    }

    /// Store an entry in the table.
    pub fn store(&mut self, hash: u64, depth: u32, score: f32, bound: Bound, best_move: Option<Action>) {
        let index = (hash as usize) & self.mask;

        let should_replace = match &self.entries[index] {
            None => true,
            Some(existing) => {
                // Replace if:
                // 1. Different position (collision)
                // 2. New entry has greater depth
                // 3. Existing entry is from older generation
                existing.hash != hash
                    || depth >= existing.depth
                    || existing.age != self.generation
            }
        };

        if should_replace {
            self.entries[index] = Some(TTEntry {
                hash,
                depth,
                score,
                bound,
                best_move,
                age: self.generation,
            });
        }
    }

    /// Get fill rate (percentage of entries used).
    #[allow(dead_code)]
    pub fn fill_rate(&self) -> f64 {
        let used = self.entries.iter().filter(|e| e.is_some()).count();
        (used as f64 / self.entries.len() as f64) * 100.0
    }

    /// Get hit rate (percentage of probes that found an entry).
    #[allow(dead_code)]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        // Default: 16MB table (~200k entries)
        Self::with_memory_mb(16)
    }
}

// =============================================================================
// MCTS Transposition Table
// =============================================================================

/// Entry in the MCTS transposition table.
///
/// Unlike Alpha-Beta's TT (which stores bounds), MCTS TT stores value estimates
/// from previous rollouts. These can be used to:
/// 1. Skip rollouts for positions we've seen many times
/// 2. Initialize node values from cached estimates
#[derive(Clone, Debug)]
pub struct MctsTranspositionEntry {
    /// Zobrist hash of the position (for collision detection)
    pub hash: u64,
    /// Number of visits to this position across the search
    pub visits: u32,
    /// Sum of rewards from rollouts starting from this position
    pub total_reward: f32,
    /// Age (generation) for replacement strategy
    pub age: u8,
}

impl MctsTranspositionEntry {
    /// Create a new entry.
    pub fn new(hash: u64) -> Self {
        Self {
            hash,
            visits: 0,
            total_reward: 0.0,
            age: 0,
        }
    }

    /// Get the average value estimate for this position.
    /// Returns 0.5 (neutral) if no visits.
    pub fn average_value(&self) -> f32 {
        if self.visits == 0 {
            0.5
        } else {
            self.total_reward / self.visits as f32
        }
    }

    /// Update with a new rollout result.
    pub fn update(&mut self, win: bool) {
        self.visits += 1;
        self.total_reward += if win { 1.0 } else { 0.0 };
    }
}

/// MCTS-specific transposition table.
///
/// Caches position evaluations across the search to avoid redundant rollouts.
/// Uses a simple replacement strategy based on visit count and age.
pub struct MctsTranspositionTable {
    /// Table entries (power of 2 size for fast modulo)
    entries: Vec<Option<MctsTranspositionEntry>>,
    /// Mask for index calculation (size - 1)
    mask: usize,
    /// Current generation/age for replacement
    generation: u8,
    /// Statistics
    pub hits: u64,
    pub misses: u64,
}

impl MctsTranspositionTable {
    /// Create a new MCTS transposition table with the given size (in entries).
    /// Size will be rounded up to the next power of 2.
    pub fn new(size_hint: usize) -> Self {
        let size = size_hint.next_power_of_two().max(1024);
        Self {
            entries: vec![None; size],
            mask: size - 1,
            generation: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Create a table with approximately the given memory budget (in MB).
    pub fn with_memory_mb(mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<MctsTranspositionEntry>>();
        let entries = (mb * 1024 * 1024) / entry_size;
        Self::new(entries)
    }

    /// Increment the generation counter (call at start of each new search).
    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = None;
        }
        self.hits = 0;
        self.misses = 0;
    }

    /// Probe the table for an entry.
    /// Returns the entry if found with matching hash.
    pub fn probe(&mut self, hash: u64) -> Option<&MctsTranspositionEntry> {
        let index = (hash as usize) & self.mask;

        if let Some(ref entry) = self.entries[index] {
            if entry.hash == hash {
                self.hits += 1;
                return Some(entry);
            }
        }

        self.misses += 1;
        None
    }

    /// Update the table with a rollout result.
    /// Creates a new entry if none exists, or updates the existing one.
    pub fn update(&mut self, hash: u64, win: bool) {
        let index = (hash as usize) & self.mask;

        match &mut self.entries[index] {
            Some(entry) if entry.hash == hash => {
                // Update existing entry
                entry.update(win);
                entry.age = self.generation;
            }
            slot => {
                // Replace with new entry (either empty or collision)
                let mut entry = MctsTranspositionEntry::new(hash);
                entry.update(win);
                entry.age = self.generation;
                *slot = Some(entry);
            }
        }
    }

    /// Get hit rate (percentage of probes that found an entry).
    #[allow(dead_code)]
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

impl Default for MctsTranspositionTable {
    fn default() -> Self {
        // Default: 8MB table for MCTS (smaller than AB since entries are simpler)
        Self::with_memory_mb(8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zobrist_table_initialized() {
        let table = get_zobrist_table();
        // Check that values are non-zero and varied
        assert!(table[0] != 0);
        assert!(table[0] != table[1]);
        assert!(table[100] != table[101]);
    }

    #[test]
    fn test_transposition_table_basic() {
        let mut tt = TranspositionTable::new(1024);

        // Store an entry
        tt.store(12345, 5, 1.5, Bound::Exact, None);

        // Probe should find it
        let entry = tt.probe(12345);
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.score, 1.5);
        assert_eq!(entry.bound, Bound::Exact);

        // Different hash should miss
        assert!(tt.probe(99999).is_none());
    }

    #[test]
    fn test_replacement_strategy() {
        let mut tt = TranspositionTable::new(1024);

        // Store entry at depth 3
        tt.store(12345, 3, 1.0, Bound::Exact, None);

        // Try to replace with depth 2 (should not replace)
        tt.store(12345, 2, 2.0, Bound::Exact, None);
        let entry = tt.probe(12345).unwrap();
        assert_eq!(entry.depth, 3);
        assert_eq!(entry.score, 1.0);

        // Replace with depth 5 (should replace)
        tt.store(12345, 5, 3.0, Bound::Exact, None);
        let entry = tt.probe(12345).unwrap();
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.score, 3.0);
    }
}
