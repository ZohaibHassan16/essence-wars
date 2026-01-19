# Phase 4: Research Platform Tracker

> **Goal**: Build the agent ecosystem and benchmarking infrastructure to establish Essence Wars as a legitimate research platform that attracts contributors and citations.

**Status**: 🔄 In Progress
**Started**: January 2026
**Target**: Complete before public launch

---

## Executive Summary

Phase 4 transforms Essence Wars from a working engine into a **research platform**. The key deliverables are:

1. **Agent Roster**: 5+ trained neural agents (PPO, AlphaZero variants)
2. **Research Contributions**: Learned card embeddings + Bitnet reward shaping
3. **Benchmarking**: Elo leaderboard + standardized evaluation
4. **Publication**: Paper 1 (systems/benchmark paper)
5. **HuggingFace Presence**: Models, datasets, documentation

---

## Current Status Overview

### Already Complete ✅

| Item | Status | Notes |
|------|--------|-------|
| PPO Agent Implementation | ✅ | `python/essence_wars/agents/ppo.py` (~300 LOC) |
| AlphaZero Implementation | ✅ | `python/essence_wars/agents/alphazero.py` (~400 LOC) |
| Benchmark API | ✅ | `python/essence_wars/benchmark/` |
| Elo Calculation | ✅ | `benchmark/elo.py` |
| HuggingFace Hub Integration | ✅ | `python/essence_wars/hub.py` |
| 5 Tutorial Notebooks | ✅ | `notebooks/01-05` |
| Dataset Generation | ✅ | `generate_dataset.rs` + 3 datasets |
| Datasets on HF | ✅ | See [Datasets](#datasets-published) below |

### In Progress 🔄

| Item | Status | Owner |
|------|--------|-------|
| Trained Model Checkpoints | 🔄 | Need to train and upload |
| Learned Card Embeddings | 🔄 | Design + implementation |
| Bitnet Reward Shaping | 🔄 | Research spike + implementation |
| Paper 1 | 🔄 | Writing |

### Not Started ❌

| Item | Blocked By |
|------|------------|
| Elo Leaderboard Publication | Trained models |
| Agent Submission Documentation | Finalized agent interface |
| Notebook Review/Updates | Agent roster completion |

---

## Datasets Published

| Dataset | Games | Size | HuggingFace URL |
|---------|-------|------|-----------------|
| mcts-1k-sims100 | 1,000 | 5 MB | [Chris-Essence-Wars/mcts-1k-sims100](https://huggingface.co/datasets/Chris-Essence-Wars/mcts-1k-sims100) |
| mcts-10k-sims100 | 10,000 | 56 MB | [Chris-Essence-Wars/mcts-10k-sims100](https://huggingface.co/datasets/Chris-Essence-Wars/mcts-10k-sims100) |
| mcts-100k-sims100 | 100,000 | 565 MB | [Chris-Essence-Wars/mcts-100k-sims100](https://huggingface.co/datasets/Chris-Essence-Wars/mcts-100k-sims100) |

**Local copies**: `/home/chris/ai-cardgame/data/datasets/`

---

## Track A: Agent Roster

**Goal**: 5 trained neural agents with published checkpoints

### A1. PPO Agents

| Agent | Training Data | Target Win Rate | Status |
|-------|---------------|-----------------|--------|
| PPO-Generalist | All factions | >55% vs MCTS-100 | ❌ Not trained |
| PPO-Argentum | Argentum decks | >60% vs MCTS-100 | ❌ Not trained |
| PPO-Symbiote | Symbiote decks | >60% vs MCTS-100 | ❌ Not trained |
| PPO-Obsidion | Obsidion decks | >60% vs MCTS-100 | ❌ Not trained |

**Tasks**:
- [ ] Configure training hyperparameters for generalist
- [ ] Train PPO-Generalist (target: 10M steps)
- [ ] Evaluate against MCTS-100 baseline
- [ ] Train 3 faction specialists
- [ ] Upload all checkpoints to HF

**Training Script**: `python/scripts/train_ppo.py`

### A2. AlphaZero Agent

| Agent | Self-Play Games | Target Win Rate | Status |
|-------|-----------------|-----------------|--------|
| AlphaZero-v1 | 100K+ | >60% vs MCTS-100 | ❌ Not trained |

**Tasks**:
- [ ] Configure self-play parameters
- [ ] Train AlphaZero-v1
- [ ] Evaluate against baselines
- [ ] Upload checkpoint to HF

**Training Script**: `python/scripts/train_alphazero.py`

---

## Track B: Research Infrastructure

### B1. Learned Card Embeddings

**Research Question**: Does replacing one-hot card IDs with learned embeddings improve:
1. Training sample efficiency?
2. Generalization to unseen decks?
3. Transfer to new cards (future expansions)?

**Implementation Plan**:

```
observation_mode="flat"     (current)  → 326 floats, one-hot card IDs
observation_mode="embedded" (new)      → Variable size, learned embeddings
```

**Architecture Comparison** (implement both):

| Approach | Description | Pros | Cons |
|----------|-------------|------|------|
| End-to-End | Learn embeddings with policy | Simple, task-specific | May not transfer |
| Pre-trained | card2vec → fine-tune | Better transfer | More complex |

**Tasks**:
- [x] Design embedding layer architecture
- [x] Implement `ObservationTransformer` class
- [x] Implement `EmbeddedPPONetwork` (end-to-end approach)
- [x] Implement `EmbeddedAlphaZeroNetwork`
- [x] Add `--observation-mode` to training scripts
- [x] Add unit tests (35 tests passing)
- [x] Implement pre-trained Card2Vec approach
- [ ] Train PPO with both modes
- [ ] Compare sample efficiency curves
- [ ] Compare generalization (train on 3 factions, test on 4th)
- [ ] Document findings for Paper 1

**Key Files**:
- `python/essence_wars/agents/embeddings.py` - ObservationTransformer, EmbeddedNetworks
- `python/essence_wars/agents/ppo.py` - PPOConfig with observation_mode
- `python/scripts/train_ppo.py` - CLI with --observation-mode flag
- `docs/embedding-design.md` - Full design documentation

### B2. Bitnet Reward Shaping

**Research Question**: Can we improve learning in sparse-reward card games by adding "shadow rewards" while keeping the final reward binary?

**Concept**:
```
Final Reward:   Win = +1.0, Loss = -1.0 (unchanged)
Shadow Rewards: Dense signals for intermediate states (not used for final evaluation)
```

**Proposed Shadow Signals**:

| Signal | Description | Range |
|--------|-------------|-------|
| Life Differential | (my_life - opp_life) / 60 | [-1, 1] |
| Board Control | (my_creatures - opp_creatures) / 10 | [-1, 1] |
| Card Advantage | (my_hand + my_deck - opp_hand - opp_deck) / 60 | [-1, 1] |
| Tempo | action_points_used / 3 | [0, 1] |

**Implementation Approaches**:

| Approach | Description |
|----------|-------------|
| Auxiliary Heads | Separate prediction heads for shadow signals (multi-task learning) |
| Reward Shaping | Add scaled shadow signals to reward: `r = final + 0.01 * shadow` |
| Curriculum | Start with dense shadows, anneal to sparse |

**Tasks**:
- [ ] Implement shadow reward calculation in engine
- [ ] Add `reward_mode` parameter: `sparse`, `dense`, `curriculum`
- [ ] Train PPO with each mode
- [ ] Compare learning curves
- [ ] Analyze if shadow rewards hurt final performance
- [ ] Document findings for Paper 1

**Hypothesis**: Shadow rewards accelerate early learning but may need annealing to avoid suboptimal convergence.

---

## Track C: Benchmarking & Leaderboard

### C1. Benchmark Suite

**Already Implemented** in `python/essence_wars/benchmark/`:
- `EssenceWarsBenchmark` class
- 4 standard baselines: Random, Greedy, MCTS-50, MCTS-100
- Per-deck performance breakdown
- Elo rating calculation
- JSON export

**Tasks**:
- [ ] Validate benchmark API works end-to-end
- [ ] Run baseline evaluations (MCTS vs MCTS)
- [ ] Document benchmark methodology
- [ ] Add timing/throughput metrics

### C2. Elo Leaderboard

**Format**: Static leaderboard (markdown + JSON)

```markdown
# Essence Wars Agent Leaderboard

| Rank | Agent | Elo | vs Random | vs Greedy | vs MCTS-100 |
|------|-------|-----|-----------|-----------|-------------|
| 1 | AlphaZero-v1 | 1850 | 99.2% | 87.3% | 62.1% |
| 2 | PPO-Generalist | 1720 | 98.5% | 78.2% | 55.3% |
| 3 | MCTS-500 | 1680 | 97.8% | 72.1% | 58.0% |
| ... | ... | ... | ... | ... | ... |
```

**Tasks**:
- [ ] Define leaderboard schema (`leaderboard.json`)
- [ ] Create leaderboard generation script
- [ ] Run evaluations for all agents
- [ ] Generate `docs/leaderboard.md`
- [ ] Add to GitHub Pages dashboard

---

## Track D: Documentation & Publishing

### D1. Research Documentation

| Document | Purpose | Status |
|----------|---------|--------|
| Agent Submission Guide | How researchers contribute new agents | ❌ Not written |
| Researcher Quickstart | 5-minute path from `pip install` to training | 🔄 Needs review |
| Benchmark Methodology | How we evaluate agents | ❌ Not written |
| Embedding Comparison | Results of flat vs embedded | ❌ Pending research |

**Tasks**:
- [ ] Write `docs/agent-submission.md`
- [ ] Review and update `python/README.md` (researcher quickstart)
- [ ] Write `docs/benchmark-methodology.md`
- [ ] Write `docs/embeddings-comparison.md` (after research)

### D2. Notebook Review

**Current Notebooks**:
1. `01_quickstart.ipynb` - PyGame basics
2. `02_environment.ipynb` - Gymnasium API
3. `03_dataset_exploration.ipynb` - Loading MCTS data
4. `04_behavioral_cloning.ipynb` - Imitation learning
5. `05_alphazero_training.ipynb` - AlphaZero training

**Tasks**:
- [ ] Review each notebook for accuracy with v0.7.0
- [ ] Add notebook using Agent Roster (load pretrained, evaluate)
- [ ] Add notebook comparing embedding modes
- [ ] Ensure all notebooks run end-to-end

### D3. HuggingFace Publishing

**Namespace**:  `Chris-Essence-Wars/`

**Models to Upload**:
| Model | HF Path | Status |
|-------|---------|--------|
| PPO-Generalist | `Chris-Essence-Wars/ppo-generalist` | ❌ Pending |
| PPO-Argentum | `Chris-Essence-Wars/ppo-argentum` | ❌ Pending |
| PPO-Symbiote | `Chris-Essence-Wars/ppo-symbiote` | ❌ Pending |
| PPO-Obsidion | `Chris-Essence-Wars/ppo-obsidion` | ❌ Pending |
| AlphaZero-v1 | `Chris-Essence-Wars/alphazero-v1` | ❌ Pending |

**Tasks**:
- [ ] Train models (Track A)
- [ ] Create model cards with usage examples
- [ ] Upload via `hub.py` upload functions
- [ ] Verify download works via `load_pretrained()`

---

## Track E: Research Output

### E1. Paper 1: Systems/Benchmark Paper

**Title**: "Essence Wars: A High-Performance Card Game Engine for RL Research"

**Target Venues**:
- NeurIPS 2026 Datasets & Benchmarks Track (primary)
- AAAI Artifact Track (backup)
- arXiv (immediate)

**Key Contributions**:
1. Zero-allocation engine design (101ns state cloning)
2. Deterministic execution (perfect reproducibility)
3. Fast Python bindings (~268k steps/sec vectorized)
4. Complete benchmark suite (agents, decks, datasets)
5. **Novel**: Learned card embeddings comparison
6. **Novel**: Bitnet reward shaping for sparse-reward games

**Outline**:
```
1. Introduction
   - Motivation: Card games underexplored in RL
   - Contribution summary

2. Related Work
   - Game AI environments (Atari, MuJoCo, StarCraft)
   - Card game AI (Hearthstone, MTG)
   - Deterministic vs stochastic environments

3. Essence Wars Engine
   - Game mechanics overview
   - Architecture decisions (zero-allocation, determinism)
   - Performance benchmarks

4. Research Infrastructure
   - Gymnasium interface
   - Benchmark suite
   - Dataset generation

5. Experiments
   5.1 Baseline Agents (MCTS, Greedy, Random)
   5.2 Neural Agents (PPO, AlphaZero)
   5.3 Learned Card Embeddings (flat vs embedded)
   5.4 Reward Shaping (sparse vs dense vs curriculum)

6. Results
   - Win rates and Elo ratings
   - Sample efficiency comparison
   - Embedding transfer learning
   - Reward shaping analysis

7. Discussion & Future Work
   - LLM agents
   - Multi-agent training
   - Expansion sets (continual learning)

8. Conclusion
```

**Tasks**:
- [ ] Create LaTeX/Overleaf project
- [ ] Write Section 1-3 (can do now)
- [ ] Complete experiments (Tracks A, B)
- [ ] Write Section 4-6 (after experiments)
- [ ] Internal review
- [ ] Submit to arXiv
- [ ] Submit to NeurIPS (deadline TBD, typically May/June)

---

## Research Ideas Backlog

These are ideas noted in the roadmap but not prioritized for Phase 4 core:

| Idea | Description | Priority | Notes |
|------|-------------|----------|-------|
| DQN/Rainbow Agent | Algorithm diversity | Medium | Would strengthen Paper 1 |
| W&B Integration | Experiment tracking | Medium | Nice-to-have |
| ONNX Export | Browser deployment | Low | Phase 5A |
| Distributed PPO | Ray/RLlib scaling | Low | Performance optimization |
| LLM Agent | Reasoning-based play | Low | Phase 6 |

---

## Dependencies & Sequencing

```
                    ┌─────────────────┐
                    │  Datasets (✅)  │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
       ┌────────────┐ ┌────────────┐ ┌────────────┐
       │ Train PPO  │ │  Train AZ  │ │ Embeddings │
       │   Agents   │ │   Agent    │ │  Research  │
       └──────┬─────┘ └──────┬─────┘ └──────┬─────┘
              │              │              │
              └──────────────┼──────────────┘
                             ▼
                    ┌─────────────────┐
                    │  Benchmark All  │
                    │    Agents       │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
       ┌────────────┐ ┌────────────┐ ┌────────────┐
       │ Leaderboard│ │  Upload to │ │   Paper 1  │
       │   .md/.json│ │     HF     │ │   Writing  │
       └────────────┘ └────────────┘ └────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  Public Launch  │
                    │   (Phase 5A)    │
                    └─────────────────┘
```

---

## Success Criteria

Phase 4 is complete when:

1. **Agent Roster**: 5 trained models uploaded to HuggingFace
2. **Embeddings**: Comparison documented (flat vs embedded)
3. **Reward Shaping**: Bitnet approach evaluated
4. **Leaderboard**: Published with all agent Elo ratings
5. **Paper 1**: Submitted to arXiv (conference submission is bonus)
6. **Notebooks**: All run successfully, 1-2 new notebooks added

---

## Timeline (Tentative)

| Milestone | Target Date | Status |
|-----------|-------------|--------|
| Embedding architecture design | Week 1 | ❌ |
| PPO-Generalist trained | Week 2-3 | ❌ |
| Reward shaping experiments | Week 3-4 | ❌ |
| AlphaZero-v1 trained | Week 4-5 | ❌ |
| Faction specialists trained | Week 5-6 | ❌ |
| Benchmark all agents | Week 6-7 | ❌ |
| Paper 1 draft complete | Week 8-10 | ❌ |
| HuggingFace models uploaded | Week 10 | ❌ |
| arXiv submission | Week 11 | ❌ |

---

## Notes & Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-01-19 | Build First strategy | Complete all features before public launch |
| 2026-01-19 | Personal HF namespace | Use `christianwissmann/` or `Chris-Essence-Wars/` |
| 2026-01-19 | Full agent roster | 5 models: PPO-Gen + 3 specialists + AZ-v1 |
| 2026-01-19 | Both embedding approaches | End-to-end AND pre-trained for paper comparison |
| 2026-01-19 | Implement Bitnet reward shaping | Novel research contribution |

---

## References

- [ROADMAP.md](./ROADMAP.md) - Overall project roadmap
- [research-agenda.md](/home/chris/ai-cardgame/research-agenda.md) - Research paper ideas
- [design-engine.md](./design-engine.md) - Engine architecture
- [tuning-pipeline.md](./tuning-pipeline.md) - Weight tuning methodology
