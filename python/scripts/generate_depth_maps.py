#!/usr/bin/env python3
"""
Generate depth maps for card art using Marigold depth estimation.

Depth maps are used for the parallax shader effect in the Bevy 3D client,
creating the "window into another dimension" illusion on gem tokens.

Usage:
    # Generate depth maps for all card art in the repo
    uv run python python/scripts/generate_depth_maps.py

    # Generate for specific faction only
    uv run python python/scripts/generate_depth_maps.py --faction argentum

    # Generate for specific cards
    uv run python python/scripts/generate_depth_maps.py --cards 1057,2060,3055

    # Skip existing depth maps
    uv run python python/scripts/generate_depth_maps.py --skip-existing

    # Use custom input/output directories
    uv run python python/scripts/generate_depth_maps.py \
        --input ~/.ai-assets/output/essence-wars/finals/ \
        --output ~/.ai-assets/output/essence-wars/depth_maps/

    # Dry run
    uv run python python/scripts/generate_depth_maps.py --dry-run

Requirements:
    pip install diffusers torch torchvision pillow
"""

import argparse
import sys
import time
from pathlib import Path

import torch
from PIL import Image


# Default paths
DEFAULT_INPUT_DIR = Path(__file__).parent.parent.parent / "crates" / "essence-wars-3d" / "assets" / "textures" / "cards"
DEFAULT_OUTPUT_DIR = DEFAULT_INPUT_DIR  # Depth maps go alongside card art


def load_marigold_pipeline(device: str = "cuda"):
    """Load the Marigold depth estimation pipeline."""
    from diffusers import MarigoldDepthPipeline

    print("Loading Marigold depth estimation model...")
    print("(This may take a moment on first run to download the model)")

    pipe = MarigoldDepthPipeline.from_pretrained(
        "prs-eth/marigold-depth-lcm-v1-0",
        torch_dtype=torch.float16 if device == "cuda" else torch.float32,
        variant="fp16" if device == "cuda" else None,
    )
    pipe = pipe.to(device)

    # Enable memory efficient attention if available
    if device == "cuda":
        try:
            pipe.enable_xformers_memory_efficient_attention()
            print("Enabled xformers memory efficient attention")
        except Exception:
            pass  # xformers not available, that's fine

    print(f"Model loaded on {device}")
    return pipe


def generate_depth_map(
    pipe,
    image_path: Path,
    output_path: Path,
    dry_run: bool = False,
) -> bool:
    """Generate a depth map for a single image."""

    if dry_run:
        print(f"  DRY RUN: {image_path.name} -> {output_path.name}")
        return True

    try:
        # Load image
        image = Image.open(image_path).convert("RGB")

        # Generate depth map
        # Marigold LCM is fast - only needs 1-4 steps
        depth = pipe(
            image,
            num_inference_steps=4,
            ensemble_size=1,
        )

        # Get the depth map as a PIL image
        depth_image = pipe.image_processor.visualize_depth(depth.prediction)[0]

        # Save as grayscale PNG
        depth_image = depth_image.convert("L")
        depth_image.save(output_path)

        return True

    except Exception as e:
        print(f"  Error: {e}")
        return False


def find_card_images(
    input_dir: Path,
    faction: str | None = None,
    card_ids: list[int] | None = None,
) -> list[tuple[Path, Path]]:
    """Find card images and their output depth map paths."""

    results = []

    factions = [faction] if faction else ["argentum", "symbiote", "obsidion", "neutral"]

    for f in factions:
        faction_dir = input_dir / f
        if not faction_dir.exists():
            continue

        for image_path in sorted(faction_dir.glob("*.png")):
            # Skip existing depth maps
            if "_depth" in image_path.stem:
                continue

            # Extract card ID from filename
            try:
                card_id = int(image_path.stem.split("_")[0])
            except ValueError:
                continue

            # Filter by card IDs if specified
            if card_ids and card_id not in card_ids:
                continue

            # Output path: same directory, add _depth suffix
            output_path = image_path.parent / f"{image_path.stem}_depth.png"

            results.append((image_path, output_path))

    return results


def main():
    parser = argparse.ArgumentParser(
        description="Generate depth maps for card art using Marigold",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )

    parser.add_argument(
        "--input",
        type=Path,
        default=DEFAULT_INPUT_DIR,
        help=f"Input directory with card art (default: {DEFAULT_INPUT_DIR})",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Output directory for depth maps (default: same as input)",
    )
    parser.add_argument(
        "--faction",
        choices=["argentum", "symbiote", "obsidion", "neutral"],
        help="Generate only for this faction",
    )
    parser.add_argument(
        "--cards",
        type=str,
        help="Comma-separated list of card IDs (e.g., 1057,2060,3055)",
    )
    parser.add_argument(
        "--skip-existing",
        action="store_true",
        help="Skip images that already have depth maps",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be generated without running",
    )
    parser.add_argument(
        "--device",
        choices=["cuda", "cpu"],
        default="cuda" if torch.cuda.is_available() else "cpu",
        help="Device to use (default: cuda if available)",
    )

    args = parser.parse_args()

    # Parse card IDs
    card_ids = None
    if args.cards:
        card_ids = [int(x.strip()) for x in args.cards.split(",")]

    # Find images to process
    images = find_card_images(args.input, args.faction, card_ids)

    if not images:
        print("No card images found matching criteria")
        sys.exit(1)

    # Filter existing if requested
    if args.skip_existing:
        images = [(i, o) for i, o in images if not o.exists()]

    if not images:
        print("All depth maps already exist (use without --skip-existing to regenerate)")
        sys.exit(0)

    print(f"Found {len(images)} images to process")
    print(f"Device: {args.device}")

    if args.dry_run:
        print("\nDry run - showing planned operations:")
        for image_path, output_path in images:
            print(f"  {image_path.name} -> {output_path.name}")
        return

    # Load model
    pipe = load_marigold_pipeline(args.device)

    # Process images
    success_count = 0
    fail_count = 0

    print(f"\nGenerating depth maps...")
    start_time = time.time()

    for i, (image_path, output_path) in enumerate(images):
        print(f"[{i+1}/{len(images)}] {image_path.name}", end=" ")

        img_start = time.time()
        if generate_depth_map(pipe, image_path, output_path, args.dry_run):
            elapsed = time.time() - img_start
            size_kb = output_path.stat().st_size / 1024 if output_path.exists() else 0
            print(f"-> {output_path.name} ({elapsed:.1f}s, {size_kb:.0f}KB)")
            success_count += 1
        else:
            fail_count += 1

    total_time = time.time() - start_time

    print(f"\n{'='*60}")
    print(f"Depth Map Generation Complete!")
    print(f"  Success: {success_count}")
    print(f"  Failed:  {fail_count}")
    print(f"  Time:    {total_time:.1f}s ({total_time/len(images):.1f}s per image)")


if __name__ == "__main__":
    main()
