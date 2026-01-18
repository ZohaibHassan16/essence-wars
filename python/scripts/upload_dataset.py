#!/usr/bin/env python3
"""Upload a dataset to Huggingface Hub.

Usage:
    # Upload MCTS self-play dataset
    uv run python python/scripts/upload_dataset.py \\
        --dataset data/datasets/mcts_100k.jsonl.gz \\
        --repo username/essence-wars-mcts-100k

    # Upload with metadata
    uv run python python/scripts/upload_dataset.py \\
        --dataset data/datasets/mcts_100k.jsonl.gz \\
        --repo username/essence-wars-mcts-100k \\
        --metadata metadata.json

    # Private repository
    uv run python python/scripts/upload_dataset.py \\
        --dataset data/datasets/my_data.jsonl.gz \\
        --repo username/private-dataset \\
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
        description="Upload an Essence Wars dataset to Huggingface Hub",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    parser.add_argument(
        "--dataset",
        type=Path,
        required=True,
        help="Path to the dataset file (.jsonl or .jsonl.gz)",
    )
    parser.add_argument(
        "--repo",
        type=str,
        required=True,
        help="Huggingface repository ID (e.g., username/dataset-name)",
    )
    parser.add_argument(
        "--filename",
        type=str,
        default=None,
        help="Name for the file in the repo (default: original filename)",
    )
    parser.add_argument(
        "--metadata",
        type=Path,
        default=None,
        help="JSON file with dataset metadata",
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

    # Validate dataset exists
    if not args.dataset.exists():
        print(f"Error: Dataset not found: {args.dataset}")
        return 1

    # Determine filename
    filename = args.filename or args.dataset.name

    # Load metadata if provided
    metadata = None
    if args.metadata:
        if not args.metadata.exists():
            print(f"Error: Metadata file not found: {args.metadata}")
            return 1
        with open(args.metadata) as f:
            metadata = json.load(f)

    # Auto-generate basic metadata if not provided
    if metadata is None:
        import gzip

        # Get file size
        file_size = args.dataset.stat().st_size
        file_size_mb = file_size / (1024 * 1024)

        # Count lines (games) if reasonable size
        num_games: int | None = None
        if file_size_mb < 500:  # Only count for <500MB files
            try:
                if args.dataset.suffix == ".gz":
                    with gzip.open(args.dataset, "rt") as f:
                        num_games = sum(1 for _ in f)
                else:
                    with open(args.dataset) as f:
                        num_games = sum(1 for _ in f)
            except Exception:
                pass

        metadata = {
            "file_size_mb": f"{file_size_mb:.1f}",
            "num_games": num_games if num_games is not None else "unknown",
            "format": "jsonl.gz" if args.dataset.suffix == ".gz" else "jsonl",
        }

    # Import here to give better error messages
    try:
        from essence_wars.hub import upload_dataset
    except ImportError as e:
        print(f"Error: Failed to import essence_wars.hub: {e}")
        print("Make sure huggingface_hub is installed: pip install huggingface_hub")
        return 1

    print(f"Uploading {args.dataset} to {args.repo}...")
    if metadata:
        print(f"Metadata: {json.dumps(metadata, indent=2)}")

    try:
        url = upload_dataset(
            dataset_path=args.dataset,
            repo_id=args.repo,
            filename=filename,
            metadata=metadata,
            private=args.private,
            token=args.token,
        )
        print("Successfully uploaded dataset!")
        print(f"View at: {url}")
        return 0

    except Exception as e:
        print(f"Error uploading dataset: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
