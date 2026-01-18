"""
Proximal Policy Optimization (PPO) for Essence Wars.

This module provides a complete PPO implementation with:
- Action masking for illegal action handling
- Mixed opponent strategy (self, greedy, random)
- Vectorized environment support for high throughput
- TensorBoard logging

Based on CleanRL's PPO implementation with modifications for card games.
"""

from __future__ import annotations

import time
from collections.abc import Callable
from dataclasses import dataclass

import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim

from essence_wars.agents.networks import EssenceWarsNetwork
from essence_wars.env import EssenceWarsEnv, VectorizedEssenceWars


class RunningMeanStd:
    """
    Running mean and standard deviation for observation normalization.

    Uses Welford's online algorithm for numerical stability.
    """

    def __init__(self, shape: tuple, epsilon: float = 1e-8):
        self.mean = np.zeros(shape, dtype=np.float64)
        self.var = np.ones(shape, dtype=np.float64)
        self.count = epsilon
        self.epsilon = epsilon

    def update(self, batch: np.ndarray) -> None:
        """Update running statistics with a batch of observations."""
        batch = np.asarray(batch)
        batch_mean = batch.mean(axis=0)
        batch_var = batch.var(axis=0)
        batch_count = batch.shape[0]

        self._update_from_moments(batch_mean, batch_var, batch_count)

    def _update_from_moments(self, batch_mean, batch_var, batch_count):
        """Update from batch statistics."""
        delta = batch_mean - self.mean
        tot_count = self.count + batch_count

        new_mean = self.mean + delta * batch_count / tot_count
        m_a = self.var * self.count
        m_b = batch_var * batch_count
        m_2 = m_a + m_b + np.square(delta) * self.count * batch_count / tot_count
        new_var = m_2 / tot_count

        self.mean = new_mean
        self.var = new_var
        self.count = tot_count

    def normalize(self, x: np.ndarray) -> np.ndarray:
        """Normalize observations using running statistics."""
        return (x - self.mean) / np.sqrt(self.var + self.epsilon)


@dataclass
class PPOConfig:
    """Configuration for PPO training."""

    # Environment
    num_envs: int = 64
    max_episode_steps: int = 500

    # Training
    total_timesteps: int = 1_000_000
    learning_rate: float = 3e-4
    num_steps: int = 128  # Steps per rollout per env
    num_epochs: int = 4  # PPO epochs per update
    num_minibatches: int = 4
    gamma: float = 0.99
    gae_lambda: float = 0.95

    # PPO specific
    clip_coef: float = 0.2
    clip_vloss: bool = True
    ent_coef: float = 0.01
    vf_coef: float = 0.5
    max_grad_norm: float = 0.5
    target_kl: float | None = None  # Early stopping KL threshold

    # Network
    hidden_dim: int = 256

    # Observation normalization
    normalize_obs: bool = True  # Use running mean/std normalization

    # Opponent strategy (for single-agent training)
    opponent_type: str = "greedy"  # "greedy", "random", or "mixed"

    # Evaluation
    eval_interval: int = 10_000
    eval_episodes: int = 100

    # Logging
    log_interval: int = 1000
    save_interval: int = 50_000

    # Device
    device: str = "cuda" if torch.cuda.is_available() else "cpu"

    @property
    def batch_size(self) -> int:
        return self.num_envs * self.num_steps

    @property
    def minibatch_size(self) -> int:
        return self.batch_size // self.num_minibatches


class RolloutBuffer:
    """Buffer for storing rollout experience."""

    def __init__(
        self,
        num_steps: int,
        num_envs: int,
        obs_dim: int,
        action_dim: int,
        device: str,
    ) -> None:
        self.num_steps = num_steps
        self.num_envs = num_envs
        self.device = device

        # Pre-allocate tensors
        self.obs = torch.zeros((num_steps, num_envs, obs_dim), device=device)
        self.actions = torch.zeros((num_steps, num_envs), dtype=torch.long, device=device)
        self.action_masks = torch.zeros((num_steps, num_envs, action_dim), dtype=torch.bool, device=device)
        self.log_probs = torch.zeros((num_steps, num_envs), device=device)
        self.rewards = torch.zeros((num_steps, num_envs), device=device)
        self.dones = torch.zeros((num_steps, num_envs), device=device)
        self.values = torch.zeros((num_steps, num_envs), device=device)

        # Computed during finalization
        self.advantages = torch.zeros((num_steps, num_envs), device=device)
        self.returns = torch.zeros((num_steps, num_envs), device=device)

        self.step = 0

    def add(
        self,
        obs: torch.Tensor,
        action: torch.Tensor,
        action_mask: torch.Tensor,
        log_prob: torch.Tensor,
        reward: torch.Tensor,
        done: torch.Tensor,
        value: torch.Tensor,
    ) -> None:
        """Add a step to the buffer."""
        self.obs[self.step] = obs
        self.actions[self.step] = action
        self.action_masks[self.step] = action_mask
        self.log_probs[self.step] = log_prob
        self.rewards[self.step] = reward
        self.dones[self.step] = done
        self.values[self.step] = value
        self.step += 1

    def compute_returns_and_advantages(
        self,
        last_value: torch.Tensor,
        gamma: float,
        gae_lambda: float,
    ) -> None:
        """Compute GAE advantages and returns."""
        last_gae = 0
        for t in reversed(range(self.num_steps)):
            if t == self.num_steps - 1:
                next_non_terminal = 1.0 - self.dones[t]
                next_value = last_value
            else:
                next_non_terminal = 1.0 - self.dones[t]
                next_value = self.values[t + 1]

            delta = self.rewards[t] + gamma * next_value * next_non_terminal - self.values[t]
            self.advantages[t] = last_gae = delta + gamma * gae_lambda * next_non_terminal * last_gae

        self.returns = self.advantages + self.values

    def get_batches(self, minibatch_size: int):
        """Yield minibatches of experience."""
        batch_size = self.num_steps * self.num_envs
        indices = np.random.permutation(batch_size)

        # Flatten tensors
        obs = self.obs.reshape(-1, self.obs.shape[-1])
        actions = self.actions.reshape(-1)
        action_masks = self.action_masks.reshape(-1, self.action_masks.shape[-1])
        log_probs = self.log_probs.reshape(-1)
        advantages = self.advantages.reshape(-1)
        returns = self.returns.reshape(-1)
        values = self.values.reshape(-1)

        for start in range(0, batch_size, minibatch_size):
            end = start + minibatch_size
            batch_indices = indices[start:end]

            yield (
                obs[batch_indices],
                actions[batch_indices],
                action_masks[batch_indices],
                log_probs[batch_indices],
                advantages[batch_indices],
                returns[batch_indices],
                values[batch_indices],
            )

    def reset(self) -> None:
        """Reset buffer for next rollout."""
        self.step = 0


class PPOTrainer:
    """
    PPO trainer for Essence Wars.

    Trains a policy network using PPO with action masking and
    mixed opponent strategy.

    Args:
        config: PPO configuration
        network: Optional pre-built network
        writer: Optional TensorBoard SummaryWriter

    Example:
        trainer = PPOTrainer(PPOConfig(num_envs=64))
        trainer.train(total_timesteps=1_000_000)

        # Save trained model
        trainer.save("checkpoints/ppo_model.pt")

        # Evaluate
        win_rate = trainer.evaluate_vs_greedy(100)
    """

    def __init__(
        self,
        config: PPOConfig | None = None,
        network: EssenceWarsNetwork | None = None,
        writer=None,
    ) -> None:
        self.config = config or PPOConfig()
        self.device = torch.device(self.config.device)

        # Create network
        if network is not None:
            self.network = network.to(self.device)
        else:
            self.network = EssenceWarsNetwork(
                hidden_dim=self.config.hidden_dim,
            ).to(self.device)

        # Optimizer
        self.optimizer = optim.Adam(
            self.network.parameters(),
            lr=self.config.learning_rate,
            eps=1e-5,
        )

        # Vectorized environment
        self.envs = VectorizedEssenceWars(
            num_envs=self.config.num_envs,
        )

        # Rollout buffer
        self.buffer = RolloutBuffer(
            num_steps=self.config.num_steps,
            num_envs=self.config.num_envs,
            obs_dim=326,
            action_dim=256,
            device=self.device,
        )

        # TensorBoard writer
        self.writer = writer

        # Observation normalizer
        self.obs_normalizer = RunningMeanStd((326,)) if self.config.normalize_obs else None

        # Training state
        self.global_step = 0
        self.start_time = None

        # Episode tracking
        self.episode_rewards = []
        self.episode_lengths = []

    def collect_rollout(self) -> dict:
        """Collect rollout experience from vectorized environments."""
        self.network.eval()
        self.buffer.reset()

        obs, masks = self.envs.reset() if self.global_step == 0 else (self._last_obs, self._last_masks)

        episode_infos = []

        for step in range(self.config.num_steps):
            # Normalize observations if enabled
            if self.obs_normalizer is not None:
                self.obs_normalizer.update(obs)
                obs_normalized = self.obs_normalizer.normalize(obs).astype(np.float32)
            else:
                obs_normalized = obs

            # Convert to tensors
            obs_t = torch.tensor(obs_normalized, dtype=torch.float32, device=self.device)
            masks_t = torch.tensor(masks, dtype=torch.bool, device=self.device)

            # Get action from policy
            with torch.no_grad():
                action, log_prob, _ = self.network.get_action(obs_t, masks_t)
                value = self.network.get_value(obs_t)

            # Step environment
            next_obs, rewards, dones, next_masks = self.envs.step(action.cpu().numpy())

            # Track episodes that finished
            for i, done in enumerate(dones):
                if done:
                    episode_infos.append({
                        "reward": self.envs.get_episode_rewards()[i],
                        "length": self.envs.get_episode_lengths()[i],
                    })

            # Store in buffer
            self.buffer.add(
                obs=obs_t,
                action=action,
                action_mask=masks_t,
                log_prob=log_prob,
                reward=torch.tensor(rewards, dtype=torch.float32, device=self.device),
                done=torch.tensor(dones, dtype=torch.float32, device=self.device),
                value=value,
            )

            obs, masks = next_obs, next_masks
            self.global_step += self.config.num_envs

        # Store last observation for next rollout
        self._last_obs = obs
        self._last_masks = masks

        # Compute final value for GAE
        with torch.no_grad():
            if self.obs_normalizer is not None:
                obs_normalized = self.obs_normalizer.normalize(obs).astype(np.float32)
            else:
                obs_normalized = obs
            obs_t = torch.tensor(obs_normalized, dtype=torch.float32, device=self.device)
            last_value = self.network.get_value(obs_t)

        # Compute advantages
        self.buffer.compute_returns_and_advantages(
            last_value=last_value,
            gamma=self.config.gamma,
            gae_lambda=self.config.gae_lambda,
        )

        return {
            "episode_infos": episode_infos,
        }

    def update(self) -> dict:
        """Perform PPO update on collected rollout."""
        self.network.train()

        # Normalize advantages
        advantages = self.buffer.advantages.reshape(-1)
        advantages = (advantages - advantages.mean()) / (advantages.std() + 1e-8)
        self.buffer.advantages = advantages.reshape(self.config.num_steps, self.config.num_envs)

        # Track metrics
        pg_losses = []
        value_losses = []
        entropy_losses = []
        approx_kls = []
        clipfracs = []

        for epoch in range(self.config.num_epochs):
            for batch in self.buffer.get_batches(self.config.minibatch_size):
                (
                    obs_batch,
                    actions_batch,
                    masks_batch,
                    old_log_probs_batch,
                    advantages_batch,
                    returns_batch,
                    old_values_batch,
                ) = batch

                # Get new log probs, entropy, and values
                new_log_probs, entropy, new_values = self.network.evaluate_actions(
                    obs_batch, masks_batch, actions_batch
                )

                # Compute ratio
                log_ratio = new_log_probs - old_log_probs_batch
                ratio = log_ratio.exp()

                # Approximate KL divergence
                with torch.no_grad():
                    approx_kl = ((ratio - 1) - log_ratio).mean()
                    approx_kls.append(approx_kl.item())
                    clipfracs.append(((ratio - 1.0).abs() > self.config.clip_coef).float().mean().item())

                # Policy loss
                pg_loss1 = -advantages_batch * ratio
                pg_loss2 = -advantages_batch * torch.clamp(
                    ratio, 1 - self.config.clip_coef, 1 + self.config.clip_coef
                )
                pg_loss = torch.max(pg_loss1, pg_loss2).mean()

                # Value loss
                if self.config.clip_vloss:
                    v_loss_unclipped = (new_values - returns_batch) ** 2
                    v_clipped = old_values_batch + torch.clamp(
                        new_values - old_values_batch,
                        -self.config.clip_coef,
                        self.config.clip_coef,
                    )
                    v_loss_clipped = (v_clipped - returns_batch) ** 2
                    v_loss = 0.5 * torch.max(v_loss_unclipped, v_loss_clipped).mean()
                else:
                    v_loss = 0.5 * ((new_values - returns_batch) ** 2).mean()

                # Entropy loss
                entropy_loss = entropy.mean()

                # Total loss
                loss = pg_loss - self.config.ent_coef * entropy_loss + self.config.vf_coef * v_loss

                # Optimize
                self.optimizer.zero_grad()
                loss.backward()
                nn.utils.clip_grad_norm_(self.network.parameters(), self.config.max_grad_norm)
                self.optimizer.step()

                pg_losses.append(pg_loss.item())
                value_losses.append(v_loss.item())
                entropy_losses.append(entropy_loss.item())

            # Early stopping on KL
            if self.config.target_kl is not None and approx_kl > self.config.target_kl:
                break

        return {
            "policy_loss": np.mean(pg_losses),
            "value_loss": np.mean(value_losses),
            "entropy": np.mean(entropy_losses),
            "approx_kl": np.mean(approx_kls),
            "clipfrac": np.mean(clipfracs),
        }

    def train(
        self,
        total_timesteps: int | None = None,
        callback: Callable[[int, dict], bool] | None = None,
    ) -> dict:
        """
        Train the agent.

        Args:
            total_timesteps: Override config total_timesteps
            callback: Optional callback(step, info) -> should_stop

        Returns:
            Training statistics
        """
        total_timesteps = total_timesteps or self.config.total_timesteps
        self.start_time = time.time()

        # Initialize first observation
        self._last_obs, self._last_masks = self.envs.reset(seed=42)

        num_updates = total_timesteps // self.config.batch_size
        all_episode_rewards = []

        print(f"Starting PPO training for {total_timesteps:,} timesteps...")
        print(f"  Batch size: {self.config.batch_size:,}")
        print(f"  Updates: {num_updates:,}")
        print(f"  Device: {self.device}")

        for update in range(1, num_updates + 1):
            # Collect rollout
            rollout_info = self.collect_rollout()

            # PPO update
            update_info = self.update()

            # Track episode rewards
            for ep_info in rollout_info["episode_infos"]:
                all_episode_rewards.append(ep_info["reward"])

            # Logging
            if update % (self.config.log_interval // self.config.batch_size + 1) == 0:
                elapsed = time.time() - self.start_time
                sps = self.global_step / elapsed

                recent_rewards = all_episode_rewards[-100:] if all_episode_rewards else [0]
                mean_reward = np.mean(recent_rewards)

                print(
                    f"Step {self.global_step:>10,} | "
                    f"SPS: {sps:>6,.0f} | "
                    f"Reward: {mean_reward:>6.2f} | "
                    f"PG Loss: {update_info['policy_loss']:>7.4f} | "
                    f"V Loss: {update_info['value_loss']:>7.4f} | "
                    f"Entropy: {update_info['entropy']:>5.3f}"
                )

                if self.writer is not None:
                    self.writer.add_scalar("charts/SPS", sps, self.global_step)
                    self.writer.add_scalar("charts/mean_reward", mean_reward, self.global_step)
                    self.writer.add_scalar("losses/policy_loss", update_info["policy_loss"], self.global_step)
                    self.writer.add_scalar("losses/value_loss", update_info["value_loss"], self.global_step)
                    self.writer.add_scalar("losses/entropy", update_info["entropy"], self.global_step)

            # Evaluation
            if update % (self.config.eval_interval // self.config.batch_size + 1) == 0:
                win_rate = self.evaluate_vs_greedy(self.config.eval_episodes)
                print(f"  -> Eval vs Greedy: {win_rate:.1%} win rate")

                if self.writer is not None:
                    self.writer.add_scalar("eval/win_rate_vs_greedy", win_rate, self.global_step)

            # Callback
            if callback is not None:
                info = {
                    "step": self.global_step,
                    "update": update,
                    **update_info,
                }
                if callback(self.global_step, info):
                    break

        return {
            "total_timesteps": self.global_step,
            "mean_reward": np.mean(all_episode_rewards) if all_episode_rewards else 0,
            "final_win_rate": self.evaluate_vs_greedy(100),
        }

    def evaluate_vs_greedy(self, num_games: int = 100) -> float:
        """
        Evaluate trained policy against GreedyBot.

        Args:
            num_games: Number of evaluation games

        Returns:
            Win rate (0.0 to 1.0)
        """
        self.network.eval()
        wins = 0

        for game_idx in range(num_games):
            env = EssenceWarsEnv(opponent="greedy")
            obs, info = env.reset(seed=game_idx + 10000)
            done = False

            while not done:
                # Normalize observation if enabled
                if self.obs_normalizer is not None:
                    obs_normalized = self.obs_normalizer.normalize(obs).astype(np.float32)
                else:
                    obs_normalized = obs

                obs_t = torch.tensor(obs_normalized, dtype=torch.float32, device=self.device).unsqueeze(0)
                mask_t = torch.tensor(info["action_mask"], dtype=torch.bool, device=self.device).unsqueeze(0)

                with torch.no_grad():
                    action, _, _ = self.network.get_action(obs_t, mask_t, deterministic=True)

                obs, reward, terminated, truncated, info = env.step(action.item())
                done = terminated or truncated

                if done and reward > 0:
                    wins += 1

        return wins / num_games

    def evaluate_vs_random(self, num_games: int = 100) -> float:
        """Evaluate trained policy against RandomBot."""
        self.network.eval()
        wins = 0

        for game_idx in range(num_games):
            env = EssenceWarsEnv(opponent="random")
            obs, info = env.reset(seed=game_idx + 20000)
            done = False

            while not done:
                # Normalize observation if enabled
                if self.obs_normalizer is not None:
                    obs_normalized = self.obs_normalizer.normalize(obs).astype(np.float32)
                else:
                    obs_normalized = obs

                obs_t = torch.tensor(obs_normalized, dtype=torch.float32, device=self.device).unsqueeze(0)
                mask_t = torch.tensor(info["action_mask"], dtype=torch.bool, device=self.device).unsqueeze(0)

                with torch.no_grad():
                    action, _, _ = self.network.get_action(obs_t, mask_t, deterministic=True)

                obs, reward, terminated, truncated, info = env.step(action.item())
                done = terminated or truncated

                if done and reward > 0:
                    wins += 1

        return wins / num_games

    def save(self, path: str) -> None:
        """Save model checkpoint."""
        checkpoint = {
            "network_state_dict": self.network.state_dict(),
            "optimizer_state_dict": self.optimizer.state_dict(),
            "global_step": self.global_step,
            "config": self.config,
        }
        # Save normalizer state if enabled
        if self.obs_normalizer is not None:
            checkpoint["obs_normalizer"] = {
                "mean": self.obs_normalizer.mean,
                "var": self.obs_normalizer.var,
                "count": self.obs_normalizer.count,
            }
        torch.save(checkpoint, path)
        print(f"Saved checkpoint to {path}")

    def load(self, path: str) -> None:
        """Load model checkpoint."""
        checkpoint = torch.load(path, map_location=self.device, weights_only=False)
        self.network.load_state_dict(checkpoint["network_state_dict"])
        self.optimizer.load_state_dict(checkpoint["optimizer_state_dict"])
        self.global_step = checkpoint["global_step"]
        # Restore normalizer state if present
        if "obs_normalizer" in checkpoint and self.obs_normalizer is not None:
            self.obs_normalizer.mean = checkpoint["obs_normalizer"]["mean"]
            self.obs_normalizer.var = checkpoint["obs_normalizer"]["var"]
            self.obs_normalizer.count = checkpoint["obs_normalizer"]["count"]
        print(f"Loaded checkpoint from {path}")

    def get_policy(self) -> EssenceWarsNetwork:
        """Get the trained policy network."""
        return self.network
