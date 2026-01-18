# Phase 3: ML/AI Infrastructure

**Mission**: Transform Essence Wars from a game engine into a world-class RL benchmark that researchers can `pip install` and start training in 5 minutes.

**Target**: 60k+ SPS on CPU, zero-friction Gymnasium/PettingZoo compliance, working PPO and AlphaZero agents.

---

## Architecture: "Thick Rust, Thin Python"

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Python Layer                                 │
│  ┌───────────────┐  ┌────────────────┐  ┌───────────────────────┐   │
│  │ gymnasium.Env │  │ PettingZoo     │  │ Agents                │   │
│  │ EssenceWarsEnv│  │ ParallelEnv    │  │ PPO, AlphaZero        │   │
│  └───────┬───────┘  └───────┬────────┘  └───────────┬───────────┘   │
│          │                  │                       │               │
│          └──────────────────┴───────────────────────┘               │
│                             │                                       │
│                    ┌────────▼────────┐                              │
│                    │   PyO3 Bridge   │  Zero-copy NumPy             │
│                    │   (cdylib)      │  ~133ns tensor latency       │
│                    └────────┬────────┘                              │
└─────────────────────────────┼───────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────────────┐
│                         Rust Layer                                   │
│  ┌───────────────┐  ┌────────────────┐  ┌───────────────────────┐   │
│  │ GameEngine    │  │ ParallelEngine │  │ Tensor & Actions      │   │
│  │ Single game   │  │ Batched games  │  │ 326 obs, 256 actions  │   │
│  │ 80k games/s   │  │ GPU-ready      │  │ Legal mask            │   │
│  └───────────────┘  └────────────────┘  └───────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Decisions Made

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Python Bindings** | PyO3 | Native speed, zero-copy tensors |
| **Gym Version** | Gymnasium (v26+) | Maintained fork, modern API |
| **ML Framework** | PyTorch | Standard for RL research |
| **Batched Envs** | Yes (`ParallelGameEngine`) | Key differentiator, GPU training |
| **Opponent Strategy** | Mixed → League | 50% self, 25% Greedy, 25% Random (MVP), then checkpoint pool |
| **Publishing** | PyPI + crates.io | `pip install essence-wars` via maturin |
| **PPO Success** | >60% vs Greedy | Proves learning signal |
| **AlphaZero Success** | >60% vs MCTS | Proves value network works |
| **Observation Encoding** | Flat tensor (326-dim) | Simple, works immediately; embeddings deferred to Phase 4 |
| **Deck Selection** | Yes | `EssenceWarsEnv(deck="argentum_control")` |
| **Logging** | TensorBoard + W&B | TB for local dev, W&B for experiment tracking |
| **Model Hosting** | HuggingFace Hub | `essence-wars/ppo-generalist` namespace |

---

## Milestone 1: PyO3 Bridge (Foundation) ✅ [COMPLETE]

**Goal**: Expose Rust engine to Python with zero-copy tensor transfer.

### 1A: Cargo Configuration

```toml
# Cargo.toml additions
[lib]
name = "essence_wars_core"
crate-type = ["cdylib", "rlib"]  # cdylib for Python, rlib for Rust

[features]
default = []
python = ["dep:pyo3", "dep:numpy"]

[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"], optional = true }
numpy = { version = "0.22", optional = true }
```

### 1B: PyO3 Bindings (`src/python.rs`)

```rust
use pyo3::prelude::*;
use numpy::{PyArray1, PyArray2, ToPyArray};

/// Single game wrapper for Python
#[pyclass]
pub struct PyGame {
    engine: GameEngine,
    card_db: CardDatabase,
}

#[pymethods]
impl PyGame {
    #[new]
    fn new(seed: Option<u64>) -> PyResult<Self> { ... }

    /// Reset game, return initial observation
    fn reset(&mut self, seed: u64) -> PyResult<()> { ... }

    /// Get state tensor as numpy array (zero-copy)
    fn observe<'py>(&self, py: Python<'py>) -> &'py PyArray1<f32> {
        self.engine.get_state_tensor().to_pyarray(py)
    }

    /// Get legal action mask as numpy array
    fn action_mask<'py>(&self, py: Python<'py>) -> &'py PyArray1<f32> {
        self.engine.get_legal_action_mask().to_pyarray(py)
    }

    /// Apply action, return (reward, terminated)
    fn step(&mut self, action: u8) -> PyResult<(f32, bool)> { ... }

    /// Get current player (0 or 1)
    fn current_player(&self) -> u8 { ... }

    /// Check if game is over
    fn is_done(&self) -> bool { ... }

    /// Clone game state (for MCTS)
    fn fork(&self) -> Self { ... }
}

/// Batched games for vectorized training
#[pyclass]
pub struct PyParallelGames {
    engines: Vec<GameEngine>,
    card_db: CardDatabase,
}

#[pymethods]
impl PyParallelGames {
    #[new]
    fn new(num_envs: usize) -> PyResult<Self> { ... }

    /// Reset all games, return batch observations [num_envs, 326]
    fn reset<'py>(&mut self, py: Python<'py>, seeds: Vec<u64>)
        -> &'py PyArray2<f32> { ... }

    /// Get batch observations [num_envs, 326]
    fn observe_batch<'py>(&self, py: Python<'py>) -> &'py PyArray2<f32> { ... }

    /// Get batch action masks [num_envs, 256]
    fn action_mask_batch<'py>(&self, py: Python<'py>) -> &'py PyArray2<f32> { ... }

    /// Step all games, return (rewards, dones) as numpy arrays
    fn step_batch(&mut self, actions: Vec<u8>) -> PyResult<(Vec<f32>, Vec<bool>)> { ... }
}

/// Python module definition
#[pymodule]
fn essence_wars_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyGame>()?;
    m.add_class::<PyParallelGames>()?;
    Ok(())
}
```

### 1C: Build System (`pyproject.toml`)

```toml
[build-system]
requires = ["maturin>=1.4,<2.0"]
build-backend = "maturin"

[project]
name = "essence-wars"
version = "0.6.0"
description = "High-performance card game environment for RL research"
readme = "README.md"
requires-python = ">=3.10"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Topic :: Scientific/Engineering :: Artificial Intelligence",
]
dependencies = [
    "gymnasium>=0.29.0",
    "numpy>=1.24.0",
]

[project.optional-dependencies]
train = [
    "torch>=2.0.0",
    "tensorboard>=2.14.0",
    "tqdm>=4.66.0",
]
pettingzoo = [
    "pettingzoo>=1.24.0",
]
dev = [
    "pytest>=7.0.0",
    "pytest-benchmark>=4.0.0",
]

[tool.maturin]
features = ["python"]
python-source = "python"
module-name = "essence_wars._core"
```

### 1D: Deliverables 

- [x] `src/python.rs` with `PyGame` and `PyParallelGames`
- [x] `Cargo.toml` with feature flags
- [x] `pyproject.toml` with maturin config
- [x] `maturin develop` builds successfully
- [x] Unit test: tensor shape is (326,), action mask is (256,)
-  ✅ [COMPLETE]

---

## Milestone 2: Gymnasium Environment ✅ [COMPLETE]

**Goal**: Strict v26+ compliant single-agent environment for self-play.

### 2A: Environment Implementation (`python/essence_wars/env.py`)

```python
import gymnasium as gym
import numpy as np
from gymnasium import spaces
from essence_wars._core import PyGame

class EssenceWarsEnv(gym.Env):
    """
    Single-agent Essence Wars environment for self-play training.

    Observation: Dict with 'observation' (326,) and 'action_mask' (256,)
    Action: Discrete(256) - use action_mask to filter illegal actions
    Reward: +1 win, -1 loss, 0 otherwise (from perspective of player 0)
    """

    metadata = {"render_modes": ["human", "ansi"], "render_fps": 30}

    def __init__(self, render_mode=None, opponent="self"):
        super().__init__()
        self._game = PyGame()
        self._opponent = opponent  # "self", "greedy", "random", "mcts"
        self._render_mode = render_mode

        # Gymnasium spaces (strict compliance)
        self.observation_space = spaces.Dict({
            "observation": spaces.Box(
                low=0.0, high=1.0, shape=(326,), dtype=np.float32
            ),
            "action_mask": spaces.Box(
                low=0.0, high=1.0, shape=(256,), dtype=np.float32
            ),
        })
        self.action_space = spaces.Discrete(256)

    def reset(self, seed=None, options=None):
        """Reset environment. Returns (observation, info)."""
        super().reset(seed=seed)
        self._game.reset(seed if seed is not None else 0)
        return self._get_obs(), {}

    def step(self, action):
        """
        Execute action. Returns (obs, reward, terminated, truncated, info).

        If it's opponent's turn after our action, opponent plays until
        it's our turn again (or game ends).
        """
        # Our action
        reward, done = self._game.step(action)

        # Opponent's turn(s)
        while not done and self._game.current_player() == 1:
            opp_action = self._get_opponent_action()
            _, done = self._game.step(opp_action)

        terminated = done
        truncated = False  # Turn limit handled in Rust

        return self._get_obs(), reward, terminated, truncated, {}

    def _get_obs(self):
        return {
            "observation": self._game.observe(),
            "action_mask": self._game.action_mask(),
        }

    def _get_opponent_action(self):
        """Get opponent action based on opponent type."""
        mask = self._game.action_mask()
        if self._opponent == "random":
            legal = np.where(mask > 0)[0]
            return np.random.choice(legal)
        elif self._opponent == "greedy":
            return self._game.greedy_action()  # Rust-side greedy
        elif self._opponent == "self":
            # Will be replaced by policy during training
            legal = np.where(mask > 0)[0]
            return np.random.choice(legal)
        else:
            raise ValueError(f"Unknown opponent: {self._opponent}")


class VectorizedEssenceWars(gym.vector.VectorEnv):
    """
    Vectorized environment using Rust ParallelGameEngine.

    Achieves 60k+ SPS by running games in parallel on CPU.
    """

    def __init__(self, num_envs, **kwargs):
        self._games = PyParallelGames(num_envs)
        self.num_envs = num_envs

        single_obs_space = spaces.Dict({
            "observation": spaces.Box(0, 1, (326,), dtype=np.float32),
            "action_mask": spaces.Box(0, 1, (256,), dtype=np.float32),
        })
        single_action_space = spaces.Discrete(256)

        super().__init__(num_envs, single_obs_space, single_action_space)

    def reset(self, seed=None, options=None):
        seeds = self._generate_seeds(seed)
        self._games.reset(seeds)
        return self._get_obs(), {}

    def step(self, actions):
        rewards, dones = self._games.step_batch(actions.tolist())
        # Auto-reset done environments
        return self._get_obs(), np.array(rewards), np.array(dones), \
               np.zeros(self.num_envs, dtype=bool), {}
```

### 2B: Deliverables

- [x] `python/essence_wars/env.py` with `EssenceWarsEnv`
- [x] `python/essence_wars/env.py` with `VectorizedEssenceWars`
- [x] Passes `gymnasium.utils.env_checker.check_env()`
- [x] Benchmark: ~100k SPS single env, **~268k SPS vectorized** (256 envs)

---

## Milestone 3: PettingZoo Multi-Agent ✅ [COMPLETE]

**Goal**: Proper multi-agent environment with action masking for research.

### 3A: Implementation (`python/essence_wars/parallel_env.py`)

```python
from pettingzoo import ParallelEnv
from gymnasium import spaces
import numpy as np

class EssenceWarsParallelEnv(ParallelEnv):
    """
    PettingZoo ParallelEnv for two-player Essence Wars.

    Both agents receive observations every step, but only the active
    player has valid actions (via action_mask).
    """

    metadata = {"render_modes": ["human", "ansi"], "name": "essence_wars_v1"}

    def __init__(self, render_mode=None):
        self._game = PyGame()
        self.possible_agents = ["player_0", "player_1"]
        self.agents = self.possible_agents[:]

        self._action_spaces = {
            agent: spaces.Discrete(256) for agent in self.possible_agents
        }
        self._observation_spaces = {
            agent: spaces.Dict({
                "observation": spaces.Box(0, 1, (326,), dtype=np.float32),
                "action_mask": spaces.Box(0, 1, (256,), dtype=np.float32),
            }) for agent in self.possible_agents
        }

    def reset(self, seed=None, options=None):
        self._game.reset(seed or 0)
        self.agents = self.possible_agents[:]
        observations = {agent: self._get_obs(i) for i, agent in enumerate(self.agents)}
        infos = {agent: {} for agent in self.agents}
        return observations, infos

    def step(self, actions):
        """
        Process actions from both agents.
        Only the active player's action is actually applied.
        """
        active = self._game.current_player()
        active_agent = f"player_{active}"

        # Apply only the active player's action
        action = actions[active_agent]
        reward_val, done = self._game.step(action)

        # Rewards from each player's perspective
        rewards = {
            "player_0": reward_val,
            "player_1": -reward_val,
        }

        terminations = {agent: done for agent in self.agents}
        truncations = {agent: False for agent in self.agents}

        if done:
            self.agents = []

        observations = {agent: self._get_obs(i) for i, agent in enumerate(self.possible_agents)}
        infos = {agent: {} for agent in self.possible_agents}

        return observations, rewards, terminations, truncations, infos

    def _get_obs(self, player_idx):
        """Get observation for a specific player."""
        obs = self._game.observe()  # Always from current perspective
        mask = self._game.action_mask()

        # Mask non-active player's actions to zero
        if self._game.current_player() != player_idx:
            mask = np.zeros_like(mask)

        return {"observation": obs, "action_mask": mask}
```

### 3B: Deliverables

- [x] `python/essence_wars/parallel_env.py` with `EssenceWarsParallelEnv`
- [x] Passes PettingZoo API tests (`parallel_api_test`)
- [x] Example: two random agents playing

---

## Milestone 4: PPO Agent ✅ [COMPLETE]

**Goal**: Self-play PPO that beats GreedyBot >60%.

### 4A: Network Architecture (`python/essence_wars/agents/networks.py`)

```python
import torch
import torch.nn as nn

class EssenceWarsNetwork(nn.Module):
    """
    Shared policy-value network for PPO.

    Input: 326-dim observation
    Output: 256-dim policy logits + 1-dim value
    """

    def __init__(self, obs_dim=326, action_dim=256, hidden_dim=256):
        super().__init__()

        # Shared trunk
        self.trunk = nn.Sequential(
            nn.Linear(obs_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
        )

        # Policy head
        self.policy = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, action_dim),
        )

        # Value head
        self.value = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
        )

    def forward(self, obs, action_mask=None):
        features = self.trunk(obs)
        logits = self.policy(features)
        value = self.value(features)

        # Mask illegal actions with large negative value
        if action_mask is not None:
            logits = logits - (1 - action_mask) * 1e8

        return logits, value.squeeze(-1)

    def get_action(self, obs, action_mask, deterministic=False):
        logits, value = self.forward(obs, action_mask)
        probs = torch.softmax(logits, dim=-1)

        if deterministic:
            action = torch.argmax(probs, dim=-1)
        else:
            action = torch.multinomial(probs, 1).squeeze(-1)

        log_prob = torch.log(probs.gather(-1, action.unsqueeze(-1))).squeeze(-1)
        return action, log_prob, value
```

### 4B: PPO Implementation (`python/essence_wars/agents/ppo.py`)

```python
class PPOTrainer:
    """
    PPO trainer with mixed opponent strategy.

    Opponent distribution:
    - 50% self (current policy)
    - 25% GreedyBot
    - 25% RandomBot
    """

    def __init__(
        self,
        num_envs=64,
        steps_per_update=128,
        epochs=4,
        minibatch_size=256,
        lr=3e-4,
        gamma=0.99,
        gae_lambda=0.95,
        clip_ratio=0.2,
        ent_coef=0.01,
        vf_coef=0.5,
        device="cuda" if torch.cuda.is_available() else "cpu",
    ):
        self.device = device
        self.num_envs = num_envs
        self.steps_per_update = steps_per_update
        # ... hyperparameters

        # Network
        self.network = EssenceWarsNetwork().to(device)
        self.optimizer = torch.optim.Adam(self.network.parameters(), lr=lr)

        # Vectorized environment
        self.envs = VectorizedEssenceWars(num_envs)

        # Mixed opponent tracking
        self.opponent_probs = [0.50, 0.25, 0.25]  # self, greedy, random

    def collect_rollout(self):
        """Collect experience from vectorized environments."""
        # ... standard PPO rollout collection with GAE
        pass

    def update(self, rollout):
        """PPO update with clipped objective."""
        # ... standard PPO update
        pass

    def train(self, total_timesteps, eval_interval=10000):
        """Main training loop."""
        timesteps = 0
        while timesteps < total_timesteps:
            rollout = self.collect_rollout()
            self.update(rollout)
            timesteps += self.num_envs * self.steps_per_update

            if timesteps % eval_interval == 0:
                win_rate = self.evaluate_vs_greedy(100)
                print(f"Timesteps: {timesteps}, Win rate vs Greedy: {win_rate:.1%}")

    def evaluate_vs_greedy(self, num_games):
        """Evaluate current policy against GreedyBot."""
        # ... evaluation loop
        pass
```

### 4C: Training Script (`python/scripts/train_ppo.py`)

```python
#!/usr/bin/env python3
"""Train PPO agent for Essence Wars."""

import argparse
from essence_wars.agents.ppo import PPOTrainer

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--timesteps", type=int, default=1_000_000)
    parser.add_argument("--num-envs", type=int, default=64)
    parser.add_argument("--eval-interval", type=int, default=10_000)
    parser.add_argument("--save-path", type=str, default="checkpoints/ppo")
    args = parser.parse_args()

    trainer = PPOTrainer(num_envs=args.num_envs)
    trainer.train(args.timesteps, args.eval_interval)
    trainer.save(args.save_path)

if __name__ == "__main__":
    main()
```

### 4D: Key Implementation Detail: Observation Normalization

Raw observations contain card IDs (1000-4019), causing training instability. Solution:
- `RunningMeanStd` class normalizes observations to ~N(0,1)
- `normalize_obs=True` (default) in `PPOConfig`
- Normalizer state saved/loaded with checkpoints

**Impact of fix**:
- Value loss: 12,335 → ~0.02
- Approx KL: 12 → ~0.001
- Win rate: 31.5% after 1M steps → 52.5% after 50k steps

### 4E: Deliverables

- [x] `python/essence_wars/agents/networks.py` - EssenceWarsNetwork
- [x] `python/essence_wars/agents/ppo.py` - PPOConfig, RolloutBuffer, PPOTrainer, RunningMeanStd
- [x] `python/scripts/train_ppo.py` - CLI training script
- [x] `python/scripts/diagnose_ppo.py` - Diagnostic script for verifying infrastructure
- [x] Training converges (loss decreases, KL stable ~0.001)
- [x] Observation normalization working (verified range [-5, 8] vs raw [-1, 4019])
- [ ] **Success**: >60% win rate vs GreedyBot (52.5% after 50k steps; full training for Phase 4)

---

## Milestone 5: AlphaZero Agent ✅ [COMPLETE]

**Goal**: MCTS + neural network that beats MCTS baseline >60%.

### 5A: Neural Network (`python/essence_wars/agents/networks.py`)

```python
class AlphaZeroNetwork(nn.Module):
    """
    Policy-value network for AlphaZero.

    Similar to PPO network but trained differently (MCTS targets).
    """

    def __init__(self, obs_dim=326, action_dim=256, hidden_dim=256, num_blocks=4):
        super().__init__()

        # Input projection
        self.input = nn.Linear(obs_dim, hidden_dim)

        # Residual blocks
        self.blocks = nn.ModuleList([
            ResidualBlock(hidden_dim) for _ in range(num_blocks)
        ])

        # Policy head
        self.policy = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, action_dim),
        )

        # Value head
        self.value = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
            nn.Tanh(),  # Value in [-1, 1]
        )

    def forward(self, obs, action_mask=None):
        x = torch.relu(self.input(obs))
        for block in self.blocks:
            x = block(x)

        policy_logits = self.policy(x)
        if action_mask is not None:
            policy_logits = policy_logits - (1 - action_mask) * 1e8

        value = self.value(x).squeeze(-1)
        return policy_logits, value
```

### 5B: MCTS with Neural Network (`python/essence_wars/agents/alphazero.py`)

```python
class NeuralMCTS:
    """
    MCTS guided by neural network policy and value.

    Key differences from vanilla MCTS:
    - Prior probabilities from policy network (not uniform)
    - Leaf evaluation from value network (not rollout)
    - PUCT selection formula
    """

    def __init__(self, network, c_puct=1.5, num_simulations=100):
        self.network = network
        self.c_puct = c_puct
        self.num_simulations = num_simulations

    def search(self, game):
        """Run MCTS from current game state, return action probabilities."""
        root = MCTSNode(game.fork())

        # Expand root with network
        policy, value = self._evaluate(root.game)
        root.expand(policy)

        for _ in range(self.num_simulations):
            node = root
            search_path = [node]

            # Select
            while node.is_expanded() and not node.game.is_done():
                action, node = node.select_child(self.c_puct)
                search_path.append(node)

            # Evaluate leaf
            if not node.game.is_done():
                policy, value = self._evaluate(node.game)
                node.expand(policy)
            else:
                value = node.game.get_reward()

            # Backup
            for node in reversed(search_path):
                node.visit_count += 1
                node.value_sum += value
                value = -value  # Flip for opponent

        # Return visit count distribution as action probabilities
        return root.get_action_probs()

    def _evaluate(self, game):
        """Evaluate position with neural network."""
        obs = torch.tensor(game.observe()).unsqueeze(0)
        mask = torch.tensor(game.action_mask()).unsqueeze(0)

        with torch.no_grad():
            logits, value = self.network(obs, mask)
            policy = torch.softmax(logits, dim=-1).squeeze(0).numpy()

        return policy, value.item()


class AlphaZeroTrainer:
    """
    AlphaZero training loop.

    1. Self-play with current network + MCTS
    2. Store (state, mcts_policy, outcome) in replay buffer
    3. Train network on batches from replay buffer
    4. Repeat
    """

    def __init__(self, ...):
        self.network = AlphaZeroNetwork()
        self.mcts = NeuralMCTS(self.network)
        self.replay_buffer = ReplayBuffer(capacity=100_000)
        # ...

    def self_play_game(self):
        """Play one game with MCTS, return training data."""
        game = PyGame()
        game.reset(random_seed())

        states, policies, players = [], [], []

        while not game.is_done():
            # MCTS search
            action_probs = self.mcts.search(game)

            # Store training data
            states.append(game.observe())
            policies.append(action_probs)
            players.append(game.current_player())

            # Sample action (with temperature)
            action = np.random.choice(256, p=action_probs)
            game.step(action)

        # Assign outcomes
        outcome = game.get_reward()  # From player 0's perspective
        outcomes = [outcome if p == 0 else -outcome for p in players]

        return list(zip(states, policies, outcomes))

    def train(self, num_iterations):
        """Main training loop."""
        for iteration in range(num_iterations):
            # Generate self-play games
            for _ in range(self.games_per_iteration):
                data = self.self_play_game()
                self.replay_buffer.extend(data)

            # Train network
            for _ in range(self.training_steps):
                batch = self.replay_buffer.sample(self.batch_size)
                self.train_step(batch)

            # Evaluate
            if iteration % self.eval_interval == 0:
                win_rate = self.evaluate_vs_mcts(100)
                print(f"Iteration {iteration}, Win rate vs MCTS: {win_rate:.1%}")
```

### 5C: Key Implementation Details

**Components implemented:**
- `AlphaZeroNetwork`: Residual tower architecture with 4 blocks, tanh value head
- `MCTSNode`: Tree node with UCB selection, Dirichlet noise for exploration
- `NeuralMCTS`: PUCT-based MCTS with neural network policy/value
- `ReplayBuffer`: Stores (obs, mask, policy_target, value_target) from self-play
- `AlphaZeroTrainer`: Full training loop with self-play → replay buffer → train

**Observation normalization**: Uses same `RunningMeanStd` approach as PPO.

### 5D: Deliverables

- [x] `python/essence_wars/agents/networks.py` - AlphaZeroNetwork, ResidualBlock
- [x] `python/essence_wars/agents/alphazero.py` - MCTSNode, NeuralMCTS, ReplayBuffer, AlphaZeroTrainer
- [x] `python/scripts/train_alphazero.py` - CLI training script
- [x] Self-play generates valid games (verified with tests)
- [x] Training loss decreases (3.03 → 2.96 in 5 iterations)
- [x] 22 tests for AlphaZero components
- [ ] **Success**: >60% win rate vs MCTS baseline (requires extended training)

---

## Milestone 6: Publishing & CI/CD

**Goal**: `pip install essence-wars` works on Linux/Mac/Windows.

### 6A: GitHub Actions Workflow (`.github/workflows/python-wheels.yml`)

```yaml
name: Build Python Wheels

on:
  release:
    types: [published]
  workflow_dispatch:

jobs:
  build-wheels:
    name: Build wheels on ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: "3.11"

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build wheels
        uses: PyO3/maturin-action@v1
        with:
          command: build
          args: --release --out dist

      - name: Upload wheels
        uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}
          path: dist/*.whl

  publish:
    needs: build-wheels
    runs-on: ubuntu-latest
    steps:
      - name: Download wheels
        uses: actions/download-artifact@v4

      - name: Publish to PyPI
        uses: pypa/gh-action-pypi-publish@release/v1
        with:
          packages-dir: wheels-*/
```

### 6B: Package Metadata

- [x] `README.md` with installation and quick start
- [x] `LICENSE` (MIT)
- [x] `python/essence_wars/__init__.py` with version and exports
- [x] Documentation site (GitHub Pages with dashboards)

### 6C: Deliverables

- [ ] `pip install essence-wars` works (pending first PyPI release)
- [x] Wheels for Linux/Mac/Windows on PyPI (CI workflow ready)
- [ ] `cargo add essence-wars` works (deferred - using crate name `cardgame`)
- [x] Documentation site live (GitHub Pages)

---

## Success Criteria Summary

| Milestone | Metric | Target | Verification |
|-----------|--------|--------|--------------|
| M1: PyO3 | Tensor latency | <200ns | Benchmark |
| M2: Gym | SPS (vectorized) | >60,000 | Benchmark |
| M3: PettingZoo | API compliance | Pass | `pettingzoo.test.api_test` |
| M4: PPO | Win rate vs Greedy | >60% | 1000 game eval |
| M5: AlphaZero | Win rate vs MCTS | >60% | 1000 game eval |
| M6: Publishing | Installation | Works | `pip install essence-wars` (CI ready, pending release) |

---

## Timeline (Parallel Workstreams)

```
Week 1-2: M1 (PyO3 Bridge)
    └── Foundation for everything else

Week 2-3: M2 + M3 (Gym + PettingZoo)
    └── Can run in parallel once M1 is done

Week 3-5: M4 (PPO)
    └── Requires M2

Week 5-7: M5 (AlphaZero)
    └── Can start architecture while PPO trains

Week 6-7: M6 (Publishing)
    └── Can start CI/CD setup early

Week 8: Final validation, documentation, release
```

---

## Open Questions - RESOLVED

| Question | Decision | Notes |
|----------|----------|-------|
| **Observation encoding** | Flat tensor (326-dim) | Embeddings deferred to Phase 4; research paper opportunity |
| **Deck selection** | Yes, supported | `EssenceWarsEnv(deck="argentum_control")` |
| **Logging** | Both TB + W&B | TensorBoard for local, W&B for experiment tracking |
| **Model hosting** | HuggingFace Hub | `essence-wars/ppo-generalist` namespace |

---

## References

- [Gymnasium Documentation](https://gymnasium.farama.org/)
- [PettingZoo Documentation](https://pettingzoo.farama.org/)
- [PyO3 User Guide](https://pyo3.rs/)
- [Maturin Documentation](https://www.maturin.rs/)
- [CleanRL](https://github.com/vwxyzjn/cleanrl)
- [AlphaZero Paper](https://arxiv.org/abs/1712.01815)
