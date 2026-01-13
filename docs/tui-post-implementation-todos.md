# TUI Post-Implementation Investigation Items

These issues were discovered during TUI Phase 3 implementation and have been **RESOLVED**.

---

## 1. MctsBot::select_action() Fallback Heuristic ✅ FIXED

**File:** `src/bots/mcts.rs`

**Original Problem:** The generic `Bot::select_action()` trait method for MctsBot fell back to a simple heuristic instead of performing actual MCTS search. Code using `Box<dyn Bot>` would get degraded behavior silently.

**Resolution:**
1. Extended the `Bot` trait with `select_action_with_engine()` default method
2. Added `requires_engine()` method to the `Bot` trait
3. Updated `GameRunner` to use `select_action_with_engine()` for all bots
4. Made `MctsBot::select_action()` panic with a clear error message
5. Added tests for the new behavior

**New API:**
```rust
pub trait Bot: Send {
    fn select_action(&mut self, ...) -> Action;

    // New: Engine-aware selection for bots like MCTS
    fn select_action_with_engine(&mut self, engine: &GameEngine) -> Action {
        // Default: extract state from engine and call select_action()
    }

    // New: Indicates if bot requires engine for full functionality
    fn requires_engine(&self) -> bool { false }
}
```

---

## 2. CardDatabase::load_from_directory() Path Confusion ✅ FIXED

**File:** `src/core/cards.rs`

**Original Problem:** The function internally appended `/sets` to the path, causing confusion. Calling with `"data/cards/sets"` would look for `"data/cards/sets/sets"` which doesn't exist, returning an empty database silently.

**Resolution:**
1. Removed the implicit `/sets` join - callers now provide the full path
2. Added validation that returns an error if:
   - Directory doesn't exist
   - Path is not a directory
   - No cards are found (empty database)
3. Updated all callers to use `"data/cards/sets"` instead of `"data/cards"`
4. Added comprehensive tests for error cases

**New Behavior:**
```rust
// Correct usage - provide full path to directory containing YAML files
let db = CardDatabase::load_from_directory("data/cards/sets")?;

// Errors now returned instead of silent failures:
// - "Card directory does not exist: ..."
// - "Path is not a directory: ..."
// - "No cards found in directory: ..."
```
