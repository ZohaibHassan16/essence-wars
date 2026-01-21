#!/usr/bin/env python3
"""
Batch card art generation using local FLUX inference via stable-diffusion.cpp.

This script reads prompts from data/art/prompts.json and generates card art
using the FLUX model with stable-diffusion.cpp CLI.

Usage:
    # Generate drafts with Schnell (fast iteration, ~30s/image)
    uv run python python/scripts/generate_card_art.py --model schnell

    # Generate finals with Dev (high quality, ~2-3min/image)
    uv run python python/scripts/generate_card_art.py --model dev

    # Generate specific cards only
    uv run python python/scripts/generate_card_art.py --model schnell --cards 1057,2060,3055

    # Generate specific faction only
    uv run python python/scripts/generate_card_art.py --model schnell --faction argentum

    # Resume from a specific card (by ID)
    uv run python python/scripts/generate_card_art.py --model schnell --start-from 2015

    # Skip existing files (resume interrupted batch)
    uv run python python/scripts/generate_card_art.py --model schnell --skip-existing

    # Generate multiple variants per card
    uv run python python/scripts/generate_card_art.py --model schnell --variants 3

    # Custom output directory
    uv run python python/scripts/generate_card_art.py --model schnell \
        --output ~/my-art-output/

    # Dry run (show commands without executing)
    uv run python python/scripts/generate_card_art.py --model schnell --dry-run
"""

import argparse
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


# Default paths
DEFAULT_PROMPTS_FILE = Path(__file__).parent.parent.parent / "data" / "art" / "prompts.json"
DEFAULT_OUTPUT_DIR = Path.home() / ".ai-assets" / "output" / "essence-wars"
DEFAULT_SD_CPP_DIR = Path.home() / "stable-diffusion.cpp"
DEFAULT_MODELS_DIR = Path.home() / ".ai-assets" / "models" / "flux"
DEFAULT_LORAS_DIR = Path.home() / ".ai-assets" / "loras"


@dataclass
class FluxConfig:
    """Configuration for FLUX model inference."""
    model_name: str
    diffusion_model: Path
    steps: int
    description: str


# Model configurations
FLUX_CONFIGS = {
    "schnell": FluxConfig(
        model_name="schnell",
        diffusion_model=Path("flux-schnell-q4.gguf"),
        steps=4,
        description="Fast drafts (~30s/image)",
    ),
    "dev": FluxConfig(
        model_name="dev",
        diffusion_model=Path("flux-dev-q8.gguf"),
        steps=20,
        description="High quality finals (~2-3min/image)",
    ),
}


def load_prompts(prompts_file: Path) -> dict:
    """Load prompts from JSON file."""
    if not prompts_file.exists():
        print(f"Error: Prompts file not found: {prompts_file}")
        print("Run the prompt generation task first to create prompts.json")
        sys.exit(1)

    with open(prompts_file) as f:
        return json.load(f)


def sanitize_filename(name: str) -> str:
    """Convert card name to safe filename."""
    # Replace spaces and special characters
    safe = name.lower()
    safe = safe.replace(" ", "_")
    safe = safe.replace("'", "")
    safe = safe.replace("-", "_")
    safe = safe.replace(",", "")
    safe = safe.replace(":", "")
    # Remove any remaining unsafe characters
    safe = "".join(c for c in safe if c.isalnum() or c == "_")
    return safe


def build_output_path(
    card: dict,
    output_dir: Path,
    model_name: str,
    variant: int = 1,
) -> Path:
    """Build output path for a card image."""
    faction = card["faction"]
    card_id = card["id"]
    name = sanitize_filename(card["name"])

    # Organize by model type and faction
    subdir = "drafts" if model_name == "schnell" else "finals"
    faction_dir = output_dir / subdir / faction
    faction_dir.mkdir(parents=True, exist_ok=True)

    if variant > 1:
        filename = f"{card_id}_{name}_v{variant}.png"
    else:
        filename = f"{card_id}_{name}.png"

    return faction_dir / filename


def build_sd_command(
    card: dict,
    output_path: Path,
    flux_config: FluxConfig,
    models_dir: Path,
    loras_dir: Path,
    sd_cpp_dir: Path,
    seed: Optional[int] = None,
) -> list[str]:
    """Build the stable-diffusion.cpp CLI command."""

    # Build the full prompt with LoRA
    prompt = card["prompt"]
    lora = card.get("lora", "")
    if lora:
        prompt = f"{prompt} {lora}"

    # Build command
    cmd = [
        str(sd_cpp_dir / "build" / "bin" / "sd-cli"),
        "--diffusion-model", str(models_dir / flux_config.diffusion_model),
        "--vae", str(models_dir / "ae.safetensors"),
        "--clip_l", str(models_dir / "clip_l.safetensors"),
        "--t5xxl", str(models_dir / "t5-Q5_K_M.gguf"),
        "--lora-model-dir", str(loras_dir),
        "-p", prompt,
        "--cfg-scale", "1.0",
        "--sampling-method", "euler",
        "--steps", str(flux_config.steps),
        "-H", "896",
        "-W", "704",
        "-o", str(output_path),
    ]

    # Add negative prompt if present
    negative = card.get("negative", "")
    if negative:
        cmd.extend(["-n", negative])

    # Add seed if specified
    if seed is not None:
        cmd.extend(["--seed", str(seed)])

    return cmd


def run_generation(
    cmd: list[str],
    card: dict,
    output_path: Path,
    dry_run: bool = False,
) -> bool:
    """Run the generation command and return success status."""

    card_info = f"[{card['id']}] {card['name']} ({card['faction']})"

    if dry_run:
        print(f"\n{'='*60}")
        print(f"DRY RUN: {card_info}")
        print(f"Output: {output_path}")
        print(f"Command: {' '.join(cmd[:10])}...")
        return True

    print(f"\n{'='*60}")
    print(f"Generating: {card_info}")
    print(f"Output: {output_path}")

    start_time = time.time()

    try:
        # Run the command
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=600,  # 10 minute timeout
        )

        elapsed = time.time() - start_time

        if result.returncode == 0 and output_path.exists():
            size_kb = output_path.stat().st_size / 1024
            print(f"Success! Generated in {elapsed:.1f}s ({size_kb:.1f} KB)")
            return True
        else:
            print(f"Failed after {elapsed:.1f}s")
            if result.stderr:
                print(f"Error: {result.stderr[:500]}")
            return False

    except subprocess.TimeoutExpired:
        print("Error: Generation timed out after 10 minutes")
        return False
    except Exception as e:
        print(f"Error: {e}")
        return False


def filter_cards(
    cards: list[dict],
    card_ids: Optional[list[int]] = None,
    faction: Optional[str] = None,
    start_from: Optional[int] = None,
    commanders_only: bool = False,
) -> list[dict]:
    """Filter cards based on criteria."""

    filtered = cards

    # Filter by specific IDs
    if card_ids:
        filtered = [c for c in filtered if c["id"] in card_ids]

    # Filter by faction
    if faction:
        filtered = [c for c in filtered if c["faction"] == faction]

    # Filter commanders only
    if commanders_only:
        filtered = [c for c in filtered if c.get("is_commander", False)]

    # Start from specific card
    if start_from:
        found_start = False
        result = []
        for c in filtered:
            if c["id"] == start_from:
                found_start = True
            if found_start:
                result.append(c)
        filtered = result

    return filtered


def main():
    parser = argparse.ArgumentParser(
        description="Generate card art using local FLUX inference",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    # Required
    parser.add_argument(
        "--model",
        choices=["schnell", "dev"],
        required=True,
        help="FLUX model to use: schnell (fast drafts) or dev (high quality)",
    )

    # Filtering options
    parser.add_argument(
        "--cards",
        type=str,
        help="Comma-separated list of card IDs to generate (e.g., 1057,2060,3055)",
    )
    parser.add_argument(
        "--faction",
        choices=["argentum", "symbiote", "obsidion", "neutral"],
        help="Generate only cards from this faction",
    )
    parser.add_argument(
        "--start-from",
        type=int,
        help="Resume from this card ID",
    )
    parser.add_argument(
        "--commanders-only",
        action="store_true",
        help="Only generate commander cards (Tier 1)",
    )

    # Output options
    parser.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_OUTPUT_DIR,
        help=f"Output directory (default: {DEFAULT_OUTPUT_DIR})",
    )
    parser.add_argument(
        "--variants",
        type=int,
        default=1,
        help="Number of variants to generate per card (default: 1)",
    )
    parser.add_argument(
        "--skip-existing",
        action="store_true",
        help="Skip cards that already have generated images",
    )

    # Path overrides
    parser.add_argument(
        "--prompts",
        type=Path,
        default=DEFAULT_PROMPTS_FILE,
        help=f"Path to prompts.json (default: {DEFAULT_PROMPTS_FILE})",
    )
    parser.add_argument(
        "--sd-cpp-dir",
        type=Path,
        default=DEFAULT_SD_CPP_DIR,
        help=f"Path to stable-diffusion.cpp (default: {DEFAULT_SD_CPP_DIR})",
    )
    parser.add_argument(
        "--models-dir",
        type=Path,
        default=DEFAULT_MODELS_DIR,
        help=f"Path to FLUX models (default: {DEFAULT_MODELS_DIR})",
    )
    parser.add_argument(
        "--loras-dir",
        type=Path,
        default=DEFAULT_LORAS_DIR,
        help=f"Path to LoRA models (default: {DEFAULT_LORAS_DIR})",
    )

    # Execution options
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show commands without executing them",
    )
    parser.add_argument(
        "--seed",
        type=int,
        help="Fixed seed for reproducibility (default: random)",
    )

    args = parser.parse_args()

    # Validate paths
    if not args.dry_run:
        sd_cli = args.sd_cpp_dir / "build" / "bin" / "sd-cli"
        if not sd_cli.exists():
            print(f"Error: sd-cli not found at {sd_cli}")
            print("Make sure stable-diffusion.cpp is built")
            sys.exit(1)

        flux_config = FLUX_CONFIGS[args.model]
        model_path = args.models_dir / flux_config.diffusion_model
        if not model_path.exists():
            print(f"Error: FLUX model not found at {model_path}")
            sys.exit(1)

    # Load prompts
    data = load_prompts(args.prompts)
    cards = data["cards"]

    print(f"Loaded {len(cards)} card prompts from {args.prompts}")

    # Parse card IDs filter
    card_ids = None
    if args.cards:
        card_ids = [int(x.strip()) for x in args.cards.split(",")]

    # Filter cards
    cards = filter_cards(
        cards,
        card_ids=card_ids,
        faction=args.faction,
        start_from=args.start_from,
        commanders_only=args.commanders_only,
    )

    if not cards:
        print("No cards match the specified filters")
        sys.exit(1)

    # Get config
    flux_config = FLUX_CONFIGS[args.model]

    print(f"\nModel: {args.model} - {flux_config.description}")
    print(f"Cards to generate: {len(cards)}")
    print(f"Variants per card: {args.variants}")
    print(f"Output directory: {args.output}")

    if args.variants > 1:
        total_images = len(cards) * args.variants
    else:
        total_images = len(cards)

    # Estimate time
    time_per_image = 30 if args.model == "schnell" else 150  # seconds
    estimated_time = total_images * time_per_image
    hours = estimated_time // 3600
    minutes = (estimated_time % 3600) // 60
    print(f"Estimated time: {hours}h {minutes}m ({total_images} images)")

    if not args.dry_run:
        print("\nStarting generation in 3 seconds... (Ctrl+C to cancel)")
        time.sleep(3)

    # Generate
    success_count = 0
    skip_count = 0
    fail_count = 0

    for i, card in enumerate(cards):
        for variant in range(1, args.variants + 1):
            # Build output path
            output_path = build_output_path(
                card,
                args.output,
                args.model,
                variant if args.variants > 1 else 1,
            )

            # Skip if exists
            if args.skip_existing and output_path.exists():
                print(f"Skipping existing: {output_path.name}")
                skip_count += 1
                continue

            # Build command
            seed = args.seed
            if seed is not None and args.variants > 1:
                seed = args.seed + variant - 1  # Different seed per variant

            cmd = build_sd_command(
                card,
                output_path,
                flux_config,
                args.models_dir,
                args.loras_dir,
                args.sd_cpp_dir,
                seed=seed,
            )

            # Progress
            progress = f"[{i+1}/{len(cards)}]"
            if args.variants > 1:
                progress = f"[{i+1}/{len(cards)} v{variant}]"

            print(f"\n{progress}", end="")

            # Run
            if run_generation(cmd, card, output_path, args.dry_run):
                success_count += 1
            else:
                fail_count += 1

    # Summary
    print(f"\n{'='*60}")
    print("Generation Complete!")
    print(f"  Success: {success_count}")
    print(f"  Skipped: {skip_count}")
    print(f"  Failed:  {fail_count}")
    print(f"\nOutput directory: {args.output}")


if __name__ == "__main__":
    main()
