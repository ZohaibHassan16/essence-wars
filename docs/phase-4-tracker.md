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
| **PPO Agent Roster** | ✅ | 5 models trained & uploaded to HF (see Track A) |
| **Learned Card Embeddings** | ✅ | Flat vs Embedded vs Pretrained comparison complete |
| **Card2Vec Pre-training** | ✅ | `models/card2vec_20260119_120507.pt` |
| **Behavioral Cloning** | ✅ | 59% vs Greedy, `models/bc_mcts_10k_best.pt` |
| **Paper 1 Findings** | ✅ | `papers/paper1-findings.md` with all results |
| **Benchmark Suite** | ✅ | `python/essence_wars/benchmark/` validated end-to-end |
| **Elo Leaderboard** | ✅ | JSON + Markdown + HuggingFace Space |
| **Agent Submission System** | ✅ | CLI + GitHub Actions + Documentation |
| **Researcher Quickstart** | ✅ | `python/README.md` - 5 min from pip to training |
| **Benchmark Methodology** | ✅ | `docs/benchmark-methodology.md` |
| **Notebooks Updated** | ✅ | All 6 notebooks with correct URLs + leaderboard links |

### In Progress 🔄

| Item | Status | Owner |
|------|--------|-------|
| Paper 1 | 🔄 | Findings documented, needs formal write-up |
| Bitnet Reward Shaping | 🔄 | Research spike (deprioritized) |
| BC + AlphaZero fine-tuning | 🔄 | Investigating mitigations for catastrophic forgetting |

### Not Started ❌

| Item | Blocked By |
|------|------------|
| Paper 1 Formal Write-up | Time (findings documented) |

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

### A1. PPO Agents ✅ COMPLETE

| Agent | Architecture | Best Win Rate | HuggingFace | Status |
|-------|--------------|---------------|-------------|--------|
| PPO-Argentum | Embedded | **72.0%** vs Greedy | [ppo-argentum](https://huggingface.co/Chris-Essence-Wars/ppo-argentum) | ✅ Uploaded |
| PPO-Flat | Flat | **71.0%** vs Greedy | [ppo-flat](https://huggingface.co/Chris-Essence-Wars/ppo-flat) | ✅ Uploaded |
| PPO-Embedded | Embedded | **65.0%** vs Greedy | [ppo-embedded](https://huggingface.co/Chris-Essence-Wars/ppo-embedded) | ✅ Uploaded |
| PPO-Symbiote | Embedded | **65.0%** vs Greedy | [ppo-symbiote](https://huggingface.co/Chris-Essence-Wars/ppo-symbiote) | ✅ Uploaded |
| PPO-Obsidion | Embedded | **62.0%** vs Greedy | [ppo-obsidion](https://huggingface.co/Chris-Essence-Wars/ppo-obsidion) | ✅ Uploaded |

**Key Finding**: Best checkpoint saving is critical - all models showed policy collapse but best checkpoints capture 62-72% performance.

**Tasks**:
- [x] Configure training hyperparameters for generalist
- [x] Train PPO-Generalist (300k steps with best checkpoint saving)
- [x] Evaluate against Greedy baseline
- [x] Train 3 faction specialists
- [x] Upload all checkpoints to HF

**Training Script**: `python/scripts/train_ppo.py`
**Roster Script**: `scripts/train_ppo_roster.sh`

### A2. AlphaZero Agent ⚠️ COLD-START FAILURE

| Agent | Self-Play Games | Win Rate vs Greedy | Status |
|-------|-----------------|-------------------|--------|
| AlphaZero (from scratch) | 10,000 | **0%** | ❌ Cold-start failure |
| AlphaZero (BC init, LR=1e-3) | 10,000 | **0%** | ❌ Catastrophic forgetting |
| AlphaZero (BC init, LR=1e-4) | 10,000 | **0%** | ❌ Catastrophic forgetting |

**Key Findings**:

1. **Cold-Start Problem**: AlphaZero from scratch achieves 0% win rate after 100 iterations (10k games, 14.3 hours). The bootstrapping problem is severe:
   - MCTS needs good value function → value function trained from MCTS → chicken-and-egg
   - Sparse rewards (only terminal) provide no gradient signal for intermediate positions
   - We used 10k games vs AlphaGo Zero's 4.9M games (500x less data)

2. **Catastrophic Forgetting**: Initializing from BC checkpoint (59% → 0%) destroys learned policy immediately, even at LR=1e-4

3. **Potential Mitigations** (to investigate):
   - KL divergence penalty to anchor to BC policy
   - Mixed replay buffer (BC data + self-play data)
   - Freeze early layers during initial fine-tuning
   - Curriculum: start vs Random before self-play

**Tasks**:
- [x] Configure self-play parameters
- [x] Train AlphaZero from scratch (100 iterations) - **0% win rate**
- [x] Train AlphaZero from BC checkpoint - **Catastrophic forgetting**
- [x] Document findings in `papers/paper1-findings.md`
- [ ] Investigate mitigation strategies (KL penalty, mixed replay)

**Training Script**: `python/scripts/train_alphazero.py`

### A3. Behavioral Cloning Agent ✅ COMPLETE

| Agent | Dataset | Win Rate | Status |
|-------|---------|----------|--------|
| BC-MCTS-10k | mcts-10k-sims100 | **59%** vs Greedy | ✅ Trained |

**Key Finding**: BC achieves 59% in just 12 minutes of training. Fine-tuning with AlphaZero causes catastrophic forgetting.

---

## Track B: Research Infrastructure

### B1. Learned Card Embeddings ✅ COMPLETE

**Research Question**: Does replacing one-hot card IDs with learned embeddings improve:
1. Training sample efficiency?
2. Generalization to unseen decks?
3. Transfer to new cards (future expansions)?

**Results Summary**:

| Architecture | Best Win Rate | Final Win Rate | Notes |
|--------------|---------------|----------------|-------|
| Flat | **71.0%** | 53.5% | Collapses but recovers with best checkpoint |
| Embedded (learned) | 65.0% | 51.0% | More stable during training |
| Embedded (pretrained) | 45.5% | 45.5% | Card2Vec objective doesn't transfer well |

**Key Findings**:
1. **Flat and Embedded are comparable** (71% vs 65%) - embeddings not strictly necessary
2. **Pretrained Card2Vec underperforms** - co-occurrence objective ≠ game-winning objective
3. **Best checkpoint saving is critical** - recovers 7-62% performance lost to collapse
4. **Two bugs discovered and fixed** in embeddings.py (see paper1-findings.md Section 5.2)

**Tasks**:
- [x] Design embedding layer architecture
- [x] Implement `ObservationTransformer` class
- [x] Implement `EmbeddedPPONetwork` (end-to-end approach)
- [x] Implement `EmbeddedAlphaZeroNetwork`
- [x] Add `--observation-mode` to training scripts
- [x] Add unit tests (35 tests passing)
- [x] Implement pre-trained Card2Vec approach
- [x] Train PPO with all three modes (flat, embedded, embedded_pretrained)
- [x] Compare sample efficiency curves
- [x] Document findings in `papers/paper1-findings.md`
- [ ] Compare generalization (train on 3 factions, test on 4th) - future work

**Key Files**:
- `python/essence_wars/agents/embeddings.py` - ObservationTransformer, EmbeddedNetworks
- `python/essence_wars/agents/ppo.py` - PPOConfig with observation_mode
- `python/scripts/train_ppo.py` - CLI with --observation-mode flag
- `papers/paper1-findings.md` - Full experimental results

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

## Track C: Benchmarking & Leaderboard ✅ COMPLETE

### C1. Benchmark Suite ✅

**Implemented** in `python/essence_wars/benchmark/`:
- `EssenceWarsBenchmark` class with `evaluate()` and `quick_evaluate()`
- 4 standard baselines: Random, Greedy, MCTS-50, MCTS-100
- Per-deck performance breakdown
- Elo rating calculation
- JSON export

**Tasks**:
- [x] Validate benchmark API works end-to-end
- [x] Run baseline evaluations (MCTS vs MCTS)
- [x] Add timing/throughput metrics (decision time in results)
- [ ] Document benchmark methodology (future: `docs/benchmark-methodology.md`)

### C2. Elo Leaderboard ✅

**Deliverables**:

| Artifact | Location | Description |
|----------|----------|-------------|
| Leaderboard Data | `data/leaderboard/leaderboard.json` | JSON schema with all agents |
| Markdown Export | `docs/leaderboard.md` | Static markdown table |
| HTML Export | `docs/leaderboard.html` | Styled HTML version |
| **HuggingFace Space** | [essence-wars-leaderboard](https://huggingface.co/spaces/Chris-Essence-Wars/essence-wars-leaderboard) | Interactive Gradio app |
| Submission CLI | `python/scripts/submit_agent.py` | Local agent submission |
| Benchmark Runner | `python/scripts/run_benchmark.py` | CI/CD evaluation |
| GitHub Actions | `.github/workflows/evaluate-agent.yml` | Automated evaluation on issue |
| Submission Guide | `docs/SUBMIT_AGENT.md` | How to submit agents |

**Tasks**:
- [x] Define leaderboard schema (`leaderboard.json`)
- [x] Create leaderboard generation script
- [x] Run evaluations for all agents (10 agents: 4 baselines + 6 trained)
- [x] Generate `docs/leaderboard.md`
- [x] Create HuggingFace Space with interactive leaderboard
- [x] Create agent submission documentation
- [x] Create GitHub Actions workflow for automated evaluation

---

## Track D: Documentation & Publishing

### D1. Research Documentation ✅ COMPLETE

| Document | Purpose | Status |
|----------|---------|--------|
| Agent Submission Guide | How researchers contribute new agents | ✅ `docs/SUBMIT_AGENT.md` |
| Researcher Quickstart | 5-minute path from `pip install` to training | ✅ `python/README.md` |
| Benchmark Methodology | How we evaluate agents | ✅ `docs/benchmark-methodology.md` |
| Embedding Comparison | Results of flat vs embedded | ✅ `papers/paper1-findings.md` |
| Paper 1 Findings | Experimental results and analysis | ✅ Complete |

**Tasks**:
- [x] Write `docs/SUBMIT_AGENT.md` (agent submission guide)
- [x] Create `python/README.md` (5-minute researcher quickstart)
- [x] Write `docs/benchmark-methodology.md`
- [x] Document embedding comparison findings

### D2. Notebook Review ✅ COMPLETE

**Notebooks** (6 total):
1. `01_quickstart.ipynb` - PyGame basics ✅ Updated
2. `02_environment.ipynb` - Gymnasium API ✅ Updated
3. `03_dataset_exploration.ipynb` - Loading MCTS data ✅ Updated
4. `04_behavioral_cloning.ipynb` - Imitation learning ✅ Updated
5. `05_alphazero_training.ipynb` - AlphaZero training ✅ Updated
6. `06_pretrained_agents.ipynb` - **NEW** Load & evaluate pretrained models ✅ Created

**Updates Made**:
- [x] Fixed Colab badge URLs (`ai-cardgame` → `essence-wars`)
- [x] Updated HuggingFace namespace (`Chris-Essence-Wars`)
- [x] Added leaderboard links to all "Next Steps" sections
- [x] Created new notebook for pretrained agent evaluation
- [x] Added submission guide links

### D3. HuggingFace Publishing ✅ PPO COMPLETE

**Namespace**:  `Chris-Essence-Wars/`

**Models Uploaded**:
| Model | HF URL | Win Rate | Status |
|-------|--------|----------|--------|
| PPO-Argentum | [ppo-argentum](https://huggingface.co/Chris-Essence-Wars/ppo-argentum) | 72.0% | ✅ Uploaded |
| PPO-Flat | [ppo-flat](https://huggingface.co/Chris-Essence-Wars/ppo-flat) | 71.0% | ✅ Uploaded |
| PPO-Embedded | [ppo-embedded](https://huggingface.co/Chris-Essence-Wars/ppo-embedded) | 65.0% | ✅ Uploaded |
| PPO-Symbiote | [ppo-symbiote](https://huggingface.co/Chris-Essence-Wars/ppo-symbiote) | 65.0% | ✅ Uploaded |
| PPO-Obsidion | [ppo-obsidion](https://huggingface.co/Chris-Essence-Wars/ppo-obsidion) | 62.0% | ✅ Uploaded |
| AlphaZero-v1 | N/A | 0% (cold-start failure) | ❌ Not uploaded |

**Tasks**:
- [x] Train PPO models (Track A)
- [x] Create model cards with usage examples
- [x] Upload PPO models via `hub.py` upload functions
- [x] ~~Upload AlphaZero after training completes~~ - Skipped (0% performance)
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

1. **Agent Roster**: 5 trained models uploaded to HuggingFace - ✅ **5 PPO models done, AlphaZero failed (cold-start)**
2. **Embeddings**: Comparison documented (flat vs embedded) - ✅ **Complete** (`papers/paper1-findings.md`)
3. **Reward Shaping**: Bitnet approach evaluated - ⏳ Deprioritized (policy collapse more interesting finding)
4. **Leaderboard**: Published with all agent Elo ratings - ✅ **Complete** (10 agents, HuggingFace Space live)
5. **Paper 1**: Submitted to arXiv (conference submission is bonus) - 🔄 Findings documented, needs formal write-up
6. **Notebooks**: All run successfully, 1-2 new notebooks added - ✅ **Complete** (6 notebooks, all updated + 1 new)

---

## Timeline (Actual Progress)

| Milestone | Target Date | Status |
|-----------|-------------|--------|
| Embedding architecture design | Week 1 | ✅ Complete |
| Card2Vec pre-training | Week 1 | ✅ Complete |
| Behavioral Cloning baseline | Week 1 | ✅ 59% vs Greedy |
| PPO Generalist trained | Week 1 | ✅ 71% (flat), 65% (embedded) |
| Faction specialists trained | Week 1 | ✅ 62-72% vs Greedy |
| HuggingFace PPO models uploaded | Week 1 | ✅ 5 models uploaded |
| **Leaderboard & Benchmarking** | Week 1 | ✅ HF Space + submission system live |
| AlphaZero-v1 trained | Week 1-2 | ❌ Cold-start failure (0% after 100 iter, 14.3 hrs) |
| Benchmark all agents | Week 2 | ✅ 10 agents evaluated |
| Paper 1 draft complete | Week 2-3 | 🔄 Findings documented |
| arXiv submission | Week 3-4 | ⏳ Pending |

---

## Notes & Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-01-19 | Build First strategy | Complete all features before public launch |
| 2026-01-19 | Personal HF namespace | Use `Chris-Essence-Wars/` |
| 2026-01-19 | Full agent roster | 5 models: PPO-Gen + 3 specialists + AZ-v1 |
| 2026-01-19 | Both embedding approaches | End-to-end AND pre-trained for paper comparison |
| 2026-01-19 | Implement Bitnet reward shaping | Novel research contribution (deprioritized) |
| 2026-01-19 | **Best checkpoint saving** | Critical for handling policy collapse - recovers 7-62% perf |
| 2026-01-19 | **Higher entropy (0.02)** | Reduces policy collapse rate |
| 2026-01-19 | **Embedded for specialists** | Faction specialists with flat architecture collapsed (2-10%), embedded solved it (62-72%) |
| 2026-01-19 | **Bug fixes in embeddings.py** | Found 2 bugs: pretrained weights not loading, raw card IDs as features |
| 2026-01-19 | **Flat ≈ Embedded performance** | Both achieve 65-71%, embeddings not strictly necessary for PPO |
| 2026-01-19 | **Pretrained Card2Vec underperforms** | Co-occurrence objective doesn't transfer to game-winning objective |
| 2026-01-19 | **Track C Complete** | Leaderboard + HF Space + submission system + GitHub Actions all working |
| 2026-01-19 | **Track D1 Complete** | Researcher quickstart + benchmark methodology docs written |
| 2026-01-19 | **Track D2 Complete** | All 6 notebooks updated, new 06_pretrained_agents.ipynb created |
| 2026-01-20 | **AlphaZero cold-start failure** | 100 iterations (10k games, 14.3 hrs) → 0% win rate. Bootstrapping problem severe. |
| 2026-01-20 | **BC→AlphaZero catastrophic forgetting** | Both LR=1e-3 and LR=1e-4 immediately destroy BC policy (59% → 0%) |
| 2026-01-20 | **Investigating mitigations** | KL penalty, mixed replay buffer, freezing layers as potential solutions |

---

## References

- [ROADMAP.md](./ROADMAP.md) - Overall project roadmap
- [research-agenda.md](/home/chris/ai-cardgame/research-agenda.md) - Research paper ideas
- [design-engine.md](./design-engine.md) - Engine architecture
- [tuning-pipeline.md](./tuning-pipeline.md) - Weight tuning methodology
- **[paper1-findings.md](/home/chris/ai-cardgame/papers/paper1-findings.md)** - Experimental results and analysis
- **[Leaderboard (HF Space)](https://huggingface.co/spaces/Chris-Essence-Wars/essence-wars-leaderboard)** - Interactive agent leaderboard
- [docs/leaderboard.md](./leaderboard.md) - Static leaderboard
- [docs/SUBMIT_AGENT.md](./SUBMIT_AGENT.md) - Agent submission guide
- [docs/benchmark-methodology.md](./benchmark-methodology.md) - Evaluation protocol
- [python/README.md](/home/chris/ai-cardgame/python/README.md) - Researcher quickstart
