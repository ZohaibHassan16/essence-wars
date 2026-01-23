#!/usr/bin/env python3
"""
Convert generated PNG assets to WebP for distribution.
Handles card art, frames, backgrounds, and card backs.
"""

import subprocess
import sys
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
import argparse

# Paths
DRAFTS_DIR = Path.home() / ".ai-assets/output/essence-wars/drafts"
OUTPUT_BASE = Path.home() / ".ai-assets/output/essence-wars"
STATIC_DIR = Path("/home/chris/ai-cardgame/crates/essence-wars-ui/static")

# WebP quality (0-100, 80-90 is good balance of quality/size)
WEBP_QUALITY = 85


def convert_to_webp(src: Path, dst: Path, quality: int = WEBP_QUALITY) -> tuple[bool, str]:
    """Convert a single PNG to WebP."""
    dst.parent.mkdir(parents=True, exist_ok=True)

    if dst.exists():
        return True, f"[SKIP] {dst.name} exists"

    cmd = [
        "cwebp",
        "-q", str(quality),
        "-m", "6",  # Compression method (0-6, 6 is slowest but smallest)
        str(src),
        "-o", str(dst)
    ]

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        if result.returncode == 0:
            src_size = src.stat().st_size / 1024
            dst_size = dst.stat().st_size / 1024
            ratio = (1 - dst_size / src_size) * 100
            return True, f"[OK] {dst.name} ({src_size:.0f}KB → {dst_size:.0f}KB, -{ratio:.0f}%)"
        else:
            return False, f"[ERR] {src.name}: {result.stderr[:100]}"
    except Exception as e:
        return False, f"[ERR] {src.name}: {e}"


def convert_card_art(factions: list[str] | None = None, parallel: int = 4):
    """Convert card art PNGs to WebP."""
    target_factions = factions or ["argentum", "symbiote", "obsidion", "neutral"]
    output_dir = STATIC_DIR / "cards" / "core_set"
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"\n=== Converting Card Art to WebP ===\n")

    all_pngs = []
    for faction in target_factions:
        faction_dir = DRAFTS_DIR / faction
        if not faction_dir.exists():
            print(f"  [WARN] No drafts found for {faction}")
            continue

        pngs = list(faction_dir.glob("*.png"))
        all_pngs.extend(pngs)
        print(f"  Found {len(pngs)} PNGs in {faction}/")

    if not all_pngs:
        print("  No PNG files found to convert!")
        return

    print(f"\n  Converting {len(all_pngs)} files with {parallel} threads...\n")

    success = 0
    with ThreadPoolExecutor(max_workers=parallel) as executor:
        futures = {}
        for png in all_pngs:
            # Extract card ID from filename (e.g., "1000_brass_sentinel.png" -> "1000")
            card_id = png.stem.split("_")[0]
            webp_path = output_dir / f"{card_id}.webp"
            futures[executor.submit(convert_to_webp, png, webp_path)] = png

        for future in as_completed(futures):
            ok, msg = future.result()
            print(f"  {msg}")
            if ok and not msg.startswith("[SKIP]"):
                success += 1

    print(f"\n=== Card Art: {success} converted ===\n")


def convert_other_assets():
    """Convert frames, backgrounds, and card backs."""
    assets = [
        ("frames", OUTPUT_BASE / "frames", STATIC_DIR / "frames"),
        ("backgrounds", OUTPUT_BASE / "backgrounds", STATIC_DIR / "backgrounds"),
        ("card_backs", OUTPUT_BASE / "card_backs", STATIC_DIR / "backs"),
    ]

    for name, src_dir, dst_dir in assets:
        if not src_dir.exists():
            print(f"  [WARN] {name}: source directory not found ({src_dir})")
            continue

        pngs = list(src_dir.glob("*.png"))
        if not pngs:
            print(f"  [WARN] {name}: no PNG files found")
            continue

        print(f"\n=== Converting {name} ({len(pngs)} files) ===\n")
        dst_dir.mkdir(parents=True, exist_ok=True)

        for png in pngs:
            webp_path = dst_dir / f"{png.stem}.webp"
            ok, msg = convert_to_webp(png, webp_path)
            print(f"  {msg}")


def show_stats():
    """Show asset directory statistics."""
    print("\n=== Asset Statistics ===\n")

    dirs = [
        ("Card Art", STATIC_DIR / "cards" / "core_set"),
        ("Frames", STATIC_DIR / "frames"),
        ("Backgrounds", STATIC_DIR / "backgrounds"),
        ("Card Backs", STATIC_DIR / "backs"),
    ]

    total_size = 0
    total_files = 0

    for name, path in dirs:
        if path.exists():
            files = list(path.glob("*.webp"))
            size = sum(f.stat().st_size for f in files) / (1024 * 1024)
            total_size += size
            total_files += len(files)
            print(f"  {name}: {len(files)} files, {size:.1f} MB")
        else:
            print(f"  {name}: (not found)")

    print(f"\n  TOTAL: {total_files} files, {total_size:.1f} MB")


def main():
    parser = argparse.ArgumentParser(description="Convert Essence Wars assets to WebP")
    parser.add_argument("--cards", action="store_true", help="Convert card art only")
    parser.add_argument("--other", action="store_true", help="Convert frames/backgrounds/backs only")
    parser.add_argument("--stats", action="store_true", help="Show asset statistics")
    parser.add_argument("--faction", type=str, help="Convert specific faction (comma-separated)")
    parser.add_argument("--parallel", "-j", type=int, default=4, help="Parallel conversion threads")
    parser.add_argument("--quality", "-q", type=int, default=85, help="WebP quality (0-100)")
    args = parser.parse_args()

    global WEBP_QUALITY
    WEBP_QUALITY = args.quality

    if args.stats:
        show_stats()
        return

    factions = args.faction.split(",") if args.faction else None

    # Default: convert everything
    if not args.cards and not args.other:
        args.cards = True
        args.other = True

    if args.cards:
        convert_card_art(factions, args.parallel)

    if args.other:
        convert_other_assets()

    show_stats()


if __name__ == "__main__":
    main()
