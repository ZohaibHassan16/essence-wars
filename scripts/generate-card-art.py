#!/usr/bin/env python3
"""
Card Art Generation Script for Essence Wars

Reads prompts from YAML files and generates card art using FLUX via stable-diffusion.cpp.
Converts output to WebP and places in the correct location.

Usage:
    python scripts/generate-card-art.py --faction symbiote
    python scripts/generate-card-art.py --faction symbiote --start 2000 --end 2010
    python scripts/generate-card-art.py --faction argentum --card 1050
    python scripts/generate-card-art.py --faction all
"""

import argparse
import subprocess
import yaml
import os
import sys
from pathlib import Path
from datetime import datetime

# Configuration
SD_CPP_PATH = Path.home() / "stable-diffusion.cpp"
MODELS_PATH = Path.home() / ".ai-assets/models/flux"
LORAS_PATH = Path.home() / ".ai-assets/loras"
OUTPUT_PATH = Path.home() / ".ai-assets/output/cards"
PROJECT_ROOT = Path(__file__).parent.parent
PROMPTS_PATH = PROJECT_ROOT / "data/art/prompts/core_set"
CARDS_OUTPUT = PROJECT_ROOT / "crates/essence-wars-ui/static/cards/core_set"

# FLUX generation settings
FLUX_SETTINGS = {
    "model": MODELS_PATH / "flux-dev-q8.gguf",
    "vae": MODELS_PATH / "ae.safetensors",
    "clip_l": MODELS_PATH / "clip_l.safetensors",
    "t5xxl": MODELS_PATH / "t5-Q5_K_M.gguf",
    "cfg_scale": "1.0",
    "sampling_method": "euler",
    "steps": "20",
    "height": "896",
    "width": "704",
}

# Faction ID ranges (expanded to include New Horizons expansion cards)
FACTION_RANGES = {
    "argentum": (1000, 1077),
    "symbiote": (2000, 2076),
    "obsidion": (3000, 3076),
    "neutral": (4000, 4076),
}


def load_prompts(faction: str) -> dict:
    """Load prompts from YAML file for a faction."""
    yaml_path = PROMPTS_PATH / f"{faction}.yaml"
    if not yaml_path.exists():
        print(f"Error: Prompt file not found: {yaml_path}")
        sys.exit(1)

    with open(yaml_path, 'r') as f:
        data = yaml.safe_load(f)

    # Index by card ID for easy lookup
    prompts = {}
    for card in data.get('cards', []):
        prompts[card['id']] = card

    return prompts


def parse_lora(lora_string: str) -> str:
    """Convert lora string format to sd-cli format."""
    # Input: "frazetta:0.5, classical-painting:0.5"
    # Output: "<lora:frazetta:0.5> <lora:classical-painting:0.5>"
    if not lora_string:
        return ""

    parts = []
    for lora in lora_string.split(','):
        lora = lora.strip()
        if ':' in lora:
            name, weight = lora.split(':')
            parts.append(f"<lora:{name.strip()}:{weight.strip()}>")

    return ' '.join(parts)


def generate_image(card_id: int, prompt_data: dict, dry_run: bool = False) -> bool:
    """Generate a single card image using FLUX."""
    name = prompt_data.get('name', 'Unknown')
    prompt = prompt_data.get('prompt', '').strip()
    lora = prompt_data.get('lora', '')
    negative = prompt_data.get('negative', '')

    if not prompt:
        print(f"  Skipping {card_id} ({name}): No prompt defined")
        return False

    # Build full prompt with LoRA
    lora_tags = parse_lora(lora)
    full_prompt = f"{prompt} {lora_tags}".strip()

    # Output paths
    png_path = OUTPUT_PATH / f"{card_id}_{name.lower().replace(' ', '_')}.png"
    webp_path = CARDS_OUTPUT / f"{card_id}.webp"

    print(f"  Generating {card_id}: {name}")

    if dry_run:
        print(f"    [DRY RUN] Would generate: {png_path}")
        print(f"    [DRY RUN] Would convert to: {webp_path}")
        return True

    # Build sd-cli command
    cmd = [
        str(SD_CPP_PATH / "build/bin/sd-cli"),
        "--diffusion-model", str(FLUX_SETTINGS["model"]),
        "--vae", str(FLUX_SETTINGS["vae"]),
        "--clip_l", str(FLUX_SETTINGS["clip_l"]),
        "--t5xxl", str(FLUX_SETTINGS["t5xxl"]),
        "--lora-model-dir", str(LORAS_PATH),
        "-p", full_prompt,
        "--cfg-scale", FLUX_SETTINGS["cfg_scale"],
        "--sampling-method", FLUX_SETTINGS["sampling_method"],
        "--steps", FLUX_SETTINGS["steps"],
        "-H", FLUX_SETTINGS["height"],
        "-W", FLUX_SETTINGS["width"],
        "-o", str(png_path),
    ]

    # Run generation
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
        if result.returncode != 0:
            print(f"    Error generating {card_id}: {result.stderr[:200]}")
            return False
    except subprocess.TimeoutExpired:
        print(f"    Timeout generating {card_id}")
        return False
    except Exception as e:
        print(f"    Exception generating {card_id}: {e}")
        return False

    # Convert to WebP
    if png_path.exists():
        webp_cmd = ["cwebp", "-q", "95", str(png_path), "-o", str(webp_path)]
        try:
            subprocess.run(webp_cmd, capture_output=True, check=True)
            print(f"    Created: {webp_path}")
            return True
        except subprocess.CalledProcessError as e:
            print(f"    Error converting to WebP: {e}")
            return False
    else:
        print(f"    PNG not created: {png_path}")
        return False


def generate_faction(faction: str, start_id: int = None, end_id: int = None,
                    single_card: int = None, dry_run: bool = False) -> tuple:
    """Generate all cards for a faction within the specified range."""
    print(f"\n{'='*60}")
    print(f"Generating {faction.upper()} faction cards")
    print(f"{'='*60}")

    prompts = load_prompts(faction)

    # Determine ID range
    faction_start, faction_end = FACTION_RANGES.get(faction, (0, 0))

    if single_card is not None:
        ids_to_generate = [single_card]
    else:
        start = start_id if start_id is not None else faction_start
        end = end_id if end_id is not None else faction_end
        ids_to_generate = range(start, end + 1)

    success_count = 0
    fail_count = 0
    skip_count = 0

    for card_id in ids_to_generate:
        if card_id not in prompts:
            print(f"  Skipping {card_id}: Not in prompt file")
            skip_count += 1
            continue

        if generate_image(card_id, prompts[card_id], dry_run):
            success_count += 1
        else:
            fail_count += 1

    return success_count, fail_count, skip_count


def main():
    parser = argparse.ArgumentParser(description="Generate Essence Wars card art")
    parser.add_argument("--faction", required=True,
                       choices=["argentum", "symbiote", "obsidion", "neutral", "all"],
                       help="Faction to generate (or 'all' for all factions)")
    parser.add_argument("--start", type=int, help="Starting card ID")
    parser.add_argument("--end", type=int, help="Ending card ID")
    parser.add_argument("--card", type=int, help="Generate single card by ID")
    parser.add_argument("--dry-run", action="store_true",
                       help="Show what would be generated without running")

    args = parser.parse_args()

    # Ensure output directories exist
    OUTPUT_PATH.mkdir(parents=True, exist_ok=True)
    CARDS_OUTPUT.mkdir(parents=True, exist_ok=True)

    # Log start time
    start_time = datetime.now()
    print(f"Card Art Generation Started: {start_time.strftime('%Y-%m-%d %H:%M:%S')}")

    total_success = 0
    total_fail = 0
    total_skip = 0

    if args.faction == "all":
        for faction in FACTION_RANGES.keys():
            s, f, sk = generate_faction(faction, args.start, args.end, args.card, args.dry_run)
            total_success += s
            total_fail += f
            total_skip += sk
    else:
        total_success, total_fail, total_skip = generate_faction(
            args.faction, args.start, args.end, args.card, args.dry_run
        )

    # Summary
    end_time = datetime.now()
    duration = end_time - start_time

    print(f"\n{'='*60}")
    print("GENERATION COMPLETE")
    print(f"{'='*60}")
    print(f"Success: {total_success}")
    print(f"Failed:  {total_fail}")
    print(f"Skipped: {total_skip}")
    print(f"Duration: {duration}")
    print(f"Finished: {end_time.strftime('%Y-%m-%d %H:%M:%S')}")


if __name__ == "__main__":
    main()
