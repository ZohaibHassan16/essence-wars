#!/usr/bin/env python3
"""Upload a trained model to Huggingface Hub.

Usage:
    # Upload AlphaZero model
    uv run python python/scripts/upload_model.py \\
        --checkpoint models/alphazero_v1.pt \\
        --repo username/essence-wars-alphazero \\
        --type alphazero

    # Upload with metrics from evaluation
    uv run python python/scripts/upload_model.py \\
        --checkpoint models/ppo_generalist.pt \\
        --repo username/essence-wars-ppo \\
        --type ppo \\
        --metrics results.json

    # Private repository
    uv run python python/scripts/upload_model.py \\
        --checkpoint models/my_model.pt \\
        --repo username/private-model \\
        --private

Note: Set HF_TOKEN environment variable or use `huggingface-cli login` first.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Upload a trained Essence Wars model to Huggingface Hub",
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
        "--repo",
        type=str,
        required=True,
        help="Huggingface repository ID (e.g., username/model-name)",
    )
    parser.add_argument(
        "--type",
        type=str,
        choices=["ppo", "alphazero", "bc"],
        default="alphazero",
        help="Model type (default: alphazero)",
    )
    parser.add_argument(
        "--metrics",
        type=Path,
        default=None,
        help="JSON file with evaluation metrics",
    )
    parser.add_argument(
        "--config",
        type=Path,
        default=None,
        help="JSON file with training configuration",
    )
    parser.add_argument(
        "--private",
        action="store_true",
        help="Make the repository private",
    )
    parser.add_argument(
        "--token",
        type=str,
        default=None,
        help="Huggingface token (or set HF_TOKEN env var)",
    )
    args = parser.parse_args()

    # Validate checkpoint exists
    if not args.checkpoint.exists():
        print(f"Error: Checkpoint not found: {args.checkpoint}")
        return 1

    # Load metrics if provided
    metrics = None
    if args.metrics:
        if not args.metrics.exists():
            print(f"Error: Metrics file not found: {args.metrics}")
            return 1
        with open(args.metrics) as f:
            metrics = json.load(f)

    # Load config if provided
    training_config = None
    if args.config:
        if not args.config.exists():
            print(f"Error: Config file not found: {args.config}")
            return 1
        with open(args.config) as f:
            training_config = json.load(f)

    # Import here to give better error messages
    try:
        from essence_wars.hub import upload_model
    except ImportError as e:
        print(f"Error: Failed to import essence_wars.hub: {e}")
        print("Make sure huggingface_hub is installed: pip install huggingface_hub")
        return 1

    print(f"Uploading {args.checkpoint} to {args.repo}...")

    try:
        url = upload_model(
            checkpoint_path=args.checkpoint,
            repo_id=args.repo,
            model_type=args.type,
            training_config=training_config,
            metrics=metrics,
            private=args.private,
            token=args.token,
        )
        print("Successfully uploaded model!")
        print(f"View at: {url}")
        return 0

    except Exception as e:
        print(f"Error uploading model: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
