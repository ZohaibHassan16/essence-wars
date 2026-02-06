#!/usr/bin/env python3
"""
Background Upscaling Script for Essence Wars

Upscales generated background PNGs using Real-ESRGAN and converts to WebP.
Run this after generate-background-art.py to get final high-resolution backgrounds.

Usage:
    uv run python scripts/upscale-backgrounds.py --all
    uv run python scripts/upscale-backgrounds.py --id extraction_dispute
    uv run python scripts/upscale-backgrounds.py --list
"""

import argparse
import subprocess
import sys
from pathlib import Path
from datetime import datetime

# Configuration
REALESRGAN_PATH = Path.home() / ".local/bin/realesrgan-ncnn-vulkan"
MODELS_PATH = Path.home() / ".local/bin/models"
INPUT_PATH = Path.home() / ".ai-assets/output/backgrounds"
UPSCALED_PATH = Path.home() / ".ai-assets/output/backgrounds_upscaled"
PROJECT_ROOT = Path(__file__).parent.parent
FINAL_OUTPUT = PROJECT_ROOT / "crates/essence-wars-ui/static/backgrounds"

# Upscale settings
SCALE = 2  # 2x upscale: 1344x768 -> 2688x1536
MODEL = "realesrgan-x4plus"  # Best quality general model


def check_realesrgan():
    """Check if Real-ESRGAN is available."""
    if not REALESRGAN_PATH.exists():
        print(f"Error: Real-ESRGAN not found at {REALESRGAN_PATH}")
        print("Install with:")
        print("  mkdir -p ~/.local/bin && cd ~/.local/bin")
        print("  wget https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.5.0/realesrgan-ncnn-vulkan-20220424-ubuntu.zip")
        print("  unzip realesrgan-ncnn-vulkan-20220424-ubuntu.zip")
        sys.exit(1)


def get_available_backgrounds() -> list:
    """Get list of generated background PNGs ready for upscaling."""
    if not INPUT_PATH.exists():
        return []
    return sorted(INPUT_PATH.glob("*.png"))


def upscale_image(input_png: Path, dry_run: bool = False) -> bool:
    """Upscale a single background image."""
    bg_id = input_png.stem
    upscaled_png = UPSCALED_PATH / f"{bg_id}_upscaled.png"
    final_webp = FINAL_OUTPUT / f"{bg_id}.webp"

    print(f"  Processing: {bg_id}")
    print(f"    Input: {input_png}")
    print(f"    Upscaled: {upscaled_png}")
    print(f"    Final: {final_webp}")

    if dry_run:
        print(f"    [DRY RUN] Would upscale {SCALE}x with {MODEL}")
        return True

    # Ensure output directories exist
    UPSCALED_PATH.mkdir(parents=True, exist_ok=True)
    FINAL_OUTPUT.mkdir(parents=True, exist_ok=True)

    # Run Real-ESRGAN
    cmd = [
        str(REALESRGAN_PATH),
        "-i", str(input_png),
        "-o", str(upscaled_png),
        "-s", str(SCALE),
        "-n", MODEL,
        "-m", str(MODELS_PATH),
        "-f", "png",
    ]

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
        if result.returncode != 0:
            print(f"    Error upscaling: {result.stderr[:200]}")
            return False
    except subprocess.TimeoutExpired:
        print(f"    Timeout upscaling {bg_id}")
        return False
    except Exception as e:
        print(f"    Exception: {e}")
        return False

    # Convert to WebP
    if upscaled_png.exists():
        webp_cmd = ["cwebp", "-q", "95", str(upscaled_png), "-o", str(final_webp)]
        try:
            subprocess.run(webp_cmd, capture_output=True, check=True)
            print(f"    Created: {final_webp}")

            # Get final dimensions
            from PIL import Image
            with Image.open(final_webp) as img:
                print(f"    Final size: {img.width}x{img.height}")

            return True
        except subprocess.CalledProcessError:
            print(f"    Error converting to WebP")
            return False
        except ImportError:
            # PIL not available, just report success without dimensions
            print(f"    Created: {final_webp}")
            return True
        except FileNotFoundError:
            print("    Error: cwebp not found. Install with: sudo apt install webp")
            return False
    else:
        print(f"    Upscaled PNG not created")
        return False


def list_backgrounds():
    """List available backgrounds for upscaling."""
    backgrounds = get_available_backgrounds()

    print("\nBackgrounds ready for upscaling:")
    print("=" * 60)

    if not backgrounds:
        print("  No PNG files found in", INPUT_PATH)
        print("  Run generate-background-art.py first!")
        return

    for bg in backgrounds:
        final_webp = FINAL_OUTPUT / f"{bg.stem}.webp"
        status = "upscaled" if final_webp.exists() else "pending"
        print(f"  {bg.stem} [{status}]")

    print(f"\nTotal: {len(backgrounds)} backgrounds")


def main():
    parser = argparse.ArgumentParser(description="Upscale Essence Wars backgrounds with Real-ESRGAN")
    parser.add_argument("--all", action="store_true",
                       help="Upscale all pending backgrounds")
    parser.add_argument("--id",
                       help="Upscale single background by ID")
    parser.add_argument("--list", action="store_true",
                       help="List available backgrounds")
    parser.add_argument("--dry-run", action="store_true",
                       help="Show what would be done without running")

    args = parser.parse_args()

    # Check Real-ESRGAN installation
    check_realesrgan()

    # Handle --list
    if args.list:
        list_backgrounds()
        return

    # Require --all or --id
    if not args.all and not args.id:
        parser.print_help()
        print("\nError: Must specify --all, --id, or --list")
        sys.exit(1)

    backgrounds = get_available_backgrounds()
    if not backgrounds:
        print("No backgrounds to upscale. Run generate-background-art.py first!")
        sys.exit(1)

    # Log start
    start_time = datetime.now()
    print(f"Background Upscaling Started: {start_time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Scale: {SCALE}x | Model: {MODEL}")

    total_success = 0
    total_fail = 0

    if args.id:
        # Single background
        input_png = INPUT_PATH / f"{args.id}.png"
        if not input_png.exists():
            print(f"Error: {input_png} not found")
            sys.exit(1)

        print(f"\n{'='*60}")
        print(f"Upscaling single background: {args.id}")
        print(f"{'='*60}")

        if upscale_image(input_png, args.dry_run):
            total_success = 1
        else:
            total_fail = 1
    else:
        # All backgrounds
        print(f"\n{'='*60}")
        print(f"Upscaling {len(backgrounds)} backgrounds")
        print(f"{'='*60}")

        for bg in backgrounds:
            if upscale_image(bg, args.dry_run):
                total_success += 1
            else:
                total_fail += 1

    # Summary
    end_time = datetime.now()
    duration = end_time - start_time

    print(f"\n{'='*60}")
    print("UPSCALING COMPLETE")
    print(f"{'='*60}")
    print(f"Success: {total_success}")
    print(f"Failed:  {total_fail}")
    print(f"Duration: {duration}")


if __name__ == "__main__":
    main()
