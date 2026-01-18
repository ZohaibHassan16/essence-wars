#!/usr/bin/env python3
"""Diagnostic script to verify PPO infrastructure is working correctly."""

import sys
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).parent.parent))


def check_environment_basics():
    """Verify basic environment functionality."""
    print("=" * 60)
    print("1. Environment Basics")
    print("=" * 60)

    from essence_wars.env import EssenceWarsEnv

    env = EssenceWarsEnv(opponent="greedy")
    obs, info = env.reset(seed=42)

    print(f"  Observation shape: {obs.shape}")
    print(f"  Observation dtype: {obs.dtype}")
    print(f"  Observation range: [{obs.min():.2f}, {obs.max():.2f}]")
    print(f"  Action mask shape: {info['action_mask'].shape}")
    print(f"  Legal actions: {np.sum(info['action_mask'])}")

    # Check observation normalization
    print("\n  Observation stats:")
    print(f"    Mean: {obs.mean():.4f}")
    print(f"    Std:  {obs.std():.4f}")

    # Observations might not be normalized - this could be an issue!
    if obs.std() > 10:
        print("  [WARN] Observations have high variance - consider normalization!")

    return True


def check_reward_signal():
    """Verify rewards are being received correctly."""
    print("\n" + "=" * 60)
    print("2. Reward Signal Check")
    print("=" * 60)

    from essence_wars.env import EssenceWarsEnv

    # Play games and collect rewards
    rewards_vs_greedy = []
    rewards_vs_random = []

    for seed in range(20):
        # Vs Greedy
        env = EssenceWarsEnv(opponent="greedy")
        obs, info = env.reset(seed=seed)
        total_reward = 0
        done = False
        while not done:
            mask = info["action_mask"]
            action = int(np.random.choice(np.where(mask)[0]))
            obs, reward, terminated, truncated, info = env.step(action)
            total_reward += reward
            done = terminated or truncated
        rewards_vs_greedy.append(total_reward)

        # Vs Random
        env = EssenceWarsEnv(opponent="random")
        obs, info = env.reset(seed=seed)
        total_reward = 0
        done = False
        while not done:
            mask = info["action_mask"]
            action = int(np.random.choice(np.where(mask)[0]))
            obs, reward, terminated, truncated, info = env.step(action)
            total_reward += reward
            done = terminated or truncated
        rewards_vs_random.append(total_reward)

    print("  Random policy vs Greedy:")
    print(f"    Rewards: {rewards_vs_greedy}")
    print(f"    Win rate: {sum(1 for r in rewards_vs_greedy if r > 0) / len(rewards_vs_greedy):.1%}")

    print("\n  Random policy vs Random:")
    print(f"    Rewards: {rewards_vs_random}")
    print(f"    Win rate: {sum(1 for r in rewards_vs_random if r > 0) / len(rewards_vs_random):.1%}")

    return True


def check_vectorized_env():
    """Verify vectorized environment works correctly."""
    print("\n" + "=" * 60)
    print("3. Vectorized Environment Check")
    print("=" * 60)

    from essence_wars.env import VectorizedEssenceWars

    vec_env = VectorizedEssenceWars(num_envs=8)
    obs, masks = vec_env.reset(seed=42)

    print(f"  Obs shape: {obs.shape}")
    print(f"  Masks shape: {masks.shape}")

    # Play some steps and check auto-reset
    episodes_completed = 0
    total_rewards = []

    for step in range(200):
        actions = []
        for i in range(8):
            legal = np.where(masks[i])[0]
            actions.append(np.random.choice(legal))
        actions = np.array(actions, dtype=np.uint8)

        obs, rewards, dones, masks = vec_env.step(actions)

        for i, (done, reward) in enumerate(zip(dones, rewards)):
            if done:
                episodes_completed += 1
                total_rewards.append(reward)

    print(f"  Episodes completed: {episodes_completed}")
    if total_rewards:
        print(f"  Reward distribution: {dict(zip(*np.unique(total_rewards, return_counts=True)))}")

    return True


def check_network_outputs():
    """Verify network produces sensible outputs."""
    print("\n" + "=" * 60)
    print("4. Network Output Check")
    print("=" * 60)

    from essence_wars.agents.networks import EssenceWarsNetwork
    from essence_wars.env import EssenceWarsEnv

    network = EssenceWarsNetwork()
    network.eval()

    env = EssenceWarsEnv(opponent="greedy")
    obs, info = env.reset(seed=42)

    obs_t = torch.tensor(obs, dtype=torch.float32).unsqueeze(0)
    mask_t = torch.tensor(info["action_mask"], dtype=torch.bool).unsqueeze(0)

    with torch.no_grad():
        logits, value = network(obs_t, mask_t)
        probs = torch.softmax(logits, dim=-1)

    print(f"  Value estimate: {value.item():.4f}")
    print(f"  Prob range: [{probs.min().item():.6f}, {probs.max().item():.6f}]")
    print(f"  Prob sum: {probs.sum().item():.4f}")
    print(f"  Entropy: {-(probs * probs.log().clamp(min=-100)).sum().item():.4f}")

    # Check if logits are reasonable
    legal_logits = logits[0, mask_t[0]]
    print(f"  Legal action logits range: [{legal_logits.min().item():.2f}, {legal_logits.max().item():.2f}]")

    return True


def check_training_loop():
    """Verify training loop mechanics."""
    print("\n" + "=" * 60)
    print("5. Training Loop Check (with observation normalization)")
    print("=" * 60)

    from essence_wars.agents.ppo import PPOConfig, PPOTrainer

    config = PPOConfig(num_envs=4, num_steps=16, total_timesteps=256, normalize_obs=True)
    trainer = PPOTrainer(config=config)

    print(f"  Observation normalization: {'ENABLED' if config.normalize_obs else 'DISABLED'}")

    # Initialize
    trainer._last_obs, trainer._last_masks = trainer.envs.reset(seed=42)

    # Collect rollout
    rollout_info = trainer.collect_rollout()

    print(f"  Rollout collected: {trainer.global_step} steps")
    print(f"  Episodes in rollout: {len(rollout_info['episode_infos'])}")

    # Check normalizer statistics
    if trainer.obs_normalizer is not None:
        print(f"\n  Normalizer statistics (after {trainer.obs_normalizer.count:.0f} samples):")
        print(f"    Mean range: [{trainer.obs_normalizer.mean.min():.2f}, {trainer.obs_normalizer.mean.max():.2f}]")
        print(f"    Std range: [{np.sqrt(trainer.obs_normalizer.var).min():.2f}, {np.sqrt(trainer.obs_normalizer.var).max():.2f}]")

    # Check buffer contents
    print("\n  Buffer contents (NORMALIZED observations):")
    print(f"    Obs range: [{trainer.buffer.obs.min().item():.2f}, {trainer.buffer.obs.max().item():.2f}]")
    print(f"    Rewards: {trainer.buffer.rewards.sum().item():.4f} total")
    print(f"    Values range: [{trainer.buffer.values.min().item():.2f}, {trainer.buffer.values.max().item():.2f}]")
    print(f"    Advantages range: [{trainer.buffer.advantages.min().item():.2f}, {trainer.buffer.advantages.max().item():.2f}]")

    # Check if normalized obs look reasonable
    obs_min = trainer.buffer.obs.min().item()
    obs_max = trainer.buffer.obs.max().item()
    if abs(obs_min) < 20 and abs(obs_max) < 20:
        print("  [OK] Normalized observations are in reasonable range!")
    else:
        print("  [WARN] Normalized observations may still have issues")

    # Perform update
    update_info = trainer.update()

    print("\n  Update info:")
    print(f"    Policy loss: {update_info['policy_loss']:.4f}")
    print(f"    Value loss: {update_info['value_loss']:.4f}")
    print(f"    Entropy: {update_info['entropy']:.4f}")
    print(f"    Approx KL: {update_info['approx_kl']:.4f}")

    # Check if metrics are healthy
    if update_info['value_loss'] < 10 and update_info['approx_kl'] < 0.1:
        print("  [OK] Training metrics look healthy!")
    else:
        print("  [WARN] Training metrics may indicate issues")

    return True


def check_observation_normalization():
    """Check if observations need normalization."""
    print("\n" + "=" * 60)
    print("6. Observation Analysis")
    print("=" * 60)

    from essence_wars.env import EssenceWarsEnv

    # Collect observations from multiple games
    all_obs = []
    for seed in range(10):
        env = EssenceWarsEnv(opponent="random")
        obs, info = env.reset(seed=seed)
        all_obs.append(obs)

        for _ in range(20):
            mask = info["action_mask"]
            action = int(np.random.choice(np.where(mask)[0]))
            obs, _, terminated, truncated, info = env.step(action)
            all_obs.append(obs)
            if terminated or truncated:
                break

    all_obs = np.stack(all_obs)

    print(f"  Collected {len(all_obs)} observations")
    print("\n  Per-feature statistics (first 20 features):")
    for i in range(min(20, all_obs.shape[1])):
        mean = all_obs[:, i].mean()
        std = all_obs[:, i].std()
        min_val = all_obs[:, i].min()
        max_val = all_obs[:, i].max()
        print(f"    [{i:3d}] mean={mean:8.2f}, std={std:8.2f}, range=[{min_val:8.2f}, {max_val:8.2f}]")

    print(f"\n  Overall: mean={all_obs.mean():.2f}, std={all_obs.std():.2f}")

    if all_obs.std() > 5:
        print("\n  [ISSUE] Observations are NOT normalized!")
        print("  This can cause training instability. Consider adding normalization.")

    return True


def main():
    print("PPO Infrastructure Diagnostics")
    print("=" * 60)

    check_environment_basics()
    check_reward_signal()
    check_vectorized_env()
    check_network_outputs()
    check_training_loop()
    check_observation_normalization()

    print("\n" + "=" * 60)
    print("Diagnostics Complete")
    print("=" * 60)


if __name__ == "__main__":
    main()
