#!/usr/bin/env python3
"""Benchmark script for vectorized environment throughput."""

import time

import numpy as np

from essence_wars.env import VectorizedEssenceWars


def benchmark_vectorized_env(num_envs: int = 64, num_steps: int = 10000) -> dict:
    """
    Benchmark vectorized environment throughput.

    Args:
        num_envs: Number of parallel environments
        num_steps: Number of steps to run

    Returns:
        Dictionary with benchmark results
    """
    print(f"Benchmarking VectorizedEssenceWars with {num_envs} environments...")

    vec_env = VectorizedEssenceWars(num_envs=num_envs)
    obs, masks = vec_env.reset(seed=42)

    # Warmup
    for _ in range(100):
        actions = []
        for i in range(num_envs):
            legal = np.where(masks[i])[0]
            actions.append(np.random.choice(legal))
        actions = np.array(actions, dtype=np.uint8)
        obs, rewards, dones, masks = vec_env.step(actions)

    # Benchmark
    episodes_completed = 0
    total_reward = 0.0

    start_time = time.perf_counter()

    for step in range(num_steps):
        # Select random legal actions for all envs
        actions = []
        for i in range(num_envs):
            legal = np.where(masks[i])[0]
            actions.append(np.random.choice(legal))
        actions = np.array(actions, dtype=np.uint8)

        obs, rewards, dones, masks = vec_env.step(actions)

        episodes_completed += np.sum(dones)
        total_reward += np.sum(rewards)

    elapsed = time.perf_counter() - start_time

    total_steps = num_steps * num_envs
    sps = total_steps / elapsed

    results = {
        "num_envs": num_envs,
        "num_steps": num_steps,
        "total_steps": total_steps,
        "elapsed_seconds": elapsed,
        "steps_per_second": sps,
        "episodes_completed": episodes_completed,
        "avg_reward": total_reward / episodes_completed if episodes_completed > 0 else 0.0,
    }

    return results


def main():
    print("=" * 60)
    print("Essence Wars Vectorized Environment Benchmark")
    print("=" * 60)

    # Test different batch sizes
    batch_sizes = [1, 8, 16, 32, 64, 128, 256]

    results = []
    for num_envs in batch_sizes:
        # Scale steps so total work is similar
        num_steps = max(1000, 100000 // num_envs)

        result = benchmark_vectorized_env(num_envs=num_envs, num_steps=num_steps)
        results.append(result)

        print(f"\n  Environments: {result['num_envs']:>4}")
        print(f"  Total steps:  {result['total_steps']:>10,}")
        print(f"  Time:         {result['elapsed_seconds']:>10.3f} seconds")
        print(f"  SPS:          {result['steps_per_second']:>10,.0f} steps/sec")
        print(f"  Episodes:     {result['episodes_completed']:>10,}")

    # Find best configuration
    best = max(results, key=lambda x: x["steps_per_second"])

    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"Best configuration: {best['num_envs']} environments")
    print(f"Peak throughput:    {best['steps_per_second']:,.0f} SPS")

    target_sps = 60000
    if best["steps_per_second"] >= target_sps:
        print(f"\n[PASS] Achieved >{target_sps:,} SPS target!")
    else:
        print(f"\n[WARN] Below {target_sps:,} SPS target")
        print(f"       Missing by {target_sps - best['steps_per_second']:,.0f} SPS")


if __name__ == "__main__":
    main()
