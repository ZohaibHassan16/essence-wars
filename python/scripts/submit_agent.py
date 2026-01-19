#!/usr/bin/env python3
"""Submit a trained agent to the Essence Wars leaderboard.

This script:
1. Validates your checkpoint can be loaded
2. Runs a quick evaluation (20 games vs Greedy)
3. Optionally uploads to HuggingFace Hub
4. Adds entry to leaderboard.json
5. Regenerates leaderboard markdown

Usage:
    # Local submission (no HuggingFace upload)
    python submit_agent.py \\
        --checkpoint models/my_agent.pt \\
        --name "My PPO Agent" \\
        --author "myusername"

    # Full submission with HuggingFace upload
    python submit_agent.py \\
        --checkpoint models/my_agent.pt \\
        --name "My PPO Agent" \\
        --repo-id myusername/essence-wars-agent \\
        --description "Custom PPO with novel architecture"

    # Run full benchmark (slower, more accurate)
    python submit_agent.py \\
        --checkpoint models/my_agent.pt \\
        --name "My PPO Agent" \\
        --repo-id myusername/essence-wars-agent \\
        --full-eval
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

# Add parent to path for local development
sys.path.insert(0, str(Path(__file__).parent.parent))


def validate_checkpoint(checkpoint_path: Path) -> tuple[bool, str]:
    """Validate that checkpoint can be loaded as a NeuralAgent."""
    try:
        from essence_wars.benchmark.agents import NeuralAgent

        agent = NeuralAgent.from_checkpoint(str(checkpoint_path), name="test")
        return True, f"Loaded successfully as {type(agent).__name__}"
    except Exception as e:
        return False, f"Failed to load: {e}"


def quick_evaluate(checkpoint_path: Path, games: int = 20) -> dict:
    """Run quick evaluation against Greedy and Random."""
    from essence_wars.benchmark.agents import GreedyAgent, NeuralAgent, RandomAgent
    from essence_wars.benchmark.api import run_matches

    agent = NeuralAgent.from_checkpoint(str(checkpoint_path), name="submitted")

    print(f"  Running {games} games vs Greedy...")
    greedy_results = run_matches(agent, GreedyAgent(), num_games=games)
    win_rate_greedy = greedy_results["wins"] / games

    print(f"  Running {games} games vs Random...")
    random_results = run_matches(agent, RandomAgent(), num_games=games)
    win_rate_random = random_results["wins"] / games

    # Estimate Elo based on win rate vs Greedy (baseline 1300)
    # Using simplified Elo estimation
    if win_rate_greedy > 0 and win_rate_greedy < 1:
        import math

        elo_diff = -400 * math.log10((1 / win_rate_greedy) - 1)
        estimated_elo = 1300 + elo_diff
    elif win_rate_greedy >= 1:
        estimated_elo = 1600
    else:
        estimated_elo = 1000

    return {
        "win_rate_vs_greedy": win_rate_greedy,
        "win_rate_vs_random": win_rate_random,
        "estimated_elo": estimated_elo,
        "games_played": games * 2,
    }


def full_evaluate(checkpoint_path: Path) -> dict:
    """Run full benchmark evaluation."""
    from essence_wars.benchmark.agents import NeuralAgent
    from essence_wars.benchmark.api import EssenceWarsBenchmark

    agent = NeuralAgent.from_checkpoint(str(checkpoint_path), name="submitted")
    benchmark = EssenceWarsBenchmark(games_per_opponent=100)

    print("  Running full benchmark (400 games)...")
    results = benchmark.evaluate(agent)

    return {
        "win_rate_vs_greedy": results.win_rates.get("greedy", 0),
        "win_rate_vs_random": results.win_rates.get("random", 0),
        "win_rate_vs_mcts50": results.win_rates.get("mcts50"),
        "win_rate_vs_mcts100": results.win_rates.get("mcts100"),
        "elo_rating": results.elo_rating,
        "games_played": 400,
    }


def upload_to_huggingface(
    checkpoint_path: Path,
    repo_id: str,
    name: str,
    description: str,
    metrics: dict,
) -> str:
    """Upload model to HuggingFace Hub."""
    from essence_wars.hub import upload_model

    url = upload_model(
        checkpoint_path=checkpoint_path,
        repo_id=repo_id,
        model_type="ppo",  # Default, could be detected
        metrics=metrics,
        training_config={"description": description},
    )
    return url


def add_to_leaderboard(
    leaderboard_path: Path,
    agent_id: str,
    name: str,
    author: str,
    model_type: str,
    description: str,
    metrics: dict,
    checkpoint_path: Path,
    model_url: str | None = None,
    tags: list[str] | None = None,
) -> None:
    """Add agent entry to leaderboard JSON."""
    # Load existing leaderboard
    with open(leaderboard_path) as f:
        data = json.load(f)

    # Check if agent already exists
    existing_ids = [a["id"] for a in data["agents"]]
    if agent_id in existing_ids:
        print(f"  Warning: Agent '{agent_id}' already exists, updating entry...")
        data["agents"] = [a for a in data["agents"] if a["id"] != agent_id]

    # Create new entry
    now = datetime.now(timezone.utc).isoformat()
    entry = {
        "id": agent_id,
        "name": name,
        "author": author,
        "model_type": model_type,
        "description": description,
        "elo_rating": metrics.get("elo_rating") or metrics.get("estimated_elo", 1200),
        "confidence_interval": 50,  # Default for new submissions
        "total_games": metrics.get("games_played", 0),
        "metrics": {
            "win_rate_vs_random": metrics.get("win_rate_vs_random"),
            "win_rate_vs_greedy": metrics.get("win_rate_vs_greedy"),
            "win_rate_vs_mcts50": metrics.get("win_rate_vs_mcts50"),
            "win_rate_vs_mcts100": metrics.get("win_rate_vs_mcts100"),
        },
        "submission": {
            "date": now,
            "source": "huggingface" if model_url else "local",
            "model_url": model_url,
            "checkpoint_path": str(checkpoint_path),
        },
        "tags": tags or ["community"],
        "is_baseline": False,
    }

    data["agents"].append(entry)
    data["metadata"]["last_updated"] = now

    # Save updated leaderboard
    with open(leaderboard_path, "w") as f:
        json.dump(data, f, indent=2)


def main():
    parser = argparse.ArgumentParser(
        description="Submit an agent to the Essence Wars leaderboard",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "--checkpoint",
        type=Path,
        required=True,
        help="Path to the .pt checkpoint file",
    )
    parser.add_argument(
        "--name",
        type=str,
        required=True,
        help="Display name for your agent",
    )
    parser.add_argument(
        "--author",
        type=str,
        default=None,
        help="Author name (default: extracted from repo-id or 'anonymous')",
    )
    parser.add_argument(
        "--repo-id",
        type=str,
        default=None,
        help="HuggingFace repo ID for upload (e.g., username/model-name)",
    )
    parser.add_argument(
        "--description",
        type=str,
        default="",
        help="Brief description of your agent",
    )
    parser.add_argument(
        "--model-type",
        type=str,
        choices=["ppo", "alphazero", "bc", "dqn", "other"],
        default="ppo",
        help="Type of model (default: ppo)",
    )
    parser.add_argument(
        "--tags",
        type=str,
        nargs="+",
        default=None,
        help="Tags for the agent (e.g., generalist specialist)",
    )
    parser.add_argument(
        "--full-eval",
        action="store_true",
        help="Run full benchmark (400 games) instead of quick eval (40 games)",
    )
    parser.add_argument(
        "--leaderboard",
        type=Path,
        default=Path("data/leaderboard/leaderboard.json"),
        help="Path to leaderboard JSON",
    )
    parser.add_argument(
        "--skip-upload",
        action="store_true",
        help="Skip HuggingFace upload (local submission only)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Validate and evaluate only, don't modify leaderboard",
    )
    args = parser.parse_args()

    print("=" * 60)
    print("Essence Wars Agent Submission")
    print("=" * 60)

    # 1. Validate checkpoint
    print(f"\n[1/5] Validating checkpoint: {args.checkpoint}")
    if not args.checkpoint.exists():
        print(f"  ERROR: Checkpoint not found: {args.checkpoint}")
        return 1

    valid, msg = validate_checkpoint(args.checkpoint)
    if not valid:
        print(f"  ERROR: {msg}")
        return 1
    print(f"  OK: {msg}")

    # 2. Run evaluation
    print(f"\n[2/5] Running evaluation...")
    if args.full_eval:
        metrics = full_evaluate(args.checkpoint)
    else:
        metrics = quick_evaluate(args.checkpoint)

    print(f"  Win rate vs Greedy: {metrics['win_rate_vs_greedy']:.1%}")
    print(f"  Win rate vs Random: {metrics['win_rate_vs_random']:.1%}")
    elo = metrics.get("elo_rating") or metrics.get("estimated_elo")
    print(f"  Estimated Elo: {elo:.0f}")

    # Check minimum performance
    if metrics["win_rate_vs_greedy"] < 0.1:
        print("\n  WARNING: Win rate vs Greedy is very low (<10%)")
        print("  Your agent may not be training correctly.")

    if args.dry_run:
        print("\n[DRY RUN] Stopping before upload/leaderboard update")
        return 0

    # 3. Upload to HuggingFace (optional)
    model_url = None
    if args.repo_id and not args.skip_upload:
        print(f"\n[3/5] Uploading to HuggingFace: {args.repo_id}")
        try:
            model_url = upload_to_huggingface(
                args.checkpoint,
                args.repo_id,
                args.name,
                args.description,
                metrics,
            )
            print(f"  Uploaded: {model_url}")
        except Exception as e:
            print(f"  ERROR: Upload failed: {e}")
            print("  Continuing with local submission...")
    else:
        print("\n[3/5] Skipping HuggingFace upload")

    # 4. Determine agent ID and author
    if args.repo_id:
        agent_id = args.repo_id
        author = args.author or args.repo_id.split("/")[0]
    else:
        agent_id = f"local/{args.name.lower().replace(' ', '-')}"
        author = args.author or "anonymous"

    # 5. Add to leaderboard
    print(f"\n[4/5] Adding to leaderboard: {args.leaderboard}")
    if not args.leaderboard.exists():
        print(f"  ERROR: Leaderboard not found: {args.leaderboard}")
        return 1

    add_to_leaderboard(
        leaderboard_path=args.leaderboard,
        agent_id=agent_id,
        name=args.name,
        author=author,
        model_type=args.model_type,
        description=args.description,
        metrics=metrics,
        checkpoint_path=args.checkpoint,
        model_url=model_url,
        tags=args.tags,
    )
    print("  Added entry to leaderboard.json")

    # 6. Regenerate markdown
    print("\n[5/5] Regenerating leaderboard markdown...")
    try:
        from generate_leaderboard import generate_markdown, load_leaderboard

        data = load_leaderboard(args.leaderboard)
        md = generate_markdown(data)
        md_path = Path("docs/leaderboard.md")
        md_path.write_text(md)
        print(f"  Generated: {md_path}")
    except Exception as e:
        print(f"  Warning: Could not regenerate markdown: {e}")

    # Summary
    print("\n" + "=" * 60)
    print("SUBMISSION COMPLETE")
    print("=" * 60)
    print(f"  Agent: {args.name}")
    print(f"  ID: {agent_id}")
    print(f"  Elo: {elo:.0f}")
    print(f"  vs Greedy: {metrics['win_rate_vs_greedy']:.1%}")
    if model_url:
        print(f"  HuggingFace: {model_url}")
    print("\nNext steps:")
    print("  1. Review your entry in docs/leaderboard.md")
    print("  2. Commit and push changes")
    print("  3. For official ranking, request full evaluation")

    return 0


if __name__ == "__main__":
    sys.exit(main())
