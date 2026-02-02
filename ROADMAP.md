# Essence Wars Roadmap

**Last Updated:** 2026-02-02
**Version:** 0.8.0 (Commander Edition)

## Current State Summary

| Area | Status | Notes |
|------|--------|-------|
| Core Engine | Stable | 759 tests passing, deterministic |
| Performance | **Regressed** | 2-5x slower than v0.7 targets |
| MCP Server | 80% complete | Missing explain tools, win rate |
| Python Gym | Code ready | Missing CI/CD for PyPI publishing |
| Tutorial | Broken | Random bot, dark overlay, seed mismatch |
| UI (Tauri) | Functional | Deck selection wizard complete |

### Performance Baseline (v0.8.0)

| Benchmark | Current | Target | Gap |
|-----------|---------|--------|-----|
| `greedy_game` | 1,193 µs | 230 µs | 5.2x slower |
| `engine_fork` | 851 ns | 245 ns | 3.5x slower |
| `random_game` | 100 µs | 30 µs | 3.3x slower |
| `GameState` size | 1,136 bytes | <800 bytes | 42% over |
| `Creature` size | 72 bytes | <40 bytes | 80% over |

**Root cause:** 48 bytes wasted per creature on token fields (`Option<Vec<TokenAbility>>` + `Option<String>`) that 95%+ of creatures never use.

---

## Sprint Plan

### Sprint 1: Foundation (Current)

| Task | Priority | Status | Notes |
|------|----------|--------|-------|
| Performance: Token data separation | P0 | Not started | Reduce Creature 72→28 bytes |
| Performance: Slot-indexed storage | P0 | Not started | O(1) creature lookup |
| Win Conditions: Option B Hybrid | P1 | Not started | Tactical Stability + Essence Extraction |
| Essential Audit: Phase 0 hygiene | P2 | Not started | `cargo udeps`, `autoflake` |
| Test: Determinism "100 Run Challenge" | P2 | Not started | Verify replay stability |

**Goal:** Recover v0.7 performance levels, implement thematic win condition reframing.

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

---

## Key Decisions Made

### Win Conditions: Option B (Hybrid)

**Decision:** Keep life as "Tactical Stability" (zero = forced retreat), add Essence Extraction as parallel win condition.

**Rationale:**
- Minimal code changes (reframe `total_damage_dealt` as essence extracted)
- Keeps healing cards and Lifesteal keyword viable
- Lore-consistent flavor
- Does not break API or observation space

**Numbers TBD:** Exact thresholds for extraction victory to be determined during implementation.

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
| Phase 0 | Dead/unused code | Sprint 1 | Quick hygiene sweep |
| Phase 1 | Determinism | Sprint 1 | Add "100 Run Challenge" test |
| Phase 2 | Safety (unwrap/unsafe) | Low priority | Only 1 unwrap in core engine |
| Phase 3 | Serialization | Sprint 1 | Add round-trip test |
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

**Phase 1 (Sprint 1):**
1. Token data separation - Move `Option<Vec<TokenAbility>>` + `Option<String>` to enum/external map
2. Slot-indexed creature storage - `[Option<Creature>; 5]` instead of `Vec<Creature>`
3. Remove redundant `owner`/`slot` fields from Creature

**Phase 2 (Future):**
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
