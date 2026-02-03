#!/usr/bin/env python3
"""
Benchmark BC models against various opponents to measure performance.

This script evaluates how well behavioral cloning models perform
compared to different opponents:
- random: Random legal moves (weakest)
- greedy: Heuristic evaluation (fast, moderate strength)
- alphabetaN: Alpha-Beta search depth N (fast, strong) - RECOMMENDED
- mctsN: MCTS with N simulations (slower, strong)

Alpha-Beta is 18-30x faster than MCTS at comparable strength,
making it the recommended opponent for fast evaluation.
"""

import argparse
import json
import sys
import time
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn

sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from essence_wars._core import PyGame


class LegacyResidualBlock(nn.Module):
    """Residual block matching the saved checkpoint format."""

    def __init__(self, hidden_dim: int) -> None:
        super().__init__()
        self.layers = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return torch.relu(x + self.layers(x))


class LegacyAlphaZeroNetwork(nn.Module):
    """
    AlphaZeroNetwork without input_norm for loading legacy checkpoints.
    """

    def __init__(
        self,
        obs_dim: int = 326,
        action_dim: int = 256,
        hidden_dim: int = 256,
        num_blocks: int = 4,
    ) -> None:
        super().__init__()

        self.obs_dim = obs_dim
        self.action_dim = action_dim
        self.hidden_dim = hidden_dim
        self.num_blocks = num_blocks

        # Input projection (NO input_norm - legacy format)
        self.input_proj = nn.Sequential(
            nn.Linear(obs_dim, hidden_dim),
            nn.ReLU(),
        )

        # Residual tower (using proper ResidualBlock structure)
        self.residual_tower = nn.ModuleList([
            LegacyResidualBlock(hidden_dim) for _ in range(num_blocks)
        ])

        # Policy head
        self.policy_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, action_dim),
        )

        # Value head
        self.value_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
            nn.Tanh(),
        )

    def forward(
        self,
        obs: torch.Tensor,
        action_mask: torch.Tensor | None = None,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        # Input projection
        x = self.input_proj(obs)

        # Residual tower (each block handles its own skip connection)
        for block in self.residual_tower:
            x = block(x)

        # Policy head
        logits = self.policy_head(x)

        # Mask illegal actions
        if action_mask is not None:
            logits = logits.masked_fill(~action_mask, float("-inf"))

        # Value head
        value = self.value_head(x).squeeze(-1)

        return logits, value


def load_legacy_bc_network(checkpoint_path: str, device: str = "cuda") -> nn.Module:
    """Load BC checkpoint with legacy format handling."""
    checkpoint = torch.load(checkpoint_path, map_location=device, weights_only=False)

    # Get args to determine architecture
    args = checkpoint.get("args", {})
    if hasattr(args, "__dict__"):
        args = vars(args)

    hidden_dim = args.get("hidden_dim", 256)
    num_blocks = args.get("num_blocks", 4)

    # Try loading with legacy network first
    network = LegacyAlphaZeroNetwork(
        hidden_dim=hidden_dim,
        num_blocks=num_blocks,
    )

    state_dict = checkpoint["model_state_dict"]

    # Check if this is legacy format (no input_norm)
    if "input_norm.weight" not in state_dict:
        network.load_state_dict(state_dict)
    else:
        # New format - use strict=False to handle differences
        network.load_state_dict(state_dict, strict=False)

    network.to(device)
    network.eval()
    return network


def evaluate_agent_vs_opponent(
    agent_fn,  # callable(obs, mask) -> action
    opponent: str,
    num_games: int,
    verbose: bool = True,
) -> dict:
    """Evaluate an agent against an opponent."""
    # Get available decks for random selection
    try:
        available_decks = PyGame.list_decks()
    except AttributeError:
        available_decks = [
            "architect_fortify", "colossus_wall", "vex_piercing",
            "broodmother_pack", "alpha_frenzy", "plague_volatile",
            "archon_burst", "shadow_weaver",
        ]

    wins = 0
    losses = 0
    draws = 0
    game_lengths = []
    agent_times = []

    for game_idx in range(num_games):
        # Random deck selection for fair evaluation
        deck1 = np.random.choice(available_decks)
        deck2 = np.random.choice(available_decks)
        game = PyGame(deck1=deck1, deck2=deck2)
        game.reset(seed=game_idx + 1000)

        turns = 0
        game_agent_times = []

        while not game.is_done():
            current_player = game.current_player()
            obs = np.array(game.observe(), dtype=np.float32)
            mask = np.array(game.action_mask(), dtype=np.float32)

            if current_player == 0:
                # Agent's turn
                start = time.perf_counter()
                action = agent_fn(obs, mask)
                elapsed = (time.perf_counter() - start) * 1000
                game_agent_times.append(elapsed)
            else:
                # Opponent's turn
                if opponent == "random":
                    valid = np.where(mask > 0.5)[0]
                    action = int(np.random.choice(valid)) if len(valid) > 0 else 255
                elif opponent == "greedy":
                    action = game.greedy_action()
                elif opponent.startswith("mcts"):
                    sims = int(opponent.replace("mcts", ""))
                    action = game.mcts_action(sims)
                elif opponent.startswith("alphabeta"):
                    # Parse depth (e.g., "alphabeta6" or "alphabeta-d6")
                    depth_str = opponent.replace("alphabeta", "").replace("-d", "").replace("d", "")
                    depth = int(depth_str) if depth_str else 6
                    action = game.alphabeta_action(depth)
                else:
                    raise ValueError(f"Unknown opponent: {opponent}")

            game.step(action)
            turns += 1

            if turns > 500:
                break

        reward = game.get_reward(0)
        if reward > 0:
            wins += 1
        elif reward < 0:
            losses += 1
        else:
            draws += 1

        game_lengths.append(turns)
        agent_times.extend(game_agent_times)

        if verbose and (game_idx + 1) % 20 == 0:
            print(f"  Progress: {game_idx + 1}/{num_games} games")

    win_rate = wins / num_games
    return {
        "wins": wins,
        "losses": losses,
        "draws": draws,
        "win_rate": win_rate,
        "games": num_games,
        "avg_game_length": float(np.mean(game_lengths)),
        "avg_decision_time_ms": float(np.mean(agent_times)) if agent_times else 0,
    }


def main():
    parser = argparse.ArgumentParser(
        description="Benchmark BC models against MCTS-100",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--models",
        nargs="+",
        default=[
            "models/bc_mcts_10k_best.pt",
            "models/distilled_mcts50_10k.pt",
            "models/bc_mcts_values.pt",
        ],
        help="Model checkpoints to evaluate",
    )
    parser.add_argument(
        "--games",
        type=int,
        default=100,
        help="Games per opponent",
    )
    parser.add_argument(
        "--opponents",
        nargs="+",
        default=["random", "greedy", "alphabeta6", "mcts100"],
        help="Opponents to test against (alphabetaN for depth N, mctsN for N sims)",
    )
    parser.add_argument(
        "--device",
        type=str,
        default="cuda",
        help="Device for inference",
    )
    parser.add_argument(
        "--output",
        type=str,
        default="experiments/bc_vs_mcts_benchmark.json",
        help="Output JSON file",
    )

    args = parser.parse_args()

    device = args.device
    if device == "cuda" and not torch.cuda.is_available():
        device = "cpu"
        print("CUDA not available, using CPU")

    print("=" * 70)
    print("BC Model Benchmark")
    print("=" * 70)
    print(f"  Models:    {args.models}")
    print(f"  Opponents: {args.opponents}")
    print(f"  Games:     {args.games} per opponent")
    print(f"  Device:    {device}")
    print("=" * 70)

    all_results = {}

    for model_path in args.models:
        model_name = Path(model_path).stem
        print(f"\n{'=' * 50}")
        print(f"Evaluating: {model_name}")
        print("=" * 50)

        if not Path(model_path).exists():
            print(f"  SKIPPING: {model_path} not found")
            continue

        # Load model
        try:
            network = load_legacy_bc_network(model_path, device)
            print(f"  Loaded successfully")
        except Exception as e:
            print(f"  FAILED to load: {e}")
            continue

        # Create agent function
        def make_agent(net, dev):
            def agent_fn(obs, mask):
                with torch.no_grad():
                    obs_t = torch.tensor(obs, dtype=torch.float32, device=dev).unsqueeze(0)
                    mask_t = torch.tensor(mask > 0, dtype=torch.bool, device=dev).unsqueeze(0)
                    logits, _ = net(obs_t, mask_t)
                    return int(logits.argmax(dim=-1).item())
            return agent_fn

        agent_fn = make_agent(network, device)

        model_results = {}

        for opponent in args.opponents:
            print(f"\n  vs {opponent}...")
            result = evaluate_agent_vs_opponent(
                agent_fn=agent_fn,
                opponent=opponent,
                num_games=args.games,
                verbose=True,
            )
            model_results[opponent] = result
            print(f"    Win rate: {result['win_rate']*100:.1f}%")
            print(f"    W/L/D: {result['wins']}/{result['losses']}/{result['draws']}")

        all_results[model_name] = model_results

    # Print summary
    print("\n" + "=" * 70)
    print("SUMMARY: Win Rates by Model")
    print("=" * 70)

    # Header
    header = f"{'Model':<30}"
    for opp in args.opponents:
        header += f" | {opp:>10}"
    print(header)
    print("-" * 70)

    # Results
    for model_name, results in all_results.items():
        row = f"{model_name:<30}"
        for opp in args.opponents:
            if opp in results:
                row += f" | {results[opp]['win_rate']*100:>9.1f}%"
            else:
                row += f" | {'N/A':>10}"
        print(row)

    print("=" * 70)

    # Key insight: MCTS-100 gap
    print("\nKEY INSIGHT: Gap to MCTS-100 quality")
    print("-" * 50)
    for model_name, results in all_results.items():
        if "mcts100" in results:
            win_rate = results["mcts100"]["win_rate"]
            if win_rate >= 0.45:
                assessment = "CLOSE (distillation may be sufficient)"
            elif win_rate >= 0.30:
                assessment = "MODERATE (hybrid approach recommended)"
            else:
                assessment = "LARGE (need faster MCTS or better training)"
            print(f"  {model_name}: {win_rate*100:.1f}% vs MCTS-100 - {assessment}")

    # Save results
    Path(args.output).parent.mkdir(parents=True, exist_ok=True)
    with open(args.output, "w") as f:
        json.dump(all_results, f, indent=2)
    print(f"\nResults saved to: {args.output}")


if __name__ == "__main__":
    main()
