# Essence Wars Agent Leaderboard

> Last updated: 2026-01-19
> Benchmark version: 1.0

## Rankings

| Rank | Agent | Author | Type | Elo | vs Greedy | vs Random |
|------|-------|--------|------|-----|-----------|-----------|
| 1 | [PPO-Argentum](https://huggingface.co/Chris-Essence-Wars/ppo-argentum) | Chris-Essence-Wars | PPO | 1450 | 72% | 99% |
| 2 | [PPO-Flat](https://huggingface.co/Chris-Essence-Wars/ppo-flat) | Chris-Essence-Wars | PPO | 1440 | 71% | 99% |
| 3 | [PPO-Embedded](https://huggingface.co/Chris-Essence-Wars/ppo-embedded) | Chris-Essence-Wars | PPO | 1400 | 65% | 98% |
| 4 | [PPO-Symbiote](https://huggingface.co/Chris-Essence-Wars/ppo-symbiote) | Chris-Essence-Wars | PPO | 1400 | 65% | 98% |
| 5 | [PPO-Obsidion](https://huggingface.co/Chris-Essence-Wars/ppo-obsidion) | Chris-Essence-Wars | PPO | 1380 | 62% | 97% |
| 6 | BC-MCTS-10k | Chris-Essence-Wars | BC | 1360 | 59% | 96% |

## Baselines

| Agent | Type | Elo | vs Greedy | Description |
|-------|------|-----|-----------|-------------|
| MCTS-100 | MCTS | 1500 | 65% | Monte Carlo Tree Search with 100 simulations per move |
| MCTS-50 | MCTS | 1450 | 58% | Monte Carlo Tree Search with 50 simulations per move |
| GreedyBot | HEURISTIC | 1300 | 50% | Heuristic evaluation with 24 tuned weights |
| RandomBot | RANDOM | 1000 | 5% | Uniform random action selection |

## How to Submit

1. Train your agent using `essence-wars` package
2. Run submission script:
   ```bash
   python scripts/submit_agent.py \
       --checkpoint your_model.pt \
       --name "My Agent" \
       --repo-id yourusername/essence-wars-agent
   ```
3. Your agent will be evaluated and added to the leaderboard

See [SUBMIT_AGENT.md](./SUBMIT_AGENT.md) for detailed instructions.

## Evaluation Methodology

- **Games per opponent**: 100
- **Game mode**: attrition
- **Baselines**: random, greedy, mcts50, mcts100
- **Elo calculation**: Standard Elo with K=32

---

*Generated automatically from [leaderboard.json](../data/leaderboard/leaderboard.json)*