# Performance Optimization Proposal

**Document Version:** 1.0
**Date:** 2026-02-02
**Author:** Claude (AI Assistant)
**Status:** Draft for Review

---

## Executive Summary

This document proposes a series of performance optimizations for the Essence Wars game engine. The primary goal is to reduce the `greedy_game` benchmark from ~580µs to <300µs, bringing performance closer to the v0.7 baseline (~230µs) while retaining all Commander Edition features.

Recent profiling identified state cloning as the primary bottleneck, with `GameState` currently at 1,136 bytes. The proposed optimizations focus on reducing struct sizes, eliminating redundant allocations, and improving hot-path efficiency.

**Expected Outcomes:**
- 30-50% reduction in `greedy_game` benchmark time
- 20-40% reduction in `engine_fork` time
- Smaller memory footprint for MCTS tree operations

---

## Current Performance Baseline

### Benchmark Results (v0.8.0 with Commander Edition)

| Benchmark | Current | v0.7 Target | Gap |
|-----------|---------|-------------|-----|
| `greedy_game` | 580 µs | 230 µs | 2.5x slower |
| `engine_fork` | 510 ns | 245 ns | 2.1x slower |
| `random_game` | 40 µs | 30 µs | 1.3x slower |
| `state_cloning` | 470-510 ns | ~200 ns | 2.3x slower |

### Struct Size Analysis

| Struct | Current Size | Notes |
|--------|--------------|-------|
| `Creature` | 72 bytes | 48 bytes wasted on token fields for regular creatures |
| `PlayerState` | 552 bytes | Contains ArrayVec of Creatures |
| `GameState` | 1,136 bytes | Full state that gets cloned in MCTS |
| `ResolvedCommanderPassive` | 8 bytes | Recently added cache |

### Profiled Hot Paths

1. **State Cloning** (MCTS fork operations) - ~40% of MCTS time
2. **Legal Action Generation** - ~15% of turn processing
3. **Greedy Evaluation** - ~25% of bot decision time (mostly cloning)
4. **Creature Lookup** - O(n) linear search, called frequently

---

## Proposed Optimizations

### Phase 1: Struct Layout Optimizations (High Impact, Low Risk)

#### 1.1 Slot-Indexed Creature Storage

**Current Implementation:**
```rust
pub struct PlayerState {
    pub creatures: ArrayVec<Creature, 5>,  // Linear search by slot
    // ...
}

pub fn get_creature(&self, slot: Slot) -> Option<&Creature> {
    self.creatures.iter().find(|c| c.slot == slot)  // O(n) lookup
}
```

**Proposed Implementation:**
```rust
pub struct PlayerState {
    pub creatures: [Option<Creature>; 5],  // Direct indexing by slot
    // ...
}

#[inline]
pub fn get_creature(&self, slot: Slot) -> Option<&Creature> {
    self.creatures[slot.0 as usize].as_ref()  // O(1) lookup
}
```

**Benefits:**
- O(1) creature lookup instead of O(n) linear search
- Removes `slot` field from `Creature` struct (-1 byte)
- Simpler, more predictable memory layout
- Eliminates iterator allocation in hot paths

**Tradeoffs:**
- Fixed memory allocation (all 5 slots always allocated)
- Slightly more complex iteration when traversing all creatures
- Breaking change to `creatures` field access patterns

**Migration Path:**
1. Add `get_creature_by_index()` helper methods
2. Update all direct `creatures` field accesses
3. Switch internal representation
4. Remove `slot` field from `Creature`

---

#### 1.2 Token Data Separation

**Current Implementation:**
```rust
pub struct Creature {
    // ... core fields (24 bytes) ...
    pub token_abilities: Option<Vec<TokenAbility>>,  // 24 bytes
    pub token_name: Option<String>,                  // 24 bytes
}
// Total: 72 bytes per creature
```

**Problem:** Regular creatures (95%+ of all creatures) never use `token_abilities` or `token_name`, but still pay the 48-byte overhead.

**Proposed Implementation (Option A - Enum):**
```rust
pub struct Creature {
    // ... core fields ...
    pub source: CreatureSource,
}

pub enum CreatureSource {
    Card(CardId),                                        // 4 bytes
    Token { name: String, abilities: Vec<TokenAbility> }, // Only when needed
}
```

**Proposed Implementation (Option B - External Map):**
```rust
pub struct Creature {
    // ... core fields ...
    pub card_id: CardId,  // CardId(0) sentinel for tokens
}

pub struct GameState {
    // ...
    pub token_data: HashMap<CreatureInstanceId, TokenData>,  // Rarely used
}
```

**Benefits:**
- Regular creatures reduced from 72 to ~28 bytes (-61%)
- Significantly faster cloning for typical game states
- Tokens remain fully functional

**Tradeoffs:**
- Option A: Enum overhead, pattern matching required
- Option B: Indirection for token access, HashMap allocation

**Recommendation:** Option A (enum) for simplicity and cache locality.

---

#### 1.3 Remove Redundant Fields

**Owner Field:**
```rust
// Current: owner stored in Creature
pub struct Creature {
    pub owner: PlayerId,  // 1 byte - but implied by which player's list it's in
}

// Proposed: owner passed as parameter when needed
pub fn process_creature(owner: PlayerId, creature: &Creature) { ... }
```

**Slot Field (if using indexed storage):**
```rust
// Current: slot stored in Creature
pub struct Creature {
    pub slot: Slot,  // 1 byte - but implied by array index
}

// Proposed: slot derived from array index
let slot = Slot(index as u8);
```

**Combined Savings:** 2 bytes per creature

---

### Phase 2: Algorithm Optimizations (Medium Impact, Medium Risk)

#### 2.1 Batch Keyword Evaluation

**Current Implementation:**
```rust
// 16 conditional branches
if kw.has_guard() { score += w.keyword_guard; }
if kw.has_lethal() { score += w.keyword_lethal; }
if kw.has_lifesteal() { score += w.keyword_lifesteal; }
// ... 13 more ...
```

**Proposed Implementation:**
```rust
impl GreedyWeights {
    /// Precomputed keyword weights indexed by bit position
    pub fn keyword_weight_table(&self) -> [f32; 16] {
        [
            self.keyword_rush,      // bit 0
            self.keyword_ranged,    // bit 1
            self.keyword_piercing,  // bit 2
            // ...
        ]
    }
}

// Single loop with table lookup
let weights = self.weights.keyword_weight_table();
let mut bits = kw.0;
while bits != 0 {
    let bit_pos = bits.trailing_zeros() as usize;
    score += weights[bit_pos];
    bits &= bits - 1;  // Clear lowest set bit
}
```

**Benefits:**
- Reduces branch mispredictions
- Better instruction cache utilization
- Scales better as keywords are added

---

#### 2.2 Greedy Evaluation Delta Scoring

**Current Implementation:**
```rust
pub fn evaluate_action(&self, engine: &GameEngine, action: Action) -> f32 {
    let mut sim_engine = engine.fork();           // ~500ns clone
    sim_engine.apply_action(action)?;             // Apply action
    self.evaluate_state(&sim_engine.state, player) // Evaluate result
}
```

**Proposed Implementation (for simple actions):**
```rust
pub fn evaluate_action(&self, engine: &GameEngine, action: Action) -> f32 {
    match action {
        Action::EndTurn => self.evaluate_end_turn_delta(engine),
        Action::Attack { attacker_slot, target } => {
            self.evaluate_attack_delta(engine, attacker_slot, target)
        }
        // Complex actions still use full simulation
        _ => self.evaluate_with_simulation(engine, action),
    }
}

fn evaluate_attack_delta(&self, engine: &GameEngine, attacker: Slot, target: Target) -> f32 {
    let base_score = self.evaluate_state(&engine.state, engine.current_player());

    // Calculate expected outcome without cloning
    let attacker = engine.state.get_creature(engine.current_player(), attacker)?;
    let defender = match target { ... };

    let damage_dealt = calculate_damage(attacker, defender);
    let damage_taken = calculate_counterattack(attacker, defender);

    // Compute score delta
    base_score + (damage_dealt as f32 * self.weights.enemy_creature_health)
               - (damage_taken as f32 * self.weights.own_creature_health)
    // ... handle deaths, keywords, etc.
}
```

**Benefits:**
- Avoids ~500ns fork for Attack and EndTurn actions
- Attack actions are ~30-40% of all actions
- Could improve `greedy_game` by 15-25%

**Tradeoffs:**
- More complex code to maintain
- Must stay in sync with actual game rules
- Edge cases (triggered effects) still need full simulation

---

#### 2.3 Legal Action Caching

**Current:** Legal actions are regenerated on every query.

**Proposed:**
```rust
pub struct GameState {
    // ...
    /// Cached legal actions (invalidated on state change)
    #[serde(skip)]
    legal_actions_cache: Option<ArrayVec<Action, 64>>,
    /// State hash for cache validation
    #[serde(skip)]
    state_hash: u64,
}

pub fn legal_actions(&mut self, card_db: &CardDatabase) -> &[Action] {
    let current_hash = self.compute_hash();
    if self.state_hash != current_hash || self.legal_actions_cache.is_none() {
        self.legal_actions_cache = Some(compute_legal_actions(self, card_db));
        self.state_hash = current_hash;
    }
    self.legal_actions_cache.as_ref().unwrap()
}
```

**Benefits:**
- Avoids redundant computation in MCTS
- Useful when same state is queried multiple times

**Tradeoffs:**
- Cache invalidation complexity
- Memory overhead for cache storage
- Hash computation cost

---

### Phase 3: Memory Optimizations (Lower Priority)

#### 3.1 Avoid Vec Allocation in Tie-Breaking

**Current:**
```rust
let mut best_actions: Vec<Action> = Vec::new();  // Heap allocation
```

**Proposed:**
```rust
let mut best_actions: ArrayVec<Action, 8> = ArrayVec::new();  // Stack allocation
```

**Savings:** Eliminates heap allocation per action selection.

---

#### 3.2 MCTS Node Pool

**Current:** Each MCTS node is individually heap-allocated.

**Proposed:** Use arena allocation or object pool for MCTS nodes.

```rust
pub struct MctsArena {
    nodes: Vec<MctsNode>,
    free_list: Vec<usize>,
}

impl MctsArena {
    pub fn allocate(&mut self) -> NodeId {
        if let Some(idx) = self.free_list.pop() {
            NodeId(idx)
        } else {
            self.nodes.push(MctsNode::default());
            NodeId(self.nodes.len() - 1)
        }
    }
}
```

**Benefits:**
- Reduces allocation overhead
- Better cache locality
- Predictable memory usage

---

## Implementation Plan

### Sprint 1: Struct Optimizations (1-2 weeks)

| Task | Priority | Effort | Risk |
|------|----------|--------|------|
| 1.1 Slot-indexed creature storage | P0 | Medium | Low |
| 1.2 Token data separation | P0 | Medium | Medium |
| 1.3 Remove owner/slot fields | P1 | Low | Low |
| Update all tests | P0 | Medium | Low |

**Success Criteria:**
- `Creature` struct reduced to <40 bytes
- `GameState` reduced to <800 bytes
- All 759 tests passing
- `engine_fork` benchmark <350ns

### Sprint 2: Algorithm Optimizations (1-2 weeks)

| Task | Priority | Effort | Risk |
|------|----------|--------|------|
| 2.1 Batch keyword evaluation | P1 | Low | Low |
| 2.2 Delta scoring (Attack only) | P1 | Medium | Medium |
| 2.3 Legal action caching | P2 | Medium | Medium |

**Success Criteria:**
- `greedy_game` benchmark <400µs
- No regression in bot win rates
- MCTS simulation throughput improved by 20%

### Sprint 3: Memory Optimizations (Optional)

| Task | Priority | Effort | Risk |
|------|----------|--------|------|
| 3.1 ArrayVec for tie-breaking | P2 | Low | Low |
| 3.2 MCTS node pooling | P3 | High | Medium |

---

## Risk Assessment

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking serialization compatibility | Medium | High | Version replay files, add migration |
| Subtle game logic bugs | Medium | High | Comprehensive test coverage, golden tests |
| Cache invalidation bugs | Medium | Medium | Extensive testing, conservative invalidation |
| Performance regression in edge cases | Low | Medium | Benchmark diverse game states |

### Compatibility Considerations

1. **Replay Files:** Serialization format will change. Old replays may need migration.
2. **Python Bindings:** Struct layout changes require PyO3 binding updates.
3. **UI Integration:** DTO layer should insulate UI from internal changes.
4. **MCP Server:** Should be unaffected (uses DTOs).

---

## Success Metrics

### Primary Metrics

| Metric | Current | Target | Stretch |
|--------|---------|--------|---------|
| `greedy_game` | 580 µs | <400 µs | <300 µs |
| `engine_fork` | 510 ns | <350 ns | <280 ns |
| `GameState` size | 1,136 bytes | <800 bytes | <600 bytes |
| `Creature` size | 72 bytes | <40 bytes | <32 bytes |

### Secondary Metrics

- MCTS simulations per second: >10% improvement
- Memory usage during MCTS: >20% reduction
- No regression in bot win rates (within statistical noise)

---

## Appendix A: Struct Size Breakdown

### Current Creature (72 bytes)

```
instance_id:      4 bytes  (CreatureInstanceId)
card_id:          2 bytes  (CardId)
owner:            1 byte   (PlayerId) <- can remove
slot:             1 byte   (Slot) <- can remove with indexed storage
attack:           1 byte   (i8)
current_health:   1 byte   (i8)
max_health:       1 byte   (i8)
base_attack:      1 byte   (u8)
base_health:      1 byte   (u8)
keywords:         2 bytes  (Keywords)
status:           1 byte   (CreatureStatus)
turn_played:      2 bytes  (u16)
frenzy_stacks:    1 byte   (u8)
padding:          ~4 bytes (alignment)
token_abilities: 24 bytes  (Option<Vec<TokenAbility>>) <- move to enum
token_name:      24 bytes  (Option<String>) <- move to enum
```

### Proposed Creature (~26 bytes)

```
instance_id:      4 bytes  (CreatureInstanceId)
card_id:          2 bytes  (CardId)
attack:           1 byte   (i8)
current_health:   1 byte   (i8)
max_health:       1 byte   (i8)
base_attack:      1 byte   (u8)
base_health:      1 byte   (u8)
keywords:         2 bytes  (Keywords)
status:           1 byte   (CreatureStatus)
turn_played:      2 bytes  (u16)
frenzy_stacks:    1 byte   (u8)
source:           8 bytes  (CreatureSource enum discriminant + CardId)
padding:          ~1 byte
```

---

## Appendix B: Related Documents

- `docs/design-commanders.md` - Commander system design
- `docs/bots-tuning-pipeline.md` - Bot weight tuning guide
- `CLAUDE.md` - Project context and commands

---

## Open Questions

1. Should we maintain backward compatibility with v0.8.0 replay files?
2. Is the delta scoring optimization worth the maintenance burden?
3. Should we consider a more aggressive refactor (e.g., ECS architecture)?

---

*This document is a proposal for discussion. Feedback welcome.*
