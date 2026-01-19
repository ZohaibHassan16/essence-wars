---
title: Essence Wars Leaderboard
emoji: 🎮
colorFrom: purple
colorTo: blue
sdk: gradio
sdk_version: 5.9.1
app_file: app.py
pinned: true
license: mit
tags:
  - reinforcement-learning
  - card-game
  - leaderboard
  - benchmark
---

# Essence Wars Agent Leaderboard

Interactive leaderboard for comparing AI agents trained on [Essence Wars](https://github.com/christianWissmann85/essence-wars), a deterministic card game designed for reinforcement learning research.

## Features

- **Sortable Rankings**: Compare agents by Elo rating and win rates
- **Filter by Type**: View only PPO, AlphaZero, or other model types
- **Agent Details**: Click on any agent to see detailed performance metrics
- **Direct Downloads**: Links to download models from HuggingFace Hub

## Baselines

| Bot | Elo | Description |
|-----|-----|-------------|
| RandomBot | 1000 | Uniform random action selection |
| GreedyBot | 1300 | Heuristic evaluation with tuned weights |
| MCTS-50 | 1450 | Monte Carlo Tree Search, 50 simulations |
| MCTS-100 | 1500 | Monte Carlo Tree Search, 100 simulations |

## Submit Your Agent

Train an agent and submit it to the leaderboard:

```bash
pip install essence-wars

# Train
python -m essence_wars.scripts.train_ppo --timesteps 300000

# Submit
python python/scripts/submit_agent.py \
    --checkpoint models/my_agent.pt \
    --name "My Agent" \
    --repo-id yourusername/essence-wars-agent
```

See the [submission guide](https://github.com/christianWissmann85/essence-wars/blob/master/docs/SUBMIT_AGENT.md) for details.

## Links

- [GitHub Repository](https://github.com/christianWissmann85/essence-wars)
- [PyPI Package](https://pypi.org/project/essence-wars/)
- [Documentation](https://github.com/christianWissmann85/essence-wars/tree/master/docs)
