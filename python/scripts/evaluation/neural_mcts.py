#!/usr/bin/env python3
"""
Evaluate Neural MCTS-Augmented Inference.

This script tests whether using MCTS at inference time improves
neural network performance over raw network output.

Usage:
    # Test PPO model with different sim counts
    uv run python python/scripts/evaluate_neural_mcts.py \
        --model models/ppo_argentum_best.pt \
        --sims 0 25 50 100 \
        --games 100

    # Test BC model
    uv run python python/scripts/evaluate_neural_mcts.py \
        --model models/bc_mcts_10k_best.pt \
        --model-type bc \
        --sims 0 50 100

    # Quick test
    uv run python python/scripts/evaluate_neural_mcts.py \
        --model models/ppo_argentum_best.pt \
        --sims 0 25 \
        --games 20
"""

import argparse
import sys
import time
from pathlib import Path

import numpy as np
import torch

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))


def evaluate_raw_network(network, num_games: int, opponent: str, device: str, obs_normalizer=None) -> dict:
    """Evaluate raw network without MCTS."""
    from essence_wars._core import PyGame

    network.eval()
    wins = 0
    total_time = 0.0
    total_moves = 0

    for game_idx in range(num_games):
        game = PyGame()
        game.reset(seed=game_idx)

        while not game.is_done():
            current_player = game.current_player()

            if current_player == 0:
                # Neural network
                obs = game.observe()
                mask = game.action_mask()

                start = time.time()

                # Apply normalization if available
                if obs_normalizer is not None:
                    obs = obs_normalizer.normalize(obs).astype(np.float32)

                obs_t = torch.tensor(obs, dtype=torch.float32, device=device).unsqueeze(0)
                mask_t = torch.tensor(mask > 0, dtype=torch.bool, device=device).unsqueeze(0)

                with torch.no_grad():
                    logits, _ = network(obs_t, mask_t)
                    action = int(logits.argmax(dim=-1).item())

                total_time += time.time() - start
                total_moves += 1
            else:
                # Opponent
                if opponent == "greedy":
                    action = game.greedy_action()
                else:
                    mask = game.action_mask()
                    legal = np.where(mask > 0)[0]
                    action = int(np.random.choice(legal))

            game.step(action)

        if game.get_reward(0) > 0:
            wins += 1

    return {
        "win_rate": wins / num_games,
        "wins": wins,
        "games": num_games,
        "avg_time_ms": (total_time / total_moves * 1000) if total_moves > 0 else 0,
        "simulations": 0,
    }


def main():
    parser = argparse.ArgumentParser(
        description="Evaluate Neural MCTS-Augmented Inference",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )

    parser.add_argument(
        "--model",
        type=str,
        required=True,
        help="Path to model checkpoint",
    )
    parser.add_argument(
        "--model-type",
        type=str,
        default="ppo",
        choices=["ppo", "bc"],
        help="Type of model (ppo or bc)",
    )
    parser.add_argument(
        "--sims",
        type=int,
        nargs="+",
        default=[0, 25, 50, 100],
        help="Simulation counts to test (0 = raw network)",
    )
    parser.add_argument(
        "--games",
        type=int,
        default=100,
        help="Games per evaluation",
    )
    parser.add_argument(
        "--opponent",
        type=str,
        default="greedy",
        choices=["greedy", "random"],
        help="Opponent type",
    )
    parser.add_argument(
        "--device",
        type=str,
        default="auto",
        help="Device (auto, cuda, cpu)",
    )

    args = parser.parse_args()

    # Set device
    if args.device == "auto":
        device = "cuda" if torch.cuda.is_available() else "cpu"
    else:
        device = args.device

    print("=" * 60)
    print("Neural MCTS-Augmented Inference Evaluation")
    print("=" * 60)
    print(f"  Model:     {args.model}")
    print(f"  Type:      {args.model_type}")
    print(f"  Sims:      {args.sims}")
    print(f"  Games:     {args.games}")
    print(f"  Opponent:  {args.opponent}")
    print(f"  Device:    {device}")
    print("=" * 60)

    # Load model
    print(f"\nLoading model from {args.model}...")
    from essence_wars.agents.neural_mcts import load_ppo_network, load_bc_network

    obs_normalizer = None
    if args.model_type == "ppo":
        network, obs_normalizer = load_ppo_network(args.model, device=device)
        if obs_normalizer is not None:
            print("  Loaded with observation normalizer")
    else:
        network = load_bc_network(args.model, device=device)

    print("  Model loaded successfully")

    # Run evaluations
    results = []

    for sims in args.sims:
        print(f"\n{'=' * 40}")
        if sims == 0:
            print(f"Evaluating: Raw Network (no MCTS)")
        else:
            print(f"Evaluating: Neural MCTS with {sims} simulations")
        print("=" * 40)

        if sims == 0:
            # Raw network evaluation
            result = evaluate_raw_network(
                network=network,
                num_games=args.games,
                opponent=args.opponent,
                device=device,
                obs_normalizer=obs_normalizer,
            )
        else:
            # MCTS-augmented evaluation
            from essence_wars.agents.neural_mcts import evaluate_neural_mcts

            result = evaluate_neural_mcts(
                network=network,
                num_simulations=sims,
                num_games=args.games,
                opponent=args.opponent,
                device=device,
                verbose=True,
                obs_normalizer=obs_normalizer,
            )

        results.append(result)

        print(f"\n  Win Rate: {result['win_rate']*100:.1f}%")
        print(f"  Avg Time: {result.get('avg_time_ms', result.get('avg_search_time_ms', 0)):.1f}ms/move")

    # Summary table
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"{'Sims':>6} | {'Win Rate':>10} | {'Avg Time':>12} | {'Improvement':>12}")
    print("-" * 60)

    baseline_win_rate = results[0]["win_rate"] if results else 0

    for i, (sims, result) in enumerate(zip(args.sims, results)):
        win_rate = result["win_rate"]
        avg_time = result.get("avg_time_ms", result.get("avg_search_time_ms", 0))

        if i == 0:
            improvement = "baseline"
        else:
            diff = (win_rate - baseline_win_rate) * 100
            improvement = f"{diff:+.1f}%"

        print(f"{sims:>6} | {win_rate*100:>9.1f}% | {avg_time:>10.1f}ms | {improvement:>12}")

    print("=" * 60)

    # Best result
    best_idx = max(range(len(results)), key=lambda i: results[i]["win_rate"])
    best_sims = args.sims[best_idx]
    best_rate = results[best_idx]["win_rate"]

    print(f"\nBest configuration: {best_sims} simulations ({best_rate*100:.1f}% win rate)")

    if best_sims > 0 and best_rate > baseline_win_rate:
        print(f"  MCTS improves performance by {(best_rate - baseline_win_rate)*100:.1f}%")
    elif best_sims == 0:
        print("  Raw network is best (MCTS doesn't help)")
    else:
        print("  MCTS matches raw network performance")


if __name__ == "__main__":
    main()
