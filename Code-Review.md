# Codebase Review: Essence Wars Engine

## 1. Executive Summary

The **Essence Wars** codebase is a robust, feature-complete implementation of the game design specifications. The core engine faithfully reproduces the rules, mechanics, and data structures outlined in the design documents. The AI infrastructure, including MCTS and the tuning pipeline, is fully integrated.

However, the project suffers from a "flat structure" organizational issue in the `src/` directory, which mixes core engine logic, AI implementations, and utility modules. Refactoring this into a clearer hierarchy is the primary recommendation.

## 2. Feature Completeness Matrix

| Component | Status | Notes |
|-----------|--------|-------|
| **Core Types** | ✅ Complete | `PlayerId`, `Slot`, `CardId`, `Keywords` (Bitfield) implemented. |
| **Game State** | ✅ Complete | `GameState`, `PlayerState`, `Creature` structs match design. |
| **Card System** | ✅ Complete | YAML loading, `CardDatabase`, and `CardDefinition` fully functional. |
| **Action System** | ✅ Complete | `Action` enum, indexing (0-255), and masking implemented. |
| **Legal Generation**| ✅ Complete | Validates Mana, AP, Slots, Guard, Exhaustion, Silenced. |
| **Engine Logic** | ✅ Complete | Turn structure, Draw, Mana/AP reset, deterministic shuffling. |
| **Combat Logic** | ✅ Complete | Lane adjacency, Guard enforcement, Keyword interactions (Quick, Shield, etc.). |
| **Effect System** | ✅ Complete | Queue-based resolution, Triggers (`OnPlay`, `OnDeath`, etc.), Buffs/Debuffs. |
| **AI Interface** | ✅ Complete | Tensor conversion (11.1 spec), `GameEnvironment` trait. |
| **Bots** | ✅ Complete | MCTS (UCB1), Greedy (Heuristic), Random bots implemented. |
| **Tuning** | ✅ Complete | CMA-ES optimizer and Evaluator infrastructure present. |

## 3. Implementation vs. Design Verification

### 3.1. Combat Mechanics
The implementation in `src/combat.rs` correctly handles the complex keyword interactions specified in `@docs/design-engine.md`:
*   **Quick:** Correctly implements initiative (Attacker strikes first if Quick vs Non-Quick).
*   **Shield:** Correctly absorbs the first instance of damage and removes the keyword.
*   **Piercing:** Correctly calculates overflow damage to face when the defender dies.
*   **Lethal:** Correctly sets health to 0 if any >0 damage is dealt.
*   **Lifesteal:** Correctly heals the owner for damage dealt (clamped to max life).

### 3.2. Legal Moves & Guard
The `legal.rs` module correctly enforces:
*   **Guard:** Attacks must target Guard creatures if present in adjacent lanes.
*   **Ranged:** Bypasses lane adjacency but still respects Guard priority (must attack a Guard if any exists).
*   **Summoning Sickness:** Creatures cannot attack on turn played unless they have **Rush**.

### 3.3. AI & Tensor Representation
The `src/tensor.rs` module implements the exact 326-float specification:
*   Global state (6 floats).
*   Player states (80 floats each).
*   Card Embeddings (IDs) filling the remainder.
*   Normalization (Life/30, Essence/10) is applied correctly.

## 4. Architectural Observations

### 4.1. The "Flat Structure" Issue
The `src/` directory currently contains 17 files and 4 directories at the root level. This mixes concerns:
*   **Core Engine:** `types.rs`, `state.rs`, `engine.rs`, `combat.rs`, `effects.rs`, `legal.rs`, `actions.rs`, `cards.rs`, `keywords.rs`.
*   **AI/ML:** `tensor.rs`.
*   **Configuration:** `config.rs`.
*   **Entry Points:** `lib.rs`.

**Recommendation:** Move core engine files into a `src/engine/` or `src/core/` submodule.

### 4.2. Effect Queue System
The `EffectQueue` in `src/engine.rs` is a strong architectural choice. It prevents recursion depth issues and ensures predictable resolution order for complex chains of triggered abilities (e.g., OnDamage -> OnDeath -> OnAllyDeath).

### 4.3. MCTS & Greedy Integration
The integration of `GreedyBot` as a rollout policy for `MctsBot` (`src/bots/mcts.rs`) is implemented exactly as described in `@docs/mcts-tuning-workflow.md`. This allows the MCTS to leverage the tuned weights for better simulation accuracy.

## 5. Refactoring Recommendations

To address the organizational chaos, I recommend the following restructuring:

```text
src/
├── core/                 # NEW: Core engine logic
│   ├── mod.rs
│   ├── types.rs
│   ├── state.rs
│   ├── cards.rs
│   ├── actions.rs
│   ├── legal.rs
│   ├── engine.rs         # The GameEngine struct
│   ├── combat.rs
│   ├── effects.rs
│   └── keywords.rs
├── ai/                   # NEW: AI specific code
│   ├── mod.rs
│   ├── tensor.rs
│   ├── bots/             # Moved from src/bots
│   └── tuning/           # Moved from src/tuning
├── arena/                # Existing arena runners
└── lib.rs                # Re-exports for external use
```

## 6. Conclusion

The Essence Wars codebase is **production-ready** in terms of logic and feature set. The implementation is rigorous and adheres strictly to the design. The only significant work remaining is organizational refactoring to improve maintainability and navigation.
