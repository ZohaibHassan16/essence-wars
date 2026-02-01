#!/usr/bin/env python3
"""
Commander Portrait Generation Script for Essence Wars

Reads prompts from commanders.yaml and generates portrait art using FLUX via stable-diffusion.cpp.
Converts output to WebP and places in the correct location.

Usage:
    python scripts/generate-commander-art.py --faction argentum
    python scripts/generate-commander-art.py --faction all
    python scripts/generate-commander-art.py --commander 5000
    python scripts/generate-commander-art.py --commander 5008 --dry-run
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
OUTPUT_PATH = Path.home() / ".ai-assets/output/portraits"
PROJECT_ROOT = Path(__file__).parent.parent
PROMPTS_PATH = PROJECT_ROOT / "data/art/prompts/commanders.yaml"
PORTRAITS_OUTPUT = PROJECT_ROOT / "crates/essence-wars-ui/static/portrait"

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

# Commander ID ranges by faction
FACTION_RANGES = {
    "argentum": (5000, 5003),
    "symbiote": (5004, 5007),
    "obsidion": (5008, 5011),
}


def load_prompts() -> dict:
    """Load commander prompts from YAML file."""
    if not PROMPTS_PATH.exists():
        print(f"Error: Prompt file not found: {PROMPTS_PATH}")
        sys.exit(1)

    with open(PROMPTS_PATH, 'r') as f:
        data = yaml.safe_load(f)

    # Index by commander ID for easy lookup
    prompts = {}
    for commander in data.get('commanders', []):
        prompts[commander['id']] = commander

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


def get_filename(name: str) -> str:
    """Convert commander name to snake_case filename."""
    # "The High Artificer" -> "the_high_artificer"
    return name.lower().replace(' ', '_')


def generate_portrait(commander_id: int, prompt_data: dict, dry_run: bool = False) -> bool:
    """Generate a single commander portrait using FLUX."""
    name = prompt_data.get('name', 'Unknown')
    faction = prompt_data.get('faction', 'unknown')
    prompt = prompt_data.get('prompt', '').strip()
    lora = prompt_data.get('lora', '')
    negative = prompt_data.get('negative', '')

    if not prompt:
        print(f"  Skipping {commander_id} ({name}): No prompt defined")
        return False

    # Build full prompt with LoRA
    lora_tags = parse_lora(lora)
    full_prompt = f"{prompt} {lora_tags}".strip()

    # Build negative prompt
    full_negative = f"no text, no words, no letters, {negative}".strip()

    # Output paths
    filename = get_filename(name)
    png_path = OUTPUT_PATH / f"{filename}.png"
    webp_path = PORTRAITS_OUTPUT / f"{filename}.webp"

    print(f"  Generating {commander_id}: {name} ({faction})")

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
        "-n", full_negative,
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
            print(f"    Error generating {commander_id}: {result.stderr[:200]}")
            return False
    except subprocess.TimeoutExpired:
        print(f"    Timeout generating {commander_id}")
        return False
    except Exception as e:
        print(f"    Exception generating {commander_id}: {e}")
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


def generate_faction(faction: str, prompts: dict, dry_run: bool = False) -> tuple:
    """Generate all commander portraits for a faction."""
    print(f"\n{'='*60}")
    print(f"Generating {faction.upper()} faction commanders")
    print(f"{'='*60}")

    # Determine ID range
    faction_start, faction_end = FACTION_RANGES.get(faction, (0, 0))
    if faction_start == 0:
        print(f"Error: Unknown faction {faction}")
        return 0, 0, 0

    success_count = 0
    fail_count = 0
    skip_count = 0

    for commander_id in range(faction_start, faction_end + 1):
        if commander_id not in prompts:
            print(f"  Skipping {commander_id}: Not in prompt file")
            skip_count += 1
            continue

        if generate_portrait(commander_id, prompts[commander_id], dry_run):
            success_count += 1
        else:
            fail_count += 1

    return success_count, fail_count, skip_count


def generate_single_commander(commander_id: int, prompts: dict, dry_run: bool = False) -> tuple:
    """Generate a single commander portrait."""
    print(f"\n{'='*60}")
    print(f"Generating Commander {commander_id}")
    print(f"{'='*60}")

    if commander_id not in prompts:
        print(f"Error: Commander {commander_id} not found in prompt file")
        return 0, 1, 0

    if generate_portrait(commander_id, prompts[commander_id], dry_run):
        return 1, 0, 0
    else:
        return 0, 1, 0


def main():
    parser = argparse.ArgumentParser(description="Generate Essence Wars commander portraits")
    parser.add_argument("--faction",
                       choices=["argentum", "symbiote", "obsidion", "all"],
                       help="Faction to generate (or 'all' for all factions)")
    parser.add_argument("--commander", type=int,
                       help="Generate single commander by ID (5000-5011)")
    parser.add_argument("--dry-run", action="store_true",
                       help="Show what would be generated without running")

    args = parser.parse_args()

    # Validate arguments
    if not args.faction and not args.commander:
        parser.error("Must specify either --faction or --commander")
    if args.faction and args.commander:
        parser.error("Cannot specify both --faction and --commander")

    # Ensure output directories exist
    OUTPUT_PATH.mkdir(parents=True, exist_ok=True)
    PORTRAITS_OUTPUT.mkdir(parents=True, exist_ok=True)

    # Load prompts
    prompts = load_prompts()

    # Log start time
    start_time = datetime.now()
    print(f"Commander Portrait Generation Started: {start_time.strftime('%Y-%m-%d %H:%M:%S')}")

    total_success = 0
    total_fail = 0
    total_skip = 0

    # Generate based on arguments
    if args.commander:
        s, f, sk = generate_single_commander(args.commander, prompts, args.dry_run)
        total_success += s
        total_fail += f
        total_skip += sk
    elif args.faction == "all":
        for faction in FACTION_RANGES.keys():
            s, f, sk = generate_faction(faction, prompts, args.dry_run)
            total_success += s
            total_fail += f
            total_skip += sk
    else:
        total_success, total_fail, total_skip = generate_faction(
            args.faction, prompts, args.dry_run
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
