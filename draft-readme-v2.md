This is a fantastic direction, Christian! 🚀 The **Monorepo** approach is absolutely the right call—it keeps your velocity high and ensures your "Game" and "Engine" never drift out of sync. And I love the clarification on the "Private Research" boundary; publishing papers while keeping the code open is the true spirit of science (even if the paper churn is high, *quality* stands out!).

The "Briefing Document" mixed with "Clean OSS" is a fun balance to strike. It gives the project personality without scaring off the academics.

Here is a draft for your new **Game-First README**. I have restructured it to hook gamers/developers first, while keeping the heavy research specs impressive but secondary.

## Proposed `README.md` Redesign

```markdown
# Essence Wars

**A Strategic Card Game Powered by Reinforcement Learning**

[![PyPI](https://img.shields.io/pypi/v/essence-wars?style=for-the-badge&logo=pypi&logoColor=white)](https://pypi.org/project/essence-wars/)
[![Python](https://img.shields.io/pypi/pyversions/essence-wars?style=for-the-badge&logo=python&logoColor=white)](https://pypi.org/project/essence-wars/)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Playable_Alpha-blue?style=for-the-badge)](https://huggingface.co/spaces/christianwissmann85/essence-wars)

---

## 🛑 Incoming Transmission: The Farsight Table

> *"Commander, the simulation is ready. The enemy isn't following a script anymore—they are thinking."*

**Essence Wars** is a high-performance digital card game where you battle against AI opponents powered by real Reinforcement Learning (PPO & AlphaZero) and Monte Carlo Tree Search (MCTS).

Unlike traditional card games with scripted bots, our AI "Agents" learn strategies from millions of self-play games. We provide the **Glassbox Mode** to let you see exactly what they are thinking in real-time.

### 🎮 [**PLAY THE WEB ALPHA**](https://huggingface.co/spaces/christianwissmann85/essence-wars)
*(Instant play in browser • No installation required • Powered by WebAssembly)*

---

## 🌟 Key Features

### 🧠 True AI Opponents
Forget "hard coded" difficulty. Face off against agents trained on our custom Rust engine:
* **The Swarm (PPO):** A neural network trained via Proximal Policy Optimization. Aggressive and unpredictable.
* **The Tactician (MCTS):** A tree-search bot that simulates thousands of future timelines per second to find the optimal move.

### 🔮 Glassbox Visualization
Don't just lose—learn *why*. Toggle **Glassbox Mode** to visualize the AI's decision-making process:
* See the **Action Probability** bars shifting in real-time.
* Watch **Ghost Arrows** map out the MCTS thought paths on the board.
* Understand the **Value Function** (win probability) as the tide of battle turns.

### ⚔️ The Factions (New Horizons Set)
Master 300 cards across three distinct playstyles:
* **Argentum Combine ("The Wall"):** Defensive constructs and high-HP guards.
* **Symbiote Circles ("The Swarm"):** Aggressive tempo and sacrificial death triggers.
* **Obsidion Syndicate ("The Shadow"):** Burst damage, lifesteal, and stealth assassins.

---

## 🏗️ Under the Hood: The Engine

For developers and researchers, **Essence Wars** is built on a high-performance, deterministic Rust core. It serves as a gym-compatible environment for RL research.

| Metric | Performance |
|--------|-------------|
| **Latency** | ~133 ns state access |
| **Throughput** | ~80,000 games/sec (Random) |
| **Vectorized** | ~268,000 steps/sec (64 envs) |
| **Architecture** | Rust Core + Bevy (3D Client) + Python (Training) |

---

## 🧪 Powered by Open Source Research

Essence Wars doubles as a research platform. You can train your own agents or use our gym environment to benchmark new algorithms.

### Python Quickstart (Gymnasium)

```python
import gymnasium as gym
from essence_wars import EssenceWarsEnv

# Initialize the environment against a pre-trained MCTS opponent
env = EssenceWarsEnv(opponent="mcts_medium")
obs, info = env.reset(seed=42)

while True:
    # Your custom agent logic here
    action = my_agent.select_action(obs, info["action_mask"])
    obs, reward, terminated, truncated, info = env.step(action)
    
    if terminated:
        print(f"Game Over. Reward: {reward}")
        break

```

### Reproducible Research

We publish our training methodologies and model weights openly.

* **[Training Dashboard](https://christianWissmann85.github.io/essence-wars/dashboard/training.html)**: View loss curves and PPO convergence.
* **[Balance Dashboard](https://christianWissmann85.github.io/essence-wars/dashboard/index.html)**: Live faction win-rate statistics.

---

## 🛠️ Installation & Development

### For Players (Rust Client)

To build the native Windows/Linux client (Game Mode):

```bash
# Clone the repository
git clone [https://github.com/christianWissmann85/essence-wars](https://github.com/christianWissmann85/essence-wars)
cd essence-wars

# Run the 3D Game Client
cargo run --release --bin essence-wars-3d

```

### For Researchers (Python Package)

To install the engine for ML experiments:

```bash
pip install essence-wars[train]

```

---

## 🗺️ Roadmap

* **Phase 1-4:** Core Engine, MCTS/PPO Agents, Gym Interface ✅
* **Phase 5A:** **Web Arena (WASM)** - Playable browser version 🔄 *(In Progress)*
* **Phase 5B:** **JRPG Campaign** - Story mode with overworld exploration 📋
* **Phase 6:** Multiplayer & Community Features 📋

---

## 📄 Citation

If you use the Essence Wars engine in your research, please cite:

```bibtex
@software{essence_wars,
  title = {Essence Wars: A Strategic Card Game & Deterministic Engine},
  author = {Wissmann, Christian},
  year = {2026},
  url = {[https://github.com/christianWissmann85/essence-wars](https://github.com/christianWissmann85/essence-wars)}
}

```

---

## License

MIT License. Assets/Art may have separate licenses (see `assets/LICENSE`).

```

### 👨‍💻 Why this structure works:

1.  **The "Briefing" Hook**: The quote at the top immediately sets the tone. It tells the user this is a *game*, not just a repo.
2.  **The "Play Now" Button**: This is your conversion funnel. By Phase 5A, having this link front-and-center is crucial.
3.  **Features First**: We talk about "True AI" and "Glassbox" as gameplay features, not research specs.
4.  **Research Section (Option B)**: It is still there, fully detailed, but it is framed as "Powered by..."—giving credibility to the game rather than dominating the intro.
5.  **Monorepo Friendly**: The Installation section now clearly splits "For Players (Cargo)" and "For Researchers (Pip)", which fits your directory structure perfectly.

How does this feel? If you are happy with it, you can swap this in and we can officially mark the "Pivot" as begun! 🥂
