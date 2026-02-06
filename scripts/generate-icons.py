#!/usr/bin/env python3
"""
Icon Generation Script for Essence Wars

Generates app icons using FLUX. Outputs square 512x512 images with solid backgrounds
that can be processed to remove backgrounds and resize to various icon sizes.

Usage:
    uv run python scripts/generate-icons.py --all
    uv run python scripts/generate-icons.py --id essence_crystal
    uv run python scripts/generate-icons.py --list
    uv run python scripts/generate-icons.py --all --schnell

Post-processing:
    uv sync --group art  # Install art dependencies (Pillow, rembg)
    uv run python scripts/process-icons.py --input ~/.ai-assets/output/icons/essence_crystal.png
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
OUTPUT_PATH = Path.home() / ".ai-assets/output/icons"
PROJECT_ROOT = Path(__file__).parent.parent
PROMPTS_FILE = PROJECT_ROOT / "data/art/prompts/icons.yaml"

# FLUX generation settings for icons (square)
FLUX_DEV_SETTINGS = {
    "model": MODELS_PATH / "flux-dev-q8.gguf",
    "vae": MODELS_PATH / "ae.safetensors",
    "clip_l": MODELS_PATH / "clip_l.safetensors",
    "t5xxl": MODELS_PATH / "t5-Q5_K_M.gguf",
    "cfg_scale": "1.0",
    "sampling_method": "euler",
    "steps": "20",
    "height": "512",
    "width": "512",
}

# FLUX Schnell settings for fast iteration
FLUX_SCHNELL_SETTINGS = {
    "model": MODELS_PATH / "flux-schnell-q4.gguf",
    "vae": MODELS_PATH / "ae.safetensors",
    "clip_l": MODELS_PATH / "clip_l.safetensors",
    "t5xxl": MODELS_PATH / "t5-Q5_K_M.gguf",
    "cfg_scale": "1.0",
    "sampling_method": "euler",
    "steps": "4",
    "height": "512",
    "width": "512",
}


def load_prompts() -> list:
    """Load icon prompts from YAML file."""
    if not PROMPTS_FILE.exists():
        print(f"Error: Prompt file not found: {PROMPTS_FILE}")
        sys.exit(1)

    with open(PROMPTS_FILE, 'r') as f:
        data = yaml.safe_load(f)

    return data.get('icons', [])


def generate_icon(icon_data: dict, flux_settings: dict, dry_run: bool = False, use_schnell: bool = False) -> bool:
    """Generate a single icon image using FLUX."""
    icon_id = icon_data.get('id', 'unknown')
    name = icon_data.get('name', 'Unknown')
    prompt = icon_data.get('prompt', '').strip()
    negative = icon_data.get('negative', '')

    if not prompt:
        print(f"  Skipping {icon_id} ({name}): No prompt defined")
        return False

    # Output path
    png_path = OUTPUT_PATH / f"{icon_id}.png"

    model_name = "FLUX Schnell" if use_schnell else "FLUX Dev"
    print(f"  Generating {icon_id}: {name}")
    print(f"    Model: {model_name} ({flux_settings['steps']} steps)")
    print(f"    Output: {png_path}")
    print(f"    Resolution: {flux_settings['width']}x{flux_settings['height']}")

    if dry_run:
        print(f"    [DRY RUN] Would generate: {png_path}")
        print(f"    [DRY RUN] Prompt preview: {prompt[:80]}...")
        return True

    # Ensure output directory exists
    png_path.parent.mkdir(parents=True, exist_ok=True)

    # Build sd-cli command
    cmd = [
        str(SD_CPP_PATH / "build/bin/sd-cli"),
        "--diffusion-model", str(flux_settings["model"]),
        "--vae", str(flux_settings["vae"]),
        "--clip_l", str(flux_settings["clip_l"]),
        "--t5xxl", str(flux_settings["t5xxl"]),
        "--lora-model-dir", str(LORAS_PATH),
        "-p", prompt,
        "--cfg-scale", flux_settings["cfg_scale"],
        "--sampling-method", flux_settings["sampling_method"],
        "--steps", flux_settings["steps"],
        "-H", flux_settings["height"],
        "-W", flux_settings["width"],
        "-o", str(png_path),
    ]

    # Add negative prompt if specified
    if negative:
        cmd.extend(["-n", negative])

    # Run generation
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
        if result.returncode != 0:
            print(f"    Error generating {icon_id}: {result.stderr[:200]}")
            return False
    except subprocess.TimeoutExpired:
        print(f"    Timeout generating {icon_id}")
        return False
    except FileNotFoundError:
        print(f"    Error: sd-cli not found at {SD_CPP_PATH / 'build/bin/sd-cli'}")
        return False
    except Exception as e:
        print(f"    Exception generating {icon_id}: {e}")
        return False

    if png_path.exists():
        print(f"    Created: {png_path}")
        return True
    else:
        print(f"    PNG not created: {png_path}")
        return False


def list_icons(icons: list):
    """List all available icons."""
    print("\nAvailable icons:")
    print("=" * 60)

    for icon in icons:
        icon_id = icon.get('id', 'unknown')
        name = icon.get('name', 'Unknown')
        desc = icon.get('description', '')
        output_path = OUTPUT_PATH / f"{icon_id}.png"
        exists = output_path.exists()
        status = "exists" if exists else "missing"
        print(f"\n  {icon_id}: {name} [{status}]")
        print(f"    {desc}")


def main():
    parser = argparse.ArgumentParser(description="Generate Essence Wars app icons")
    parser.add_argument("--all", action="store_true",
                       help="Generate all icons")
    parser.add_argument("--id",
                       help="Generate single icon by ID")
    parser.add_argument("--list", action="store_true",
                       help="List all available icons")
    parser.add_argument("--dry-run", action="store_true",
                       help="Show what would be generated without running")
    parser.add_argument("--schnell", action="store_true",
                       help="Use FLUX Schnell for fast iteration (4 steps instead of 20)")

    args = parser.parse_args()

    # Select settings based on model choice
    flux_settings = FLUX_SCHNELL_SETTINGS if args.schnell else FLUX_DEV_SETTINGS

    # Load prompts
    icons = load_prompts()

    # Handle --list
    if args.list:
        list_icons(icons)
        return

    # Require either --all or --id
    if not args.all and not args.id:
        parser.print_help()
        print("\nError: Must specify --all, --id, or --list")
        sys.exit(1)

    # Ensure output directory exists
    OUTPUT_PATH.mkdir(parents=True, exist_ok=True)

    # Log start time
    start_time = datetime.now()
    model_name = "FLUX Schnell" if args.schnell else "FLUX Dev"
    print(f"Icon Generation Started: {start_time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Model: {model_name} ({flux_settings['steps']} steps)")
    print(f"Output resolution: {flux_settings['width']}x{flux_settings['height']}")

    total_success = 0
    total_fail = 0

    if args.id:
        # Generate single icon
        icon_data = next((icon for icon in icons if icon.get('id') == args.id), None)
        if icon_data:
            print(f"\n{'='*60}")
            print(f"Generating single icon: {args.id}")
            print(f"{'='*60}")
            if generate_icon(icon_data, flux_settings, args.dry_run, args.schnell):
                total_success = 1
            else:
                total_fail = 1
        else:
            print(f"Error: Icon '{args.id}' not found")
            sys.exit(1)
    else:
        # Generate all icons
        print(f"\n{'='*60}")
        print(f"Generating all {len(icons)} icons")
        print(f"{'='*60}")

        for icon_data in icons:
            if generate_icon(icon_data, flux_settings, args.dry_run, args.schnell):
                total_success += 1
            else:
                total_fail += 1

    # Summary
    end_time = datetime.now()
    duration = end_time - start_time

    print(f"\n{'='*60}")
    print("GENERATION COMPLETE")
    print(f"{'='*60}")
    print(f"Success: {total_success}")
    print(f"Failed:  {total_fail}")
    print(f"Duration: {duration}")

    if total_success > 0 and not args.dry_run:
        print(f"\nNext steps:")
        print(f"  1. Review generated icons in: {OUTPUT_PATH}")
        print(f"  2. Install art dependencies: uv sync --group art")
        print(f"  3. Process into icon sizes: uv run python scripts/process-icons.py --input {OUTPUT_PATH}/<icon>.png")


if __name__ == "__main__":
    main()
