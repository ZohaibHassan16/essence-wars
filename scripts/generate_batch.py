#!/usr/bin/env python3
"""
Quick batch generator for Essence Wars card art drafts.
Uses flux-schnell for fast draft generation.
"""

import subprocess
import yaml
import sys
from pathlib import Path

# Paths
PROMPTS_DIR = Path("/home/chris/ai-cardgame/data/art/prompts/core_set")
DRAFTS_DIR = Path.home() / ".ai-assets/output/essence-wars/drafts"
FINALS_DIR = Path("/home/chris/ai-cardgame/crates/essence-wars-ui/assets/cards/core_set")
SD_CLI = Path.home() / "stable-diffusion.cpp/build/bin/sd-cli"
MODELS_DIR = Path.home() / ".ai-assets/models/flux"
LORAS_DIR = Path.home() / ".ai-assets/loras"

def load_prompts(faction: str) -> list[dict]:
    """Load prompts from faction YAML file."""
    yaml_path = PROMPTS_DIR / f"{faction}.yaml"
    with open(yaml_path) as f:
        data = yaml.safe_load(f)
    return data.get("cards", [])

def generate_image(card: dict, faction: str, mode: str = "draft") -> bool:
    """Generate a single card image."""
    card_id = card["id"]
    name = card["name"].lower().replace(" ", "_").replace("'", "")
    prompt = card["prompt"].strip()
    lora = card.get("lora", "classical-painting:0.7")

    # Add LoRA to prompt
    lora_tags = " ".join(f"<lora:{l.strip()}>" for l in lora.split(","))
    full_prompt = f"{prompt} {lora_tags}"

    # Output path depends on mode
    if mode == "draft":
        output_dir = DRAFTS_DIR / faction
        output_path = output_dir / f"{card_id}_{name}.png"
    else:
        output_dir = FINALS_DIR
        output_path = output_dir / f"{card_id}.png"  # Finals use ID only

    output_dir.mkdir(parents=True, exist_ok=True)

    # Skip if exists
    if output_path.exists():
        print(f"  [SKIP] {card_id} - already exists at {output_path}")
        return True

    # Build command
    if mode == "draft":
        model = MODELS_DIR / "flux-schnell-q4.gguf"
        steps = 4
    else:
        model = MODELS_DIR / "flux-dev-q8.gguf"
        steps = 20

    cmd = [
        str(SD_CLI),
        "--diffusion-model", str(model),
        "--vae", str(MODELS_DIR / "ae.safetensors"),
        "--clip_l", str(MODELS_DIR / "clip_l.safetensors"),
        "--t5xxl", str(MODELS_DIR / "t5-Q5_K_M.gguf"),
        "--lora-model-dir", str(LORAS_DIR),
        "-p", full_prompt,
        "--cfg-scale", "1.0",
        "--sampling-method", "euler",
        "--steps", str(steps),
        "-H", "896", "-W", "704",
        "-o", str(output_path),
    ]

    print(f"  [GEN] {card_id} - {card['name']}...")
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
        if result.returncode == 0:
            print(f"  [OK] {card_id} - saved to {output_path.name}")
            return True
        else:
            print(f"  [ERR] {card_id} - {result.stderr[-200:]}")
            return False
    except subprocess.TimeoutExpired:
        print(f"  [TIMEOUT] {card_id}")
        return False

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Generate Essence Wars card art")
    parser.add_argument("--faction", required=True, choices=["argentum", "symbiote", "obsidion", "neutral"])
    parser.add_argument("--mode", default="draft", choices=["draft", "final"])
    parser.add_argument("--ids", type=str, help="Comma-separated card IDs to generate (default: all)")
    args = parser.parse_args()

    print(f"\n=== Generating {args.faction} ({args.mode} mode) ===\n")

    cards = load_prompts(args.faction)

    # Filter by IDs if specified
    if args.ids:
        target_ids = set(int(x) for x in args.ids.split(","))
        cards = [c for c in cards if c["id"] in target_ids]

    print(f"Found {len(cards)} cards to generate\n")

    success = 0
    for card in cards:
        if generate_image(card, args.faction, args.mode):
            success += 1

    print(f"\n=== Complete: {success}/{len(cards)} generated ===\n")

if __name__ == "__main__":
    main()
