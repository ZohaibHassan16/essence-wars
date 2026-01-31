#!/usr/bin/env python3
"""Evaluate Decision Transformer against various opponents.

Usage:
    uv run python python/scripts/evaluate_decision_transformer.py \
        --model models/decision_transformer.pt \
        --games 500 \
        --opponent greedy
"""

from __future__ import annotations

import argparse
import sys
import time
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).parent.parent))

from essence_wars._core import PyGame
from essence_wars.agents.decision_transformer import (
    DecisionTransformer,
    DecisionTransformerAgent,
    DecisionTransformerConfig,
)


def evaluate_dt(
    agent: DecisionTransformerAgent,
    num_games: int,
    opponent: str,
    verbose: bool = True,
) -> dict:
    """Evaluate Decision Transformer against opponent."""
    wins = 0
    losses = 0
    draws = 0
    total_moves = 0
    total_time = 0.0

    for game_idx in range(num_games):
        game = PyGame()
        game.reset(seed=game_idx)
        agent.reset()

        game_moves = 0
        while not game.is_done():
            current_player = game.current_player()

            if current_player == 0:
                # Decision Transformer
                obs = game.observe()
                mask = game.action_mask()

                start = time.time()
                action = agent.get_action(obs, mask > 0)
                total_time += time.time() - start

                game_moves += 1
            else:
                # Opponent
                if opponent == "greedy":
                    action = game.greedy_action()
                else:
                    mask = game.action_mask()
                    legal = np.where(mask > 0)[0]
                    action = int(np.random.choice(legal))

            game.step(action)

        reward = game.get_reward(0)
        if reward > 0:
            wins += 1
        elif reward < 0:
            losses += 1
        else:
            draws += 1

        total_moves += game_moves

        if verbose and (game_idx + 1) % 100 == 0:
            print(
                f"  Game {game_idx + 1}/{num_games} | "
                f"W: {wins} L: {losses} D: {draws} | "
                f"WR: {wins / (game_idx + 1) * 100:.1f}%"
            )

    return {
        "wins": wins,
        "losses": losses,
        "draws": draws,
        "games": num_games,
        "win_rate": wins / num_games,
        "total_moves": total_moves,
        "avg_time_ms": (total_time / total_moves * 1000) if total_moves > 0 else 0,
    }


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Evaluate Decision Transformer",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--model",
        type=str,
        default="models/decision_transformer.pt",
        help="Path to model checkpoint",
    )
    parser.add_argument(
        "--games",
        type=int,
        default=500,
        help="Number of games to play",
    )
    parser.add_argument(
        "--opponent",
        type=str,
        default="greedy",
        choices=["greedy", "random"],
        help="Opponent type",
    )
    parser.add_argument(
        "--target-return",
        type=float,
        default=1.0,
        help="Target return for conditioning (1.0 = aim to win)",
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
    print("Decision Transformer Evaluation")
    print("=" * 60)
    print(f"  Model:         {args.model}")
    print(f"  Games:         {args.games}")
    print(f"  Opponent:      {args.opponent}")
    print(f"  Target Return: {args.target_return}")
    print(f"  Device:        {device}")
    print("=" * 60)

    # Load model
    print(f"\nLoading model from {args.model}...")
    torch.serialization.add_safe_globals([DecisionTransformerConfig])
    checkpoint = torch.load(args.model, map_location=device, weights_only=True)

    config = checkpoint["config"]
    print(f"  Config: d_model={config.d_model}, n_layers={config.n_layers}")

    model = DecisionTransformer(config)
    model.load_state_dict(checkpoint["model_state_dict"])
    model.to(device)
    model.eval()

    # Create agent
    agent = DecisionTransformerAgent(
        model=model,
        device=device,
        target_return=args.target_return,
    )

    # Evaluate
    print(f"\nEvaluating against {args.opponent}...")
    result = evaluate_dt(
        agent=agent,
        num_games=args.games,
        opponent=args.opponent,
        verbose=True,
    )

    print("\n" + "=" * 60)
    print("RESULTS")
    print("=" * 60)
    print(f"  Games:    {result['games']}")
    print(f"  Wins:     {result['wins']}")
    print(f"  Losses:   {result['losses']}")
    print(f"  Draws:    {result['draws']}")
    print(f"  Win Rate: {result['win_rate'] * 100:.1f}%")
    print(f"  Avg Time: {result['avg_time_ms']:.2f}ms/move")
    print("=" * 60)


if __name__ == "__main__":
    main()
