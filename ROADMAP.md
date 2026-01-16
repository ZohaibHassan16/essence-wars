# Essence Wars Roadmap

> A deterministic card game engine for ML/AI research, and a living open-source game.

**Vision**: Establish Essence Wars as a benchmark platform for reinforcement learning research (PPO, AlphaZero, LLMs) while building a complete, community-driven digital card game.

**Platforms**: GitHub, Huggingface, essence-wars.ai (future)

---

## Phase 1: Foundation ✅

*Core engine and bot infrastructure.*

- [x] Build deterministic game engine in Rust
- [x] Implement GreedyBot and MctsBot
- [x] Create weight tuning pipeline (CMA-ES)
- [x] Add arena for bot matches with tracing
- [x] Set up CI/CD (GitHub Actions)
- [x] Establish test tiers (quick/medium/long/overnight)
- [x] Initial card pool (107 cards)

---

## Phase 2: New Horizons Edition 🔄

*Complete the first edition card set and streamline tooling.*

- [ ] Expand card pool to 300 cards (New Horizons Edition)
- [ ] Balance all faction matchups (45-55% win rates)
- [x] Set up Modal cloud tuning (internal tooling)
- [ ] Update documentation (`docs/essence-wars-design.md`, `docs/design-engine.md`)
- [ ] Create card database reference (`docs/cards-new-horizons.md`)
- [ ] Clean up and document tuning scripts
- [x] Remove TUI code (deferred indefinitely)

---

## Phase 3: ML Infrastructure

*Make Essence Wars accessible to ML researchers.*

- [ ] Create PyO3 Python bindings
- [ ] Implement Gymnasium environment interface
- [ ] Publish `essence-wars` package to PyPI (`pip install essence-wars`)
- [ ] Publish Rust crate to crates.io (`cargo add essence-wars`)
- [ ] Train first proof-of-concept PPO agent
- [ ] Validate Gym interface with standard RL libraries (Stable-Baselines3, CleanRL)
- [ ] Write researcher quickstart guide

---

## Phase 4: Research Platform

*Build the agent ecosystem and benchmarking infrastructure.*

- [ ] Implement AlphaZero agent
- [ ] Create Agent Roster (baseline agents: MCTS, PPO, AlphaZero)
- [ ] Build Elo leaderboard system for agent rankings
- [ ] Host trained models on Huggingface
- [ ] Create benchmark suite for reproducible comparisons
- [ ] Publish dataset of self-play games
- [ ] Write documentation for agent submission

---

## Phase 5: Living Game

*Transform from research tool to playable game with community.*

- [ ] Develop lore, worldbuilding, and faction identities
- [ ] Commission or create card artwork
- [ ] Add flavor text to all cards
- [ ] Build web client (Huggingface Spaces)
- [ ] Implement Human vs AI play mode
- [ ] Implement AI vs AI spectator mode
- [ ] Launch essence-wars.ai website
- [ ] Add community features (accounts, match history)

---

## Phase 6: Frontier Research

*Advanced research directions and ongoing development.*

- [ ] LLM agent integration (reasoning-based play)
- [ ] Transfer learning experiments (new cards without retraining)
- [ ] Deck-building agents (discover combos and strategies)
- [ ] First expansion set (post-New Horizons)
- [ ] Multi-agent research (team play, drafting)
- [ ] Gather human play data for offline RL research

---

## Principles

1. **Sequential focus**: Complete one phase before moving to the next
2. **Research-first**: Every feature should serve the ML/AI research mission
3. **Open source**: No monetization, community-driven development
4. **Reproducibility**: All experiments versioned and documented
5. **Accessibility**: Python-first for researchers, Rust for performance

---

## Current Status

**Phase**: 2 (New Horizons Edition)
**Version**: 0.4.0
**Cards**: 107 / 300
**Next milestone**: Complete card expansion
