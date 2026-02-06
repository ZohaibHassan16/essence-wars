#!/usr/bin/env python3
"""
Icon Processing Script for Essence Wars

Takes a generated icon PNG, removes the background, and creates all required
icon sizes for Tauri (Windows, macOS, Linux).

Requirements:
    uv sync --group art  # Install Pillow, rembg, PyYAML

Usage:
    uv run python scripts/process-icons.py --input ~/.ai-assets/output/icons/essence_crystal.png
    uv run python scripts/process-icons.py --input icon.png --skip-rembg  # If already transparent

This will create:
    - src-tauri/icons/*.png (various sizes)
    - src-tauri/icons/icon.ico (Windows)
    - src-tauri/icons/icon.icns (macOS)
    - static/favicon.png
"""

import argparse
import subprocess
import sys
from pathlib import Path
from PIL import Image

PROJECT_ROOT = Path(__file__).parent.parent
TAURI_ICONS_PATH = PROJECT_ROOT / "crates/essence-wars-ui/src-tauri/icons"
STATIC_PATH = PROJECT_ROOT / "crates/essence-wars-ui/static"

# Required icon sizes for Tauri
ICON_SIZES = {
    # Standard sizes
    "32x32.png": 32,
    "128x128.png": 128,
    "128x128@2x.png": 256,
    "icon.png": 512,
    # Windows Store logos
    "Square30x30Logo.png": 30,
    "Square44x44Logo.png": 44,
    "Square71x71Logo.png": 71,
    "Square89x89Logo.png": 89,
    "Square107x107Logo.png": 107,
    "Square142x142Logo.png": 142,
    "Square150x150Logo.png": 150,
    "Square284x284Logo.png": 284,
    "Square310x310Logo.png": 310,
    "StoreLogo.png": 50,
}

# Favicon size
FAVICON_SIZE = 32


def remove_background(input_path: Path, output_path: Path) -> bool:
    """Remove background using rembg."""
    print(f"  Removing background from {input_path.name}...")
    try:
        # Try using rembg CLI
        result = subprocess.run(
            ["rembg", "i", str(input_path), str(output_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        if result.returncode != 0:
            print(f"    rembg error: {result.stderr[:200]}")
            return False
        print(f"    Created: {output_path}")
        return True
    except FileNotFoundError:
        print("    Error: rembg not found. Install with: pip install rembg")
        return False
    except subprocess.TimeoutExpired:
        print("    Timeout removing background")
        return False
    except Exception as e:
        print(f"    Exception: {e}")
        return False


def resize_icon(source: Image.Image, size: int) -> Image.Image:
    """Resize icon to specified size with high-quality resampling."""
    # Use LANCZOS for high-quality downscaling
    resized = source.resize((size, size), Image.Resampling.LANCZOS)
    return resized


def create_ico(source: Image.Image, output_path: Path) -> bool:
    """Create Windows ICO file with multiple sizes."""
    print(f"  Creating ICO: {output_path.name}")
    try:
        # ICO should contain multiple sizes
        sizes = [16, 24, 32, 48, 64, 128, 256]
        icons = []
        for size in sizes:
            resized = resize_icon(source, size)
            icons.append(resized)

        # Save as ICO with all sizes
        icons[0].save(
            output_path,
            format='ICO',
            sizes=[(s, s) for s in sizes],
            append_images=icons[1:]
        )
        print(f"    Created: {output_path}")
        return True
    except Exception as e:
        print(f"    Error creating ICO: {e}")
        return False


def create_icns(source: Image.Image, output_path: Path) -> bool:
    """Create macOS ICNS file."""
    print(f"  Creating ICNS: {output_path.name}")
    try:
        # ICNS requires specific sizes
        # We'll create a temporary iconset and use iconutil if available
        # Otherwise, fall back to PIL's limited ICNS support

        # Try using iconutil (macOS only)
        import platform
        if platform.system() == "Darwin":
            import tempfile
            import shutil

            with tempfile.TemporaryDirectory() as tmpdir:
                iconset_path = Path(tmpdir) / "icon.iconset"
                iconset_path.mkdir()

                # ICNS required sizes
                icns_sizes = {
                    "icon_16x16.png": 16,
                    "icon_16x16@2x.png": 32,
                    "icon_32x32.png": 32,
                    "icon_32x32@2x.png": 64,
                    "icon_128x128.png": 128,
                    "icon_128x128@2x.png": 256,
                    "icon_256x256.png": 256,
                    "icon_256x256@2x.png": 512,
                    "icon_512x512.png": 512,
                    "icon_512x512@2x.png": 1024,
                }

                for filename, size in icns_sizes.items():
                    resized = resize_icon(source, size)
                    resized.save(iconset_path / filename, "PNG")

                result = subprocess.run(
                    ["iconutil", "-c", "icns", str(iconset_path), "-o", str(output_path)],
                    capture_output=True,
                    text=True
                )
                if result.returncode == 0:
                    print(f"    Created: {output_path}")
                    return True

        # Fallback: create a simple ICNS using PIL (limited support)
        # PIL's ICNS support is limited, so we create a 512x512 version
        resized = resize_icon(source, 512)
        resized.save(output_path, format='ICNS')
        print(f"    Created (basic): {output_path}")
        return True

    except Exception as e:
        print(f"    Error creating ICNS: {e}")
        # Try a simpler approach - just copy the PNG
        try:
            resized = resize_icon(source, 512)
            resized.save(output_path, format='PNG')
            print(f"    Created as PNG (ICNS conversion failed): {output_path}")
            return True
        except Exception as e2:
            print(f"    Fallback also failed: {e2}")
            return False


def process_icon(input_path: Path, skip_rembg: bool = False) -> bool:
    """Process icon: remove background and create all required sizes."""

    if not input_path.exists():
        print(f"Error: Input file not found: {input_path}")
        return False

    print(f"\nProcessing icon: {input_path}")
    print("=" * 60)

    # Step 1: Remove background (or use as-is if already transparent)
    if skip_rembg:
        transparent_path = input_path
        print("  Skipping background removal (--skip-rembg)")
    else:
        transparent_path = input_path.parent / f"{input_path.stem}_transparent.png"
        if not remove_background(input_path, transparent_path):
            print("  Warning: Background removal failed, using original")
            transparent_path = input_path

    # Load the transparent image
    try:
        source = Image.open(transparent_path).convert("RGBA")
        print(f"  Loaded: {source.size[0]}x{source.size[1]} RGBA")
    except Exception as e:
        print(f"Error loading image: {e}")
        return False

    # Ensure output directories exist
    TAURI_ICONS_PATH.mkdir(parents=True, exist_ok=True)
    STATIC_PATH.mkdir(parents=True, exist_ok=True)

    success_count = 0
    fail_count = 0

    # Step 2: Create all PNG sizes
    print("\nCreating PNG icons...")
    for filename, size in ICON_SIZES.items():
        output_path = TAURI_ICONS_PATH / filename
        try:
            resized = resize_icon(source, size)
            resized.save(output_path, "PNG")
            print(f"  Created: {filename} ({size}x{size})")
            success_count += 1
        except Exception as e:
            print(f"  Error creating {filename}: {e}")
            fail_count += 1

    # Step 3: Create favicon
    print("\nCreating favicon...")
    favicon_path = STATIC_PATH / "favicon.png"
    try:
        favicon = resize_icon(source, FAVICON_SIZE)
        favicon.save(favicon_path, "PNG")
        print(f"  Created: favicon.png ({FAVICON_SIZE}x{FAVICON_SIZE})")
        success_count += 1
    except Exception as e:
        print(f"  Error creating favicon: {e}")
        fail_count += 1

    # Step 4: Create ICO (Windows)
    print("\nCreating Windows ICO...")
    ico_path = TAURI_ICONS_PATH / "icon.ico"
    if create_ico(source, ico_path):
        success_count += 1
    else:
        fail_count += 1

    # Step 5: Create ICNS (macOS)
    print("\nCreating macOS ICNS...")
    icns_path = TAURI_ICONS_PATH / "icon.icns"
    if create_icns(source, icns_path):
        success_count += 1
    else:
        fail_count += 1

    # Summary
    print(f"\n{'='*60}")
    print("PROCESSING COMPLETE")
    print(f"{'='*60}")
    print(f"Success: {success_count}")
    print(f"Failed:  {fail_count}")
    print(f"\nIcon files created in:")
    print(f"  {TAURI_ICONS_PATH}")
    print(f"  {STATIC_PATH}/favicon.png")

    return fail_count == 0


def main():
    parser = argparse.ArgumentParser(description="Process Essence Wars app icons")
    parser.add_argument("--input", required=True,
                       help="Input PNG file (512x512 recommended)")
    parser.add_argument("--skip-rembg", action="store_true",
                       help="Skip background removal (if already transparent)")

    args = parser.parse_args()

    input_path = Path(args.input).expanduser().resolve()

    if process_icon(input_path, args.skip_rembg):
        sys.exit(0)
    else:
        sys.exit(1)


if __name__ == "__main__":
    main()
