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

## Phase 2: New Horizons Edition ✅

*Complete the first edition card set and streamline tooling.*

- [x] Expand card pool to 300 cards (New Horizons Edition)
- [x] Balance all faction matchups (45-55% win rates)
- [x] Set up Modal cloud tuning (internal tooling)
- [x] Update documentation after the 300 Cards have been implemented (`docs/essence-wars-design.md`, `docs/design-engine.md`) to ensure all new effects and keywords are documented, all architectural and design changes are reflected and documented
- [x] Create card database reference (`docs/cards-new-horizons.md`)
- [x] Analzye, Discuss the Data Pipeline, Clean up and document tuning/validation scripts
- [x] Fix, rework or refactor `scripts/analyze-mcts.sh` and the Analysis Pipeline
- [x] Remove TUI code (deferred indefinitely)
- [x] Discuss if a Match Mode should be added for the additional Win Codnition (Victory Points), or if this would seriously disturb overall balance and hurt our mission, or be a good addition
  - 1. **Enemy life ≤ 0** → You win (Already implemented)
  - 2. **50 Victory Points** → You win (Basic Infrastructure implemented, not enabled nor integrated yet)
  - 3. **Turn 30** → Higher life wins (draw if tied) (should be implemented, check)
- [x] Version Bump of the Engine after everything is done for Phase 2

---

## Phase 3: ML Infrastructure 🔄

*Make Essence Wars accessible to ML researchers.*

- [x] Create PyO3 Python bindings
- [x] Implement Gymnasium environment interface
- [ ] Publish `essence-wars` package to PyPI (`pip install essence-wars`)
- [ ] Publish Rust crate to crates.io (`cargo add essence-wars`)
- [ ] Train first proof-of-concept PPO agent
- [x] Validate Gym interface with standard RL libraries (Stable-Baselines3, CleanRL)
- [ ] Write researcher quickstart guide
- [ ] Design JRPG integration architecture (`docs/jrpg-architecture.md`)
- [ ] Define clean API layer for external game clients

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
- [ ] Build minimal Bevy 3D integration prototype (validates JRPG tech stack)
- [ ] Design analytics API for Glassbox Mode (decision trees, replay system)
- [ ] Add Learned Card Embeddings `observation_mode="flat"` vs `observation_mode="embedded"`  
  - Research contribution: Compare both in a paper!
- [ ] Review and Improve Tutorial Notebooks 📓, add one or two using the new Agent Roster
- [ ] Idea: Use the Bitnet Approach for Reward Shaping: Rewards remain binary -1, 1, but we can add 'shadow reward shapes' that are using smaller values to give the neural nets something to work for like 1.58b Bitnet Does it for LLMs ? 

---

## Phase 5A: Web Playable Game

*Deploy interactive web experience on Huggingface Spaces.*

### Human Play Mode
- [ ] Build web client UI (web-native)
- [ ] Implement Human vs AI matches (select opponent: MCTS/PPO/AlphaZero/LLM)
- [ ] Add AI hint system (limited charges per game, suggests strong moves)
- [ ] Create deck builder interface
- [ ] Add match history and statistics tracking
- [ ] Deploy to Huggingface Spaces

### Glassbox Mode (Research Visualization)
- [ ] Implement AI vs AI spectator with replay controls
- [ ] Visualize MCTS search trees (node visits, UCB values)
- [ ] Display neural network policy/value outputs in real-time
- [ ] Show decision-making rationale (top-N action probabilities)
- [ ] Add variable playback speed and step-through controls
- [ ] Export match analytics (turn-by-turn state tensors, action logs)
- [ ] Create interactive tutorial explaining AI decision-making

### Content & Community
- [ ] Add basic lore and faction identities
- [ ] Commission or create card artwork (priority: core set)
- [ ] Add flavor text to all cards
- [ ] Set up community feedback system
- [ ] Collect human play data for offline RL research

---

## Phase 5B: JRPG Campaign

*Expand into story-driven 3D game using Bevy engine.*

**Conditional on Phase 5A success and resource availability.**

### 3D Game Infrastructure
- [ ] Build Bevy 3D overworld navigation system
- [ ] Integrate card game as turn-based battle module
- [ ] Implement third-person camera system
- [ ] Create encounter system (trigger battles from 3D world)
- [ ] Design progression system (unlock cards through story)

### Story & Content
- [ ] Write three faction campaigns (Argentum, Obsidion, Symbiote)
- [ ] Design branching narrative with meaningful choices
- [ ] Create 3D environments (towns, battle arenas, dungeons)
- [ ] Implement story-driven deck progression
- [ ] Add cutscenes and NPC dialogue system

### Advanced Features
- [ ] Build save/load system for campaign progress
- [ ] Create side quests and optional battles
- [ ] Add achievements and completion tracking
- [ ] Design boss encounters with unique mechanics
- [ ] Launch essence-wars.ai dedicated website (if distinct from Huggingface)

---

## Phase 6: Frontier Research

*Advanced research directions and ongoing development.*

- [ ] LLM agent integration (reasoning-based play with narrative context)
- [ ] Transfer learning experiments (new cards without retraining)
- [ ] Deck-building agents (discover combos and strategies)
- [ ] Procedural campaign generation (AI-driven story and encounters)
- [ ] Multi-agent research (team play, drafting, cooperative battles)
- [ ] First expansion set (post-New Horizons Edition)
- [ ] Research on narrative-aware agents (JRPG context influences strategy)
- [ ] Community-created campaigns and modding support

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
