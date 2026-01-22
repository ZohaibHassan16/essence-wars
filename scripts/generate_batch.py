#!/usr/bin/env python3
"""
Quick batch generator for Essence Wars card art drafts.
Uses flux-schnell for fast draft generation.
"""

import subprocess
import yaml
import sys
import json
from pathlib import Path
from datetime import datetime

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

def generate_image(card: dict, faction: str, mode: str = "draft", verbose: bool = False, error_log: list = None) -> bool:
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
    
    if verbose:
        print(f"    Command: {' '.join(cmd[:10])}... (truncated)")
        print(f"    Prompt: {prompt[:100]}...")
    
    try:
        # In verbose mode, don't capture output so user can see progress
        # Increase timeout for flux-dev (final mode) - it can take 3-5 minutes per image
        timeout_seconds = 300 if mode == "final" else 180
        
        if verbose:
            result = subprocess.run(cmd, timeout=timeout_seconds)
            success = result.returncode == 0
        else:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout_seconds)
            success = result.returncode == 0
            
        if success:
            print(f"  [OK] {card_id} - saved to {output_path.name}")
            return True
        else:
            # Print both stdout and stderr for debugging
            if not verbose:
                error_msg = result.stderr.strip() if result.stderr else result.stdout.strip()
                if error_msg:
                    # Show last 300 chars to get more context
                    print(f"  [ERR] {card_id} - {error_msg[-300:]}")
                else:
                    print(f"  [ERR] {card_id} - No error message (return code: {result.returncode})")
                
                # Log full error details
                if error_log is not None:
                    error_log.append({
                        "card_id": card_id,
                        "name": card["name"],
                        "return_code": result.returncode,
                        "stderr": result.stderr if result.stderr else "",
                        "stdout": result.stdout if result.stdout else "",
                        "prompt": prompt[:200],
                        "lora": lora
                    })
            else:
                print(f"  [ERR] {card_id} - Command failed with return code: {result.returncode}")
            return False
    except subprocess.TimeoutExpired:
        print(f"  [TIMEOUT] {card_id}")
        if error_log is not None:
            error_log.append({
                "card_id": card_id,
                "name": card["name"],
                "error": "timeout",
                "timeout_seconds": timeout_seconds,
                "prompt": prompt[:200]
            })
        return False
    except Exception as e:
        print(f"  [ERR] {card_id} - Exception: {str(e)}")
        if error_log is not None:
            error_log.append({
                "card_id": card_id,
                "name": card["name"],
                "error": "exception",
                "exception": str(e)
            })
        return False

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Generate Essence Wars card art")
    parser.add_argument("--faction", required=True, choices=["argentum", "symbiote", "obsidion", "neutral"])
    parser.add_argument("--mode", default="draft", choices=["draft", "final"])
    parser.add_argument("--ids", type=str, help="Comma-separated card IDs to generate (default: all)")
    parser.add_argument("--verbose", "-v", action="store_true", help="Show detailed output from sd-cli")
    args = parser.parse_args()

    print(f"\n=== Generating {args.faction} ({args.mode} mode) ===\n")

    cards = load_prompts(args.faction)

    # Filter by IDs if specified
    if args.ids:
        target_ids = set(int(x) for x in args.ids.split(","))
        cards = [c for c in cards if c["id"] in target_ids]

    print(f"Found {len(cards)} cards to generate\n")

    error_log = []
    success = 0
    for card in cards:
        if generate_image(card, args.faction, args.mode, args.verbose, error_log):
            success += 1

    print(f"\n=== Complete: {success}/{len(cards)} generated ===\n")
    
    # Save error log if there were failures
    if error_log:
        log_file = Path("/home/chris/ai-cardgame/scripts") / f"generation_errors_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
        with open(log_file, 'w') as f:
            json.dump(error_log, f, indent=2)
        print(f"Error details saved to: {log_file}\n")

if __name__ == "__main__":
    main()
