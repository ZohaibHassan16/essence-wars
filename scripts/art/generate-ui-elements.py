#!/usr/bin/env python3
"""
UI Element Art Generation Script for Essence Wars

Generates decorative UI elements (emblems, corners, dividers, frames) using FLUX.
Each category has specific sizes optimized for their use case.

Usage:
    uv run python scripts/generate-ui-elements.py --all
    uv run python scripts/generate-ui-elements.py --category faction_emblems
    uv run python scripts/generate-ui-elements.py --id emblem_argentum
    uv run python scripts/generate-ui-elements.py --list
    uv run python scripts/generate-ui-elements.py --all --schnell

Categories:
    faction_emblems  - 256x256 faction symbols
    corner_ornaments - 128x128 corner decorations
    dividers         - 512x64 horizontal separators
    button_frames    - 320x80 button decorations
    panel_frames     - 512x512 panel borders
    essence_icons    - 64x64 small UI icons
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
OUTPUT_PATH = Path.home() / ".ai-assets/output/ui_elements"
PROJECT_ROOT = Path(__file__).parent.parent.parent  # scripts/art/ -> scripts/ -> project root
PROMPTS_FILE = PROJECT_ROOT / "data/art/prompts/ui_elements.yaml"

# Size mappings for each category
SIZE_CONFIGS = {
    "256x256": {"width": "256", "height": "256"},
    "128x128": {"width": "128", "height": "128"},
    "512x64": {"width": "512", "height": "64"},
    "320x80": {"width": "320", "height": "80"},
    "512x512": {"width": "512", "height": "512"},
    "64x64": {"width": "64", "height": "64"},
}

# Base FLUX settings
FLUX_DEV_BASE = {
    "model": MODELS_PATH / "flux-dev-q8.gguf",
    "vae": MODELS_PATH / "ae.safetensors",
    "clip_l": MODELS_PATH / "clip_l.safetensors",
    "t5xxl": MODELS_PATH / "t5-Q5_K_M.gguf",
    "cfg_scale": "1.0",
    "sampling_method": "euler",
    "steps": "20",
}

FLUX_SCHNELL_BASE = {
    "model": MODELS_PATH / "flux-schnell-q4.gguf",
    "vae": MODELS_PATH / "ae.safetensors",
    "clip_l": MODELS_PATH / "clip_l.safetensors",
    "t5xxl": MODELS_PATH / "t5-Q5_K_M.gguf",
    "cfg_scale": "1.0",
    "sampling_method": "euler",
    "steps": "4",
}


def load_prompts() -> dict:
    """Load UI element prompts from YAML file."""
    if not PROMPTS_FILE.exists():
        print(f"Error: Prompt file not found: {PROMPTS_FILE}")
        sys.exit(1)

    with open(PROMPTS_FILE, 'r') as f:
        data = yaml.safe_load(f)

    return data


def get_flux_settings(size: str, use_schnell: bool) -> dict:
    """Get FLUX settings for a specific size."""
    base = FLUX_SCHNELL_BASE if use_schnell else FLUX_DEV_BASE
    size_config = SIZE_CONFIGS.get(size, SIZE_CONFIGS["256x256"])

    settings = dict(base)
    settings["width"] = size_config["width"]
    settings["height"] = size_config["height"]
    return settings


def generate_element(element: dict, use_schnell: bool, dry_run: bool = False) -> bool:
    """Generate a single UI element using FLUX."""
    element_id = element.get('id', 'unknown')
    name = element.get('name', 'Unknown')
    prompt = element.get('prompt', '').strip()
    negative = element.get('negative', '')
    size = element.get('size', '256x256')

    if not prompt:
        print(f"  Skipping {element_id}: No prompt defined")
        return False

    flux_settings = get_flux_settings(size, use_schnell)

    # Output path (organized by category inferred from ID prefix)
    png_path = OUTPUT_PATH / f"{element_id}.png"

    model_name = "FLUX Schnell" if use_schnell else "FLUX Dev"
    print(f"  Generating {element_id}: {name}")
    print(f"    Model: {model_name} ({flux_settings['steps']} steps)")
    print(f"    Size: {size}")
    print(f"    Output: {png_path}")

    if dry_run:
        print(f"    [DRY RUN] Would generate: {png_path}")
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
            print(f"    Error: {result.stderr[:200]}")
            return False
    except subprocess.TimeoutExpired:
        print(f"    Timeout generating {element_id}")
        return False
    except FileNotFoundError:
        print(f"    Error: sd-cli not found")
        return False
    except Exception as e:
        print(f"    Exception: {e}")
        return False

    if png_path.exists():
        print(f"    Created: {png_path}")
        return True
    else:
        print(f"    PNG not created")
        return False


def list_elements(data: dict):
    """List all available UI elements by category."""
    print("\nAvailable UI Elements:")
    print("=" * 70)

    # Simplified: just emblems and icons (CSS for everything else)
    categories = ['faction_emblems', 'essence_icons']

    for category in categories:
        elements = data.get(category, [])
        if elements:
            print(f"\n{category} ({len(elements)} elements):")
            for elem in elements:
                elem_id = elem.get('id', 'unknown')
                name = elem.get('name', 'Unknown')
                size = elem.get('size', '?')
                output_path = OUTPUT_PATH / f"{elem_id}.png"
                status = "exists" if output_path.exists() else "missing"
                print(f"  - {elem_id}: {name} [{size}] [{status}]")


def get_all_elements(data: dict) -> list:
    """Get all elements from all categories."""
    # Simplified: just emblems and icons (CSS for everything else)
    categories = ['faction_emblems', 'essence_icons']

    all_elements = []
    for category in categories:
        all_elements.extend(data.get(category, []))

    return all_elements


def get_category_elements(data: dict, category: str) -> list:
    """Get all elements from a specific category."""
    return data.get(category, [])


def find_element_by_id(data: dict, element_id: str) -> dict | None:
    """Find a specific element by ID."""
    for elem in get_all_elements(data):
        if elem.get('id') == element_id:
            return elem
    return None


def main():
    parser = argparse.ArgumentParser(description="Generate Essence Wars UI elements")
    parser.add_argument("--all", action="store_true",
                       help="Generate all UI elements")
    parser.add_argument("--category",
                       choices=['faction_emblems', 'essence_icons'],
                       help="Generate all elements in a category")
    parser.add_argument("--id",
                       help="Generate single element by ID")
    parser.add_argument("--list", action="store_true",
                       help="List all available elements")
    parser.add_argument("--dry-run", action="store_true",
                       help="Show what would be generated")
    parser.add_argument("--schnell", action="store_true",
                       help="Use FLUX Schnell for fast iteration")

    args = parser.parse_args()

    # Load prompts
    data = load_prompts()

    # Handle --list
    if args.list:
        list_elements(data)
        return

    # Require action
    if not args.all and not args.category and not args.id:
        parser.print_help()
        print("\nError: Must specify --all, --category, --id, or --list")
        sys.exit(1)

    # Ensure output directory exists
    OUTPUT_PATH.mkdir(parents=True, exist_ok=True)

    # Log start
    start_time = datetime.now()
    model_name = "FLUX Schnell" if args.schnell else "FLUX Dev"
    print(f"UI Element Generation Started: {start_time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Model: {model_name}")

    total_success = 0
    total_fail = 0

    if args.id:
        # Single element
        element = find_element_by_id(data, args.id)
        if element:
            print(f"\n{'='*60}")
            print(f"Generating: {args.id}")
            print(f"{'='*60}")
            if generate_element(element, args.schnell, args.dry_run):
                total_success = 1
            else:
                total_fail = 1
        else:
            print(f"Error: Element '{args.id}' not found")
            sys.exit(1)

    elif args.category:
        # Category
        elements = get_category_elements(data, args.category)
        print(f"\n{'='*60}")
        print(f"Generating category: {args.category} ({len(elements)} elements)")
        print(f"{'='*60}")

        for element in elements:
            if generate_element(element, args.schnell, args.dry_run):
                total_success += 1
            else:
                total_fail += 1

    else:
        # All elements
        elements = get_all_elements(data)
        print(f"\n{'='*60}")
        print(f"Generating all {len(elements)} UI elements")
        print(f"{'='*60}")

        for element in elements:
            if generate_element(element, args.schnell, args.dry_run):
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
        print(f"\nOutput: {OUTPUT_PATH}")
        print(f"\nNext: Remove backgrounds with:")
        print(f"  uv run python scripts/process-ui-elements.py --all")


if __name__ == "__main__":
    main()
