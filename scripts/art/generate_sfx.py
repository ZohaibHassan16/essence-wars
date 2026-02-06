#!/usr/bin/env python3
"""
Essence Wars - Battle SFX Generator using ElevenLabs API

Generates game sound effects from text prompts using ElevenLabs Sound Effects API.
Requires: ELEVENLABS_API_KEY environment variable

Usage:
    python scripts/art/generate_sfx.py                    # Generate all sounds
    python scripts/art/generate_sfx.py --category attack  # Generate specific category
    python scripts/art/generate_sfx.py --faction argentum # Generate specific faction
    python scripts/art/generate_sfx.py --dry-run          # Show what would be generated
"""

import argparse
import os
import sys
import time
from pathlib import Path

import yaml

# ElevenLabs imports
try:
    from elevenlabs.client import ElevenLabs
except ImportError:
    print("ERROR: elevenlabs not installed. Run:")
    print("  uv pip install -e '.[artgen]'")
    sys.exit(1)


# =============================================================================
# Configuration
# =============================================================================

# scripts/art/generate_sfx.py -> parent.parent.parent = project root
PROJECT_ROOT = Path(__file__).parent.parent.parent
PROMPTS_FILE = PROJECT_ROOT / "data" / "art" / "prompts" / "sfx_prompts.yml"
OUTPUT_DIR = PROJECT_ROOT / "crates" / "essence-wars-ui" / "static" / "sounds" / "battle"

# ElevenLabs settings
DURATION_SECONDS = 2.0  # Duration of generated audio (0.5 to 30)
PROMPT_INFLUENCE = 0.5  # How closely to follow prompt (0 to 1)


# =============================================================================
# ElevenLabs SFX Generator
# =============================================================================

class SFXGenerator:
    def __init__(self, api_key: str | None = None):
        self.api_key = api_key or os.environ.get("ELEVENLABS_API_KEY")
        if not self.api_key:
            print("ERROR: ELEVENLABS_API_KEY not set.")
            print("  Set it with: export ELEVENLABS_API_KEY='your-key-here'")
            print("  Get a key at: https://elevenlabs.io/")
            sys.exit(1)

        self.client = ElevenLabs(api_key=self.api_key)

    def generate(self, prompt: str, duration: float = DURATION_SECONDS) -> bytes:
        """Generate audio from a text prompt.

        Returns:
            Audio data as bytes (MP3 format)
        """
        # Add game SFX context to prompt
        enhanced_prompt = f"game sound effect, {prompt}, high quality, clear, no music"

        result = self.client.text_to_sound_effects.convert(
            text=enhanced_prompt,
            duration_seconds=duration,
            prompt_influence=PROMPT_INFLUENCE,
        )

        # Collect all audio chunks
        audio_data = b"".join(chunk for chunk in result)
        return audio_data

    def save_audio(self, audio_data: bytes, output_path: Path):
        """Save audio to file."""
        # Ensure output directory exists
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # ElevenLabs returns MP3, save directly
        mp3_path = output_path.with_suffix(".mp3")
        with open(mp3_path, "wb") as f:
            f.write(audio_data)

        # Convert to OGG for web compatibility (optional)
        try:
            from pydub import AudioSegment

            sound = AudioSegment.from_mp3(mp3_path)
            ogg_path = output_path.with_suffix(".ogg")
            sound.export(ogg_path, format="ogg", parameters=["-q:a", "6"])

            # Remove MP3, keep OGG
            mp3_path.unlink()
            return ogg_path
        except Exception as e:
            print(f"    Warning: Could not convert to OGG ({e}), keeping MP3")
            return mp3_path


# =============================================================================
# Prompt Loading
# =============================================================================

def load_prompts(prompts_file: Path) -> dict:
    """Load prompts from YAML file."""
    with open(prompts_file, "r") as f:
        return yaml.safe_load(f)


def get_generation_tasks(
    prompts: dict,
    faction_filter: str | None = None,
    category_filter: str | None = None,
) -> list[dict]:
    """Extract generation tasks from prompts structure.

    Returns list of dicts with: id, prompt, faction, category
    """
    tasks = []

    # Process faction-specific sounds
    factions = ["argentum", "symbiote", "obsidion", "neutral"]
    categories = ["attack", "summon", "death"]

    for faction in factions:
        if faction_filter and faction != faction_filter:
            continue

        faction_data = prompts.get(faction, {})

        for category in categories:
            if category_filter and category != category_filter:
                continue

            category_data = faction_data.get(category, [])

            for item in category_data:
                tasks.append({
                    "id": item["id"],
                    "prompt": item["prompt"],
                    "faction": faction,
                    "category": category,
                })

    # Process generic sounds
    if not faction_filter or faction_filter == "generic":
        generic_data = prompts.get("generic", {})

        for category, items in generic_data.items():
            # Skip non-list fields like 'description'
            if not isinstance(items, list):
                continue

            if category_filter and category != category_filter:
                continue

            for item in items:
                tasks.append({
                    "id": item["id"],
                    "prompt": item["prompt"],
                    "faction": "generic",
                    "category": category,
                })

    return tasks


# =============================================================================
# Main
# =============================================================================

def main():
    parser = argparse.ArgumentParser(description="Generate battle SFX using ElevenLabs API")
    parser.add_argument("--faction", type=str, help="Generate only for specific faction")
    parser.add_argument("--category", type=str, help="Generate only specific category")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be generated")
    parser.add_argument("--skip-existing", action="store_true", help="Skip if output file exists")
    parser.add_argument("--duration", type=float, default=DURATION_SECONDS, help="Audio duration in seconds")
    parser.add_argument("--delay", type=float, default=1.0, help="Delay between API calls (rate limiting)")
    args = parser.parse_args()

    # Load prompts
    if not PROMPTS_FILE.exists():
        print(f"ERROR: Prompts file not found: {PROMPTS_FILE}")
        sys.exit(1)

    prompts = load_prompts(PROMPTS_FILE)
    tasks = get_generation_tasks(prompts, args.faction, args.category)

    print("Essence Wars SFX Generator (ElevenLabs)")
    print("=" * 50)
    print(f"Prompts file: {PROMPTS_FILE}")
    print(f"Output dir:   {OUTPUT_DIR}")
    print(f"Tasks:        {len(tasks)} sounds to generate")
    print()

    if args.dry_run:
        print("DRY RUN - Would generate:")
        for task in tasks:
            output_path = OUTPUT_DIR / f"{task['id']}.ogg"
            exists = " (exists)" if output_path.exists() else ""
            print(f"  [{task['faction']:8}] {task['category']:8} -> {task['id']}{exists}")
        return

    # Initialize generator (validates API key)
    generator = SFXGenerator()

    # Generate sounds
    for i, task in enumerate(tasks, 1):
        output_path = OUTPUT_DIR / f"{task['id']}.ogg"

        # Skip if exists and requested
        if args.skip_existing and (output_path.exists() or output_path.with_suffix(".mp3").exists()):
            print(f"[{i}/{len(tasks)}] SKIP (exists): {task['id']}")
            continue

        print(f"[{i}/{len(tasks)}] Generating: {task['id']}")
        print(f"  Prompt: {task['prompt'][:60]}...")

        try:
            audio_data = generator.generate(task["prompt"], duration=args.duration)

            # Save
            saved_path = generator.save_audio(audio_data, output_path)
            print(f"  Saved: {saved_path}")

            # Rate limiting delay
            if i < len(tasks):
                time.sleep(args.delay)

        except Exception as e:
            print(f"  ERROR: {e}")
            continue

    print()
    print("Generation complete!")
    print(f"Output: {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
