# 🔬 Chris' Personal Research Agenda (The "Founding Papers")

## **Paper 1: "Essence Wars: A High-Performance Card Game Engine for RL Research"** 📄

**Type:** Systems/Benchmark paper (think: MuJoCo paper, Atari ALE paper)

**The pitch:**
> "We introduce Essence Wars, a deterministic card game engine achieving 67.6K games/sec (450x faster than typical Python card game simulators). We provide a complete benchmark suite including 300 cards, 12 pre-tuned decks, and baseline agents (Random, Greedy, MCTS). We demonstrate the engine's utility by training PPO and AlphaZero agents, showing that the engine's performance enables rapid experimentation."

**Why this matters:**
- **Establishes Essence Wars as a legitimate research tool** (gets cited in future papers)
- **Creates a baseline** that others will compare against

**Key contributions to highlight:**
1. ✨ **Zero-allocation design** (101ns state cloning)
2. ✨ **Deterministic execution** (perfect reproducibility)
3. ✨ **Fast Python bindings** (30-50K steps/sec expected)
4. ✨ **Complete benchmark suite** (agents, decks, matchup data)

**Where to publish:** 
- NeurIPS Datasets & Benchmarks track
- AAAI (Artifact track)
- CoRL (Conference on Robot Learning) - they love good simulators
- arXiv first, then submit to conference

**Timeline:** Write this during/after Phase 3, submit by summer 2026

---

## **Paper 2: "Weight Tuning for MCTS in Complex Card Games"** 🎲

**Type:** Algorithmic contribution (MCTS methodology)

**The pitch:**
> "We investigate weight tuning for MCTS heuristics in a 300-card strategy game. Using CMA-ES optimization over 48,000 games, we discover that tuned weights improve win rate by 40% over uniform weights, with surprising findings about the relative importance of board control vs card advantage. We provide a replicable methodology for tuning MCTS in any deterministic game."

**Why this is interesting:**
- MCTS papers are **always relevant** (used in Go, Chess, StarCraft)
- Our tuning pipeline is **actually quite sophisticated**
- The **domain (card games) is underexplored** compared to board games
- We have **real data** showing what works

**Key contributions:**
1. 📊 **CMA-ES tuning methodology** for MCTS weights
2. 📊 **Analysis of weight sensitivity** (which weights matter most?)
3. 📊 **Transfer learning** (do weights transfer across decks/matchups?)
4. 📊 **Ablation studies** (what happens if you remove each weight?)

**Cool experiments to run:**
```python
# Experiment 1: Cross-deck transfer
weights_deck_A = tune_on_deck("argentum_tokens")
test_on_deck(weights_deck_A, "obsidion_lifesteal")  # Does it transfer?

# Experiment 2: Opponent adaptation
weights_vs_greedy = tune_against("greedy")
weights_vs_mcts = tune_against("mcts_100")
# Are the optimal weights different?

# Experiment 3: Game mode transfer
weights_mode_A = tune_on_mode("essence_duel")
test_on_mode(weights_mode_A, "attrition")  # The transfer learning you designed for!
```

**Where to publish:**
- IEEE Transactions on Games
- IJCAI (International Joint Conference on AI)
- CoG (Conference on Games)
- arXiv first

**Timeline:** Write this after Phase 3, once you have PPO/AlphaZero agents to compare MCTS against

---

## **Paper 3: "Sample Efficiency in Sparse-Reward Card Games"** 🚀

**Type:** RL methodology (PPO/AlphaZero comparison)

**The pitch:**
> "Card games present unique challenges for RL: sparse rewards (only win/loss), long episodes (30+ turns), and complex state spaces (hand + board + deck). We compare PPO, DQN, and AlphaZero on Essence Wars, showing that AlphaZero achieves 65% win rate vs MCTS with 10x fewer training games than PPO. We analyze the sample efficiency gap and propose a hybrid approach combining imitation learning from MCTS with self-play refinement."

**Why this is a strong contribution:**
- **Sample efficiency is THE hot topic** in RL right now
- Card games are **harder than people think** (less studied than Atari/MuJoCo)
- We can **directly compare** multiple algorithms (fair comparison, same engine)
- **AlphaZero in card games** is relatively unexplored (most work is on board games)

**Key experiments:**
```text
┌──────────────────────────────────────────────┐
│  Algorithm Comparison (1M training steps)    │
├──────────────────────────────────────────────┤
│  PPO:         45% vs MCTS-100                │
│  AlphaZero:   65% vs MCTS-100                │
│  PPO+MCTS:    58% vs MCTS-100  (hybrid!)     │
└──────────────────────────────────────────────┘

Question: Why does AlphaZero do better?
- Better exploration via MCTS tree search
- Self-play creates diverse opponents
- Value network learns faster than policy-only

Question: Can we bootstrap PPO with MCTS demonstrations?
- Pre-train on 10K MCTS games
- Then self-play to refine
- Beats pure PPO with 3x less data
```

**Where to publish:**
- NeurIPS (main track)
- ICML (International Conference on Machine Learning)
- ICLR (International Conference on Learning Representations)

**Timeline:** This is your **Phase 4 flagship paper** (late 2026/early 2027)
