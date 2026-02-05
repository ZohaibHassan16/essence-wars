#!/usr/bin/env python3
"""
Token Art Generation Script for Essence Wars

Reads prompts from tokens.yaml and generates token art using FLUX via stable-diffusion.cpp.
Converts output to WebP and places in the correct location.

Usage:
    python scripts/generate-token-art.py --faction symbiote
    python scripts/generate-token-art.py --faction all
    python scripts/generate-token-art.py --token brass_cog
    python scripts/generate-token-art.py --list
    python scripts/generate-token-art.py --dry-run --faction argentum
"""

import argparse
import subprocess
import yaml
import sys
from pathlib import Path
from datetime import datetime

# Configuration
SD_CPP_PATH = Path.home() / "stable-diffusion.cpp"
MODELS_PATH = Path.home() / ".ai-assets/models/flux"
LORAS_PATH = Path.home() / ".ai-assets/loras"
OUTPUT_PATH = Path.home() / ".ai-assets/output/tokens"
PROJECT_ROOT = Path(__file__).parent.parent
PROMPTS_FILE = PROJECT_ROOT / "data/art/prompts/tokens.yaml"
TOKENS_OUTPUT = PROJECT_ROOT / "crates/essence-wars-ui/static/tokens"

# FLUX generation settings (same as card art for consistency)
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

# Faction list
FACTIONS = ["argentum", "symbiote", "obsidion", "neutral"]


def load_prompts() -> dict:
    """Load all token prompts from YAML file."""
    if not PROMPTS_FILE.exists():
        print(f"Error: Prompt file not found: {PROMPTS_FILE}")
        sys.exit(1)

    with open(PROMPTS_FILE, 'r') as f:
        return yaml.safe_load(f)


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


def get_output_path(token_id: str) -> Path:
    """Determine the correct output path for a token."""
    if token_id.startswith("generic_"):
        # Generic tokens go in generic/ folder with faction name
        faction = token_id.replace("generic_", "")
        return TOKENS_OUTPUT / "generic" / f"{faction}.webp"
    else:
        # Named tokens go in named/ folder
        return TOKENS_OUTPUT / "named" / f"{token_id}.webp"


def generate_image(token_id: str, token_data: dict, dry_run: bool = False) -> bool:
    """Generate a single token image using FLUX."""
    name = token_data.get('name', 'Unknown')
    prompt = token_data.get('prompt', '').strip()
    lora = token_data.get('lora', '')
    negative = token_data.get('negative', '')

    if not prompt:
        print(f"  Skipping {token_id} ({name}): No prompt defined")
        return False

    # Build full prompt with LoRA
    lora_tags = parse_lora(lora)
    full_prompt = f"{prompt} {lora_tags}".strip()

    # Output paths
    png_path = OUTPUT_PATH / f"{token_id}.png"
    webp_path = get_output_path(token_id)

    print(f"  Generating {token_id}: {name}")
    print(f"    Output: {webp_path}")

    if dry_run:
        print(f"    [DRY RUN] Would generate: {png_path}")
        print(f"    [DRY RUN] Prompt preview: {prompt[:100]}...")
        return True

    # Ensure output directories exist
    png_path.parent.mkdir(parents=True, exist_ok=True)
    webp_path.parent.mkdir(parents=True, exist_ok=True)

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

    # Add negative prompt if specified
    if negative:
        cmd.extend(["-n", negative])

    # Run generation
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
        if result.returncode != 0:
            print(f"    Error generating {token_id}: {result.stderr[:200]}")
            return False
    except subprocess.TimeoutExpired:
        print(f"    Timeout generating {token_id}")
        return False
    except FileNotFoundError:
        print(f"    Error: sd-cli not found at {SD_CPP_PATH / 'build/bin/sd-cli'}")
        print("    Make sure stable-diffusion.cpp is built.")
        return False
    except Exception as e:
        print(f"    Exception generating {token_id}: {e}")
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
        except FileNotFoundError:
            print("    Error: cwebp not found. Install with: sudo apt install webp")
            return False
    else:
        print(f"    PNG not created: {png_path}")
        return False


def generate_faction(faction: str, prompts: dict, dry_run: bool = False) -> tuple:
    """Generate all tokens for a faction."""
    print(f"\n{'='*60}")
    print(f"Generating {faction.upper()} tokens")
    print(f"{'='*60}")

    faction_tokens = prompts.get(faction, [])
    if not faction_tokens:
        print(f"  No tokens found for faction: {faction}")
        return 0, 0, 0

    success_count = 0
    fail_count = 0

    for token in faction_tokens:
        token_id = token.get('id')
        if not token_id:
            print(f"  Skipping token without ID")
            fail_count += 1
            continue

        if generate_image(token_id, token, dry_run):
            success_count += 1
        else:
            fail_count += 1

    return success_count, fail_count, 0


def generate_single_token(token_id: str, prompts: dict, dry_run: bool = False) -> bool:
    """Generate a single token by ID."""
    # Search all factions for the token
    for faction in FACTIONS:
        faction_tokens = prompts.get(faction, [])
        for token in faction_tokens:
            if token.get('id') == token_id:
                print(f"\n{'='*60}")
                print(f"Generating single token: {token_id}")
                print(f"{'='*60}")
                return generate_image(token_id, token, dry_run)

    print(f"Error: Token '{token_id}' not found in any faction")
    return False


def list_tokens(prompts: dict):
    """List all available tokens."""
    print("\nAvailable tokens:")
    print("="*60)

    for faction in FACTIONS:
        faction_tokens = prompts.get(faction, [])
        if faction_tokens:
            print(f"\n{faction.upper()}:")
            for token in faction_tokens:
                token_id = token.get('id', 'unknown')
                name = token.get('name', 'Unknown')
                output_path = get_output_path(token_id)
                exists = output_path.exists()
                status = "exists" if exists else "missing"
                print(f"  - {token_id}: {name} [{status}]")


def main():
    parser = argparse.ArgumentParser(description="Generate Essence Wars token art")
    parser.add_argument("--faction",
                       choices=FACTIONS + ["all"],
                       help="Faction to generate (or 'all' for all factions)")
    parser.add_argument("--token",
                       help="Generate single token by ID (e.g., 'brass_cog')")
    parser.add_argument("--list", action="store_true",
                       help="List all available tokens")
    parser.add_argument("--dry-run", action="store_true",
                       help="Show what would be generated without running")

    args = parser.parse_args()

    # Load prompts
    prompts = load_prompts()

    # Handle --list
    if args.list:
        list_tokens(prompts)
        return

    # Require either --faction or --token
    if not args.faction and not args.token:
        parser.print_help()
        print("\nError: Must specify --faction, --token, or --list")
        sys.exit(1)

    # Ensure output directories exist
    OUTPUT_PATH.mkdir(parents=True, exist_ok=True)
    (TOKENS_OUTPUT / "named").mkdir(parents=True, exist_ok=True)
    (TOKENS_OUTPUT / "generic").mkdir(parents=True, exist_ok=True)

    # Log start time
    start_time = datetime.now()
    print(f"Token Art Generation Started: {start_time.strftime('%Y-%m-%d %H:%M:%S')}")

    # Handle single token
    if args.token:
        success = generate_single_token(args.token, prompts, args.dry_run)
        total_success = 1 if success else 0
        total_fail = 0 if success else 1
    else:
        # Handle faction(s)
        total_success = 0
        total_fail = 0

        factions_to_generate = FACTIONS if args.faction == "all" else [args.faction]

        for faction in factions_to_generate:
            s, f, _ = generate_faction(faction, prompts, args.dry_run)
            total_success += s
            total_fail += f

    # Summary
    end_time = datetime.now()
    duration = end_time - start_time

    print(f"\n{'='*60}")
    print("GENERATION COMPLETE")
    print(f"{'='*60}")
    print(f"Success: {total_success}")
    print(f"Failed:  {total_fail}")
    print(f"Duration: {duration}")
    print(f"Finished: {end_time.strftime('%Y-%m-%d %H:%M:%S')}")


if __name__ == "__main__":
    main()
