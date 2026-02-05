#!/usr/bin/env python3
"""
Background Art Generation Script for Essence Wars

Reads prompts from backgrounds.yaml and generates background art using FLUX.
Outputs at 1344x768 (16:9) for later upscaling with Real-ESRGAN to ~2688x1536.

Usage:
    python scripts/generate-background-art.py --all
    python scripts/generate-background-art.py --id extraction_dispute
    python scripts/generate-background-art.py --list
    python scripts/generate-background-art.py --dry-run --all
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
OUTPUT_PATH = Path.home() / ".ai-assets/output/backgrounds"
PROJECT_ROOT = Path(__file__).parent.parent
PROMPTS_FILE = PROJECT_ROOT / "data/art/prompts/backgrounds.yaml"
BACKGROUNDS_OUTPUT = PROJECT_ROOT / "crates/essence-wars-ui/static/backgrounds"

# FLUX generation settings for backgrounds (16:9 landscape)
FLUX_DEV_SETTINGS = {
    "model": MODELS_PATH / "flux-dev-q8.gguf",
    "vae": MODELS_PATH / "ae.safetensors",
    "clip_l": MODELS_PATH / "clip_l.safetensors",
    "t5xxl": MODELS_PATH / "t5-Q5_K_M.gguf",
    "cfg_scale": "1.0",
    "sampling_method": "euler",
    "steps": "20",
    # 16:9 aspect ratio, FLUX-friendly dimensions
    "height": "768",
    "width": "1344",
}

# FLUX Schnell settings for fast iteration (4 steps, no guidance needed)
FLUX_SCHNELL_SETTINGS = {
    "model": MODELS_PATH / "flux-schnell-q4.gguf",
    "vae": MODELS_PATH / "ae.safetensors",
    "clip_l": MODELS_PATH / "clip_l.safetensors",
    "t5xxl": MODELS_PATH / "t5-Q5_K_M.gguf",
    "cfg_scale": "1.0",
    "sampling_method": "euler",
    "steps": "4",
    # 16:9 aspect ratio, FLUX-friendly dimensions
    "height": "768",
    "width": "1344",
}


def load_prompts() -> list:
    """Load background prompts from YAML file."""
    if not PROMPTS_FILE.exists():
        print(f"Error: Prompt file not found: {PROMPTS_FILE}")
        sys.exit(1)

    with open(PROMPTS_FILE, 'r') as f:
        data = yaml.safe_load(f)

    return data.get('backgrounds', [])


def parse_lora(lora_string: str) -> str:
    """Convert lora string format to sd-cli format."""
    if not lora_string:
        return ""

    parts = []
    for lora in lora_string.split(','):
        lora = lora.strip()
        if ':' in lora:
            name, weight = lora.split(':')
            parts.append(f"<lora:{name.strip()}:{weight.strip()}>")

    return ' '.join(parts)


def generate_image(bg_data: dict, flux_settings: dict, dry_run: bool = False, use_schnell: bool = False) -> bool:
    """Generate a single background image using FLUX."""
    bg_id = bg_data.get('id', 'unknown')
    name = bg_data.get('name', 'Unknown')
    prompt = bg_data.get('prompt', '').strip()
    lora = bg_data.get('lora', '')
    negative = bg_data.get('negative', '')

    if not prompt:
        print(f"  Skipping {bg_id} ({name}): No prompt defined")
        return False

    # Build full prompt with LoRA
    lora_tags = parse_lora(lora)
    full_prompt = f"{prompt} {lora_tags}".strip()

    # Output paths
    png_path = OUTPUT_PATH / f"{bg_id}.png"
    webp_path = BACKGROUNDS_OUTPUT / f"{bg_id}.webp"

    model_name = "FLUX Schnell" if use_schnell else "FLUX Dev"
    print(f"  Generating {bg_id}: {name}")
    print(f"    Model: {model_name} ({flux_settings['steps']} steps)")
    print(f"    Output: {webp_path}")
    print(f"    Resolution: {flux_settings['width']}x{flux_settings['height']} (will upscale later)")

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
        "--diffusion-model", str(flux_settings["model"]),
        "--vae", str(flux_settings["vae"]),
        "--clip_l", str(flux_settings["clip_l"]),
        "--t5xxl", str(flux_settings["t5xxl"]),
        "--lora-model-dir", str(LORAS_PATH),
        "-p", full_prompt,
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
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=600)
        if result.returncode != 0:
            print(f"    Error generating {bg_id}: {result.stderr[:200]}")
            return False
    except subprocess.TimeoutExpired:
        print(f"    Timeout generating {bg_id}")
        return False
    except FileNotFoundError:
        print(f"    Error: sd-cli not found at {SD_CPP_PATH / 'build/bin/sd-cli'}")
        print("    Make sure stable-diffusion.cpp is built.")
        return False
    except Exception as e:
        print(f"    Exception generating {bg_id}: {e}")
        return False

    # Convert to WebP (keeping PNG for upscaling later)
    if png_path.exists():
        webp_cmd = ["cwebp", "-q", "95", str(png_path), "-o", str(webp_path)]
        try:
            subprocess.run(webp_cmd, capture_output=True, check=True)
            print(f"    Created: {webp_path}")
            print(f"    Note: PNG kept at {png_path} for upscaling")
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


def list_backgrounds(backgrounds: list):
    """List all available backgrounds."""
    print("\nAvailable backgrounds:")
    print("=" * 70)

    for bg in backgrounds:
        bg_id = bg.get('id', 'unknown')
        name = bg.get('name', 'Unknown')
        desc = bg.get('description', '')
        output_path = BACKGROUNDS_OUTPUT / f"{bg_id}.webp"
        exists = output_path.exists()
        status = "exists" if exists else "missing"
        print(f"\n  {bg_id}: {name} [{status}]")
        print(f"    {desc}")


def main():
    parser = argparse.ArgumentParser(description="Generate Essence Wars background art")
    parser.add_argument("--all", action="store_true",
                       help="Generate all backgrounds")
    parser.add_argument("--id",
                       help="Generate single background by ID")
    parser.add_argument("--list", action="store_true",
                       help="List all available backgrounds")
    parser.add_argument("--dry-run", action="store_true",
                       help="Show what would be generated without running")
    parser.add_argument("--schnell", action="store_true",
                       help="Use FLUX Schnell for fast iteration (4 steps instead of 20)")

    args = parser.parse_args()

    # Select settings based on model choice
    flux_settings = FLUX_SCHNELL_SETTINGS if args.schnell else FLUX_DEV_SETTINGS

    # Load prompts
    backgrounds = load_prompts()

    # Handle --list
    if args.list:
        list_backgrounds(backgrounds)
        return

    # Require either --all or --id
    if not args.all and not args.id:
        parser.print_help()
        print("\nError: Must specify --all, --id, or --list")
        sys.exit(1)

    # Ensure output directories exist
    OUTPUT_PATH.mkdir(parents=True, exist_ok=True)
    BACKGROUNDS_OUTPUT.mkdir(parents=True, exist_ok=True)

    # Log start time
    start_time = datetime.now()
    model_name = "FLUX Schnell" if args.schnell else "FLUX Dev"
    print(f"Background Art Generation Started: {start_time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Model: {model_name} ({flux_settings['steps']} steps)")
    print(f"Output resolution: {flux_settings['width']}x{flux_settings['height']} (16:9)")
    print(f"Target after upscale: ~2688x1536")

    total_success = 0
    total_fail = 0

    if args.id:
        # Generate single background
        bg_data = next((bg for bg in backgrounds if bg.get('id') == args.id), None)
        if bg_data:
            print(f"\n{'='*60}")
            print(f"Generating single background: {args.id}")
            print(f"{'='*60}")
            if generate_image(bg_data, flux_settings, args.dry_run, args.schnell):
                total_success = 1
            else:
                total_fail = 1
        else:
            print(f"Error: Background '{args.id}' not found")
            sys.exit(1)
    else:
        # Generate all backgrounds
        print(f"\n{'='*60}")
        print(f"Generating all {len(backgrounds)} backgrounds")
        print(f"{'='*60}")

        for bg_data in backgrounds:
            if generate_image(bg_data, flux_settings, args.dry_run, args.schnell):
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
    print(f"Finished: {end_time.strftime('%Y-%m-%d %H:%M:%S')}")

    if total_success > 0 and not args.dry_run:
        print(f"\nNext step: Upscale PNGs with Real-ESRGAN:")
        print(f"  realesrgan-ncnn-vulkan -i {OUTPUT_PATH}/<name>.png -o <output>.png -s 2")


if __name__ == "__main__":
    main()
