# Essence Wars Roadmap

**Last Updated:** 2026-02-02
**Version:** 0.8.0 (Commander Edition)

## Current State Summary

| Area | Status | Notes |
|------|--------|-------|
| Core Engine | Stable | 759 tests passing, deterministic |
| Performance | **Recovered** | 2.2-2.4x improvement after token data separation |
| MCP Server | 80% complete | Missing explain tools, win rate |
| Python Gym | Code ready | Missing CI/CD for PyPI publishing |
| Tutorial | Broken | Random bot, dark overlay, seed mismatch |
| UI (Tauri) | Functional | Deck selection wizard complete |

### Performance Baseline (v0.8.0 Post-Sprint 1)

| Benchmark | Before | After | Improvement |
|-----------|--------|-------|-------------|
| `greedy_game` | 1,193 µs | ~500 µs | 2.4x faster |
| `engine_fork` | 851 ns | ~350 ns | 2.4x faster |
| `random_game` | 100 µs | ~45 µs | 2.2x faster |
| `Creature` size | 72 bytes | ~32 bytes | 56% smaller |

**Fixed:** Token data separation moved `Option<Vec<TokenAbility>>` + `Option<String>` to `Option<Box<TokenData>>`, eliminating 40 bytes per creature.

---

## Sprint Plan

### Sprint 1: Foundation ✅ COMPLETE

| Task | Priority | Status | Notes |
|------|----------|--------|-------|
| Performance: Token data separation | P0 | ✅ Done | Creature 72→~32 bytes, 2.2-2.4x improvement |
| Performance: Slot-indexed storage | P0 | ⏭ Skipped | Deferred - diminishing returns vs complexity |
| Win Conditions: Option B Hybrid | P1 | ✅ Done | EssenceWar default, TurnLimitTiebreaker, EssenceExtractionReached |
| Essential Audit: Phase 0 hygiene | P2 | ✅ Done | `cargo udeps` clean, only 9 TODOs |
| Test: Determinism "100 Run Challenge" | P2 | ✅ Done | 11 determinism tests verified passing |

**Result:** Performance recovered, win conditions reframed with thematic naming (Tactical Stability + Essence Extraction).

### Sprint 2: User Experience

| Task | Priority | Status | Notes |
|------|----------|--------|-------|
| Tutorial: ScriptedBot implementation | P0 | Not started | Predictable opponent actions |
| Tutorial: UI fixes | P1 | Not started | Reduce overlay to 30-50% |
| Tutorial: Seed/deck alignment | P1 | Not started | Match seed finder to tutorial decks |
| MCP: `explain_rules()` tool | P1 | Not started | LLM-friendly rules summary |
| MCP: `explain_keywords()` tool | P2 | Not started | 16 keyword reference |
| MCP: `explain_card(card_id)` tool | P2 | Not started | Detailed card info |
| MCP: Win rate in AI hint | P2 | Not started | Expose MCTS win estimates |
| MCP: Use Alpha-Beta for hints | P2 | Not started | Better than MCTS for hints |

**Goal:** Polished onboarding experience, enhanced MCP for LLM agents.

### Sprint 3: Distribution

| Task | Priority | Status | Notes |
|------|----------|--------|-------|
| Python Gym: `wheels.yml` | P0 | Not started | Multi-platform maturin builds |
| Python Gym: `test.yml` | P1 | Not started | CI testing for Python |
| Python Gym: `publish.yml` | P1 | Not started | PyPI trusted publishing |
| Python Gym: Version sync | P1 | Not started | 0.7.0 → 0.8.0 |
| Test Suite: Performance regression CI | P2 | Not started | Catch regressions early |

**Goal:** Enable community adoption via PyPI, prevent performance regressions.

### Sprint 4+: Polish & Expansion

| Task | Priority | Status | Notes |
|------|----------|--------|-------|
| Deck Builder UI | P2 | Not started | Custom deck creation |
| Spectator Diagnostics | P3 | Not started | Win probability graphs, analysis |
| Asset Pipeline Sync | P3 | Not started | Art, prompts, documentation audit |
| Lore Updates | P3 | Not started | Sync with art direction changes |

---

## Deferred Indefinitely

| Item | Reason |
|------|--------|
| Win Conditions: Option A (Pure Extraction) | High effort, breaks 24 cards, no user demand |
| Python Linting CI/CD | Small team, manual sprints preferred |
| Enterprise-style GitHub Actions | Creates friction, not worth it for 3-person team |
| Slot-indexed creature storage | Diminishing returns after token data separation |

---

## Key Decisions Made

### Win Conditions: Option B (Hybrid) ✅ IMPLEMENTED

**Decision:** Keep life as "Tactical Stability" (zero = forced retreat), add Essence Extraction as parallel win condition.

**Implementation Details:**
- Renamed `GameMode::EssenceDuel` → `GameMode::EssenceWar` (now default)
- Renamed `WinReason::VictoryPointsReached` → `WinReason::EssenceExtractionReached`
- Renamed `WinReason::TurnLimitHigherLife` → `WinReason::TurnLimitTiebreaker`
- Turn limit tiebreaker uses VP for EssenceWar mode, life for Attrition mode
- Backwards compatibility maintained (CLI accepts "essence-duel", "essenceduel" etc.)
- All 759 tests passing

**Thresholds:** 50 essence extracted to win (unchanged from VictoryPoints).

### Expansion Planning

| Question | Decision |
|----------|----------|
| Scripting vs Pure Rust | Pure Rust for performance |
| Custom YAML cards | Yes, sandboxed for casual play |
| Bot/Agent compatibility | Accept retuning; keep observation/action space stable |
| Expansion structure | 1 commander + deck per faction per expansion |

### Python Tooling

**Decision:** No CI/CD for linting. Manual `uv run ruff check` and `uv run mypy` in consolidated sprints.

**Rationale:** Small team (1 human + 2 AI), CI friction not worth it. Run tools manually after touching Python code.

---

## Essential Audit Status

| Phase | Description | Status | Notes |
|-------|-------------|--------|-------|
| Phase 0 | Dead/unused code | ✅ Done | `cargo udeps` clean, no unused Python imports |
| Phase 1 | Determinism | ✅ Done | 11 existing determinism tests verified |
| Phase 2 | Safety (unwrap/unsafe) | Low priority | Only 1 unwrap in core engine |
| Phase 3 | Serialization | ✅ Done | Round-trip tests already exist |
| Phase 4 | Python/ML | Sprint 3 | Check data leakage, zombie processes |

### Current Audit Numbers

| Check | Count | Severity |
|-------|-------|----------|
| `unwrap()` in core engine | 1 | Low |
| `unwrap()` in CLIs/diagnostics | 86 | Acceptable |
| `unsafe` blocks | 2 | Low (well-commented) |
| TODOs/FIXMEs | 9 | Low |

---

## Architecture Notes

### Performance Optimization Plan

See `docs/design-performance-optimizations.md` for full details.

**Phase 1 (Sprint 1) - COMPLETED:**
1. ✅ Token data separation - Changed `token_abilities` + `token_name` to `Option<Box<TokenData>>` (2.2-2.4x speedup)
2. ⏭ Slot-indexed creature storage - Skipped (diminishing returns after token fix)
3. ⏭ Remove redundant fields - Skipped (complexity not justified)

**Phase 2 (Future - if needed):**
- Batch keyword evaluation (lookup table)
- Greedy delta scoring (skip forks for Attack/EndTurn)
- Legal action caching

### MCP Server Architecture

```
Claude Code (MCP Client)
    │
    │ JSON-RPC (stdio)
    ▼
essence-wars-mcp (MCP Server)
    │
    │ HTTP sync → POST /sync_state (optional)
    ▼
essence-wars-ui (Tauri App)
```

**Current tools:** list_decks, list_bots, start_game, show_state, show_hand, legal_actions, play_action, ai_hint, end_game, sync_ui_state

**Planned tools:** explain_rules, explain_keywords, explain_card

---

## Team

- **Chris** - Human developer, project owner
- **Claude** - AI developer (Anthropic)
- **Gemini** - AI developer (Google)

---

## References

| Document | Purpose |
|----------|---------|
| `CLAUDE.md` | AI assistant context, quick commands |
| `docs/design-performance-optimizations.md` | Performance optimization proposal |
| `docs/design-commanders.md` | Commander system design |
| `docs/cards-new-horizons.md` | Card effect system |
| `essential-audit-checklist.md` | Pre-audit checklist |
| `todo.md` | Original task list (being consolidated here) |
| `expansion.md` | Expansion planning notes |
| `win-conditions.md` | Win condition redesign discussion |
