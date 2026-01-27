# 🔴 Critical Refactoring Candidates

## **1. effect_queue.rs (1,339 lines) - WORST OFFENDER**

**Issues:**
- **God Object Pattern**: Single file with 26+ methods handling ALL effect types
- **Massive private methods**: Each `apply_*` method (damage, heal, buff, destroy, summon, transform, copy, etc.) is 50-150+ lines
- **Duplicate target resolution**: Nearly identical match blocks repeated in `apply_damage`, `apply_heal`, `apply_buff`, `apply_grant_keyword` for resolving `AllCreatures`, `AllAllyCreatures`, `AllEnemyCreatures`
- **Deep coupling**: Directly mutates GameState in 20+ different ways
- **No abstraction**: Effect application logic is procedural, not using visitor/strategy patterns

**Refactoring Strategy** (non-breaking):
```rust
// Extract effect handlers into separate modules
core/engine/effects/
  ├── damage_handler.rs     // DamageEffect, HealEffect
  ├── stat_handler.rs       // BuffEffect, SetStatsEffect
  ├── summon_handler.rs     // SummonEffect, TransformEffect, CopyEffect
  ├── keyword_handler.rs    // GrantKeywordEffect, RemoveKeywordEffect, SilenceEffect
  ├── utility_handler.rs    // DrawEffect, BounceEffect, RefreshEffect
  └── target_resolver.rs    // Centralized EffectTarget → Vec<(Owner, Slot)> logic
```

**Benefit**: Each handler is 100-200 lines instead of 1,339 in one file. Target resolution DRY principle.

---

## **2. game_engine.rs (1,120 lines) - SECOND WORST**

**Issues:**
- **Fat interface**: 11 public methods + tons of private helpers all in one impl block
- **Mixed concerns**: Game initialization, turn flow, action execution, and victory checking all together
- **Duplicate tracer logic**: `execute_attack()` and `execute_attack_with_tracers()` nearly identical
- **Effect queue creation scattered**: Every action method creates its own `EffectQueue::new()`

**Refactoring Strategy** (non-breaking):
```rust
// Split into focused components
impl GameEngine {
    // Keep initialization & state access
    pub fn new(...) -> Self
    pub fn start_game(...) 
    pub fn state(&self) -> &GameState
}

// Extract turn management
struct TurnManager<'a> {
    engine: &'a mut GameEngine,
}
impl TurnManager {
    fn start_turn(&mut self)
    fn end_turn(&mut self)
    fn process_regenerate_healing(&mut self, player: PlayerId)
}

// Extract action executor
struct ActionExecutor<'a> {
    engine: &'a mut GameEngine,
    combat_tracer: Option<&'a mut CombatTracer>,
    effect_tracer: Option<&'a mut EffectTracer>,
}
impl ActionExecutor {
    fn execute_play_card(&mut self, ...)
    fn execute_attack(&mut self, ...)
    fn execute_use_ability(&mut self, ...)
}
```

**Benefit**: Each component 300-400 lines, single responsibility.

---

## **3. tracing.rs (1,050 lines) - THIRD WORST**

**Issues:**
- **Two unrelated systems**: CombatTracer (lines 1-600) + EffectTracer (lines 600-1050) in same file
- **Massive log methods**: `log_quick_check`, `log_shield_check`, `log_lethal_check`, etc. - 15+ near-duplicate methods
- **Builder pattern abuse**: Every CombatStep has `.with_keywords()`, `.with_damage()`, `.with_death()` chained

**Refactoring Strategy** (non-breaking):
```rust
// Split into separate files
core/tracing/
  ├── combat_tracer.rs      // CombatTracer, CombatTrace, CombatStep, CombatPhase
  └── effect_tracer.rs      // EffectTracer, EffectTrace, EffectStep

// Simplify CombatStep creation
impl CombatStep {
    pub fn damage(phase: CombatPhase, desc: String, amount: u8) -> Self
    pub fn death(phase: CombatPhase, desc: String, victim: String) -> Self
    pub fn healing(phase: CombatPhase, desc: String, amount: u8) -> Self
}
```

**Benefit**: 500 lines per file, clearer separation of concerns.

---

## **4. combat.rs (932 lines)**

**Issues:**
- **One giant function**: `resolve_combat()` is 400+ lines with nested conditionals
- **Keyword spaghetti**: Quick/Shield/Ranged/Piercing/Lethal/Lifesteal logic interleaved
- **Duplicate tracer calls**: Every keyword has `if let Some(t) = tracer { t.log_X() }`

**Refactoring Strategy** (non-breaking):
```rust
// Extract keyword handlers as separate functions
fn apply_quick_strike(...) -> (damage_dealt, defender_died)
fn apply_shield_absorption(...) -> damage_after_shield
fn apply_lethal_check(...) -> target_died
fn apply_piercing_overflow(...) -> face_damage
fn apply_lifesteal_healing(...) -> healed_amount

// Main combat flow becomes declarative
pub fn resolve_combat(...) -> CombatResult {
    let ctx = CombatContext::new(state, attacker, defender);
    
    if ctx.has_quick() {
        ctx.resolve_quick_strike();
    }
    if ctx.has_shield() {
        ctx.apply_shield();
    }
    ctx.resolve_main_damage();
    // ...
}
```

**Benefit**: 100-150 lines per keyword handler, testable in isolation.

---

## Summary & Priority

| File | Lines | Methods | Issue | Priority |
|------|-------|---------|-------|----------|
| effect_queue.rs | 1,339 | 26 | God object, duplicate target resolution | 🔴 **HIGHEST** |
| game_engine.rs | 1,120 | 11 | Mixed concerns, fat interface | 🔴 **HIGH** |
| tracing.rs | 1,050 | 50+ | Two unrelated systems in one file | 🟡 MEDIUM |
| combat.rs | 932 | 1 | One 400+ line function with nested logic | 🟡 MEDIUM |

**Recommendation**: Start with effect_queue.rs - it has the most duplication and would benefit most from the Extract Module refactoring pattern. The target resolution alone appears 10+ times identically.
