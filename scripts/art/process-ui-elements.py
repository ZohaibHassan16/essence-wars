#!/usr/bin/env python3
"""
UI Element Processing Script for Essence Wars

Processes generated UI elements: removes backgrounds and organizes into
the static/ui folder for use in the application.

Requirements:
    uv sync --group art  # Install Pillow, rembg

Usage:
    uv run python scripts/process-ui-elements.py --all
    uv run python scripts/process-ui-elements.py --id emblem_argentum
    uv run python scripts/process-ui-elements.py --category faction_emblems
    uv run python scripts/process-ui-elements.py --list
"""

import argparse
import sys
from pathlib import Path
from PIL import Image

# Try to import rembg
try:
    from rembg import remove as rembg_remove
    REMBG_AVAILABLE = True
except ImportError:
    REMBG_AVAILABLE = False
    print("Warning: rembg not available. Run: uv sync --group art")

PROJECT_ROOT = Path(__file__).parent.parent.parent  # scripts/art/ -> scripts/ -> project root
INPUT_PATH = Path.home() / ".ai-assets/output/ui_elements"
OUTPUT_PATH = PROJECT_ROOT / "crates/essence-wars-ui/static/ui/decorations"

# Category to subfolder mapping (simplified - just emblems and icons, CSS for everything else)
CATEGORY_FOLDERS = {
    "emblem_": "emblems",
    "essence_": "icons",
}


def get_category_folder(element_id: str) -> str:
    """Determine output subfolder based on element ID prefix."""
    for prefix, folder in CATEGORY_FOLDERS.items():
        if element_id.startswith(prefix):
            return folder
    return "misc"


def remove_background(input_path: Path) -> Image.Image | None:
    """Remove background from image using rembg."""
    if not REMBG_AVAILABLE:
        # Fallback: just load the image as-is
        return Image.open(input_path).convert("RGBA")

    try:
        with open(input_path, 'rb') as f:
            input_data = f.read()

        output_data = rembg_remove(input_data)

        from io import BytesIO
        return Image.open(BytesIO(output_data)).convert("RGBA")
    except Exception as e:
        print(f"    Error removing background: {e}")
        return None


def process_element(input_path: Path, skip_rembg: bool = False) -> bool:
    """Process a single UI element."""
    element_id = input_path.stem
    category_folder = get_category_folder(element_id)

    output_dir = OUTPUT_PATH / category_folder
    output_dir.mkdir(parents=True, exist_ok=True)

    output_path = output_dir / f"{element_id}.png"

    print(f"  Processing: {element_id}")
    print(f"    Input: {input_path}")
    print(f"    Output: {output_path}")

    if skip_rembg:
        # Just copy/convert to RGBA
        try:
            img = Image.open(input_path).convert("RGBA")
        except Exception as e:
            print(f"    Error loading image: {e}")
            return False
    else:
        # Remove background
        img = remove_background(input_path)
        if img is None:
            return False

    # Save processed image
    try:
        img.save(output_path, "PNG")
        print(f"    Created: {output_path}")
        return True
    except Exception as e:
        print(f"    Error saving: {e}")
        return False


def list_elements():
    """List available elements for processing."""
    print("\nGenerated UI Elements (ready for processing):")
    print("=" * 60)

    if not INPUT_PATH.exists():
        print(f"  No elements found in {INPUT_PATH}")
        print("  Run generate-ui-elements.py first!")
        return

    png_files = sorted(INPUT_PATH.glob("*.png"))

    if not png_files:
        print(f"  No PNG files found in {INPUT_PATH}")
        return

    # Group by category
    categories = {}
    for png in png_files:
        category = get_category_folder(png.stem)
        if category not in categories:
            categories[category] = []
        categories[category].append(png)

    for category, files in sorted(categories.items()):
        print(f"\n{category}/ ({len(files)} files):")
        for f in files:
            output_path = OUTPUT_PATH / category / f"{f.stem}.png"
            status = "processed" if output_path.exists() else "pending"
            print(f"  - {f.stem} [{status}]")


def main():
    parser = argparse.ArgumentParser(description="Process Essence Wars UI elements")
    parser.add_argument("--all", action="store_true",
                       help="Process all generated elements")
    parser.add_argument("--id",
                       help="Process single element by ID")
    parser.add_argument("--category",
                       choices=['emblems', 'icons'],
                       help="Process all elements in a category")
    parser.add_argument("--list", action="store_true",
                       help="List available elements")
    parser.add_argument("--skip-rembg", action="store_true",
                       help="Skip background removal")

    args = parser.parse_args()

    # Handle --list
    if args.list:
        list_elements()
        return

    # Require action
    if not args.all and not args.id and not args.category:
        parser.print_help()
        print("\nError: Must specify --all, --id, --category, or --list")
        sys.exit(1)

    # Ensure output directory exists
    OUTPUT_PATH.mkdir(parents=True, exist_ok=True)

    print(f"UI Element Processing")
    print("=" * 60)

    total_success = 0
    total_fail = 0

    if args.id:
        # Single element
        input_path = INPUT_PATH / f"{args.id}.png"
        if not input_path.exists():
            print(f"Error: {input_path} not found")
            sys.exit(1)

        if process_element(input_path, args.skip_rembg):
            total_success = 1
        else:
            total_fail = 1

    elif args.category:
        # Find elements matching category
        prefix_map = {v: k for k, v in CATEGORY_FOLDERS.items()}
        prefix = prefix_map.get(args.category, "")

        png_files = [f for f in INPUT_PATH.glob("*.png") if f.stem.startswith(prefix.rstrip('_'))]

        if not png_files:
            print(f"No elements found for category: {args.category}")
            sys.exit(1)

        print(f"Processing {len(png_files)} elements in {args.category}")

        for input_path in sorted(png_files):
            if process_element(input_path, args.skip_rembg):
                total_success += 1
            else:
                total_fail += 1

    else:
        # All elements
        png_files = sorted(INPUT_PATH.glob("*.png"))

        if not png_files:
            print(f"No elements found in {INPUT_PATH}")
            print("Run generate-ui-elements.py first!")
            sys.exit(1)

        print(f"Processing {len(png_files)} elements")

        for input_path in png_files:
            if process_element(input_path, args.skip_rembg):
                total_success += 1
            else:
                total_fail += 1

    # Summary
    print(f"\n{'='*60}")
    print("PROCESSING COMPLETE")
    print(f"{'='*60}")
    print(f"Success: {total_success}")
    print(f"Failed:  {total_fail}")
    print(f"\nOutput: {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
