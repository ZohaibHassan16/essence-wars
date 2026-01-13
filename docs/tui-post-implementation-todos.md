# TUI Post-Implementation Investigation Items

These issues were discovered during TUI Phase 3 implementation and should be addressed after all TUI phases are complete.

## 1. MctsBot::select_action() Fallback Heuristic

**File:** `src/bots/mcts.rs`

The generic `Bot::select_action()` trait method for MctsBot falls back to a simple heuristic instead of performing actual MCTS search:

```rust
fn select_action(
    &mut self,
    _state_tensor: &[f32; STATE_TENSOR_SIZE],
    _legal_mask: &[f32; 256],
    legal_actions: &[Action],
) -> Action {
    // MCTS requires engine access for simulation
    // Fall back to simple heuristic without engine
    ...
}
```

**Problem:** Code that uses `Box<dyn Bot>` and calls the generic `select_action()` method will get the fallback heuristic instead of actual MCTS search. Only code that uses the concrete `MctsBot` type and calls `select_action_with_engine()` gets real MCTS.

**Investigation:** Consider whether the `Bot` trait should be redesigned to support engine-based bots, or if documentation should be clearer about when to use `select_action_with_engine()`.

---

## 2. CardDatabase::load_from_directory() Path Confusion

**File:** `src/core/cards.rs`

The `load_from_directory()` function internally appends `/sets` to the provided path:

```rust
pub fn load_from_directory<P: AsRef<Path>>(path: P) -> Result<Self, CardLoadError> {
    let sets_path = path.as_ref().join("sets");
    ...
}
```

**Problem:** Calling `load_from_directory("data/cards/sets")` results in looking for `"data/cards/sets/sets"` which doesn't exist, causing 0 cards to be loaded silently (no error, just empty database).

**Investigation:** Consider either:
- Removing the internal `/sets` join and requiring callers to provide the full path
- Adding validation/warning when 0 cards are loaded
- Better documenting the expected path structure
