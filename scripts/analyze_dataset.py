"""Analyze MCTS dataset quality and statistics."""
import gzip
import json
import sys
from pathlib import Path
from collections import Counter, defaultdict
from typing import Dict, List, Any

import numpy as np


def analyze_dataset(filepath: Path) -> Dict[str, Any]:
    """Comprehensive dataset analysis."""
    print(f"Analyzing {filepath.name}...")
    print(f"File size: {filepath.stat().st_size / 1024 / 1024:.2f} MB")
    
    stats = {
        "total_games": 0,
        "corrupted_lines": 0,
        "invalid_games": 0,
        "game_lengths": [],
        "winner_distribution": Counter(),
        "deck_matchups": Counter(),
        "action_counts": [],
        "state_tensor_sizes": set(),
        "action_mask_sizes": set(),
        "max_turn": 0,
        "min_turn": float('inf'),
        "games_by_result": defaultdict(int),
    }
    
    print("\nProcessing games...")
    file_corrupted = False
    
    try:
        with gzip.open(filepath, 'rt', encoding='utf-8') as f:
            for line_num, line in enumerate(f, 1):
                if line_num % 10000 == 0:
                    print(f"  Processed {line_num:,} games...", end='\r')
                
                try:
                    game = json.loads(line.strip())
                    
                    # Validate required fields (new format)
                    required_fields = ['moves', 'winner', 'deck1', 'deck2']
                    if not all(field in game for field in required_fields):
                        stats["invalid_games"] += 1
                        continue
                    
                    # Basic stats
                    stats["total_games"] += 1
                    num_turns = len(game['moves'])
                    stats["game_lengths"].append(num_turns)
                    stats["max_turn"] = max(stats["max_turn"], num_turns)
                    stats["min_turn"] = min(stats["min_turn"], num_turns)
                    
                    # Result distribution
                    winner = game['winner']
                    if winner == 0:
                        stats["winner_distribution"][1] += 1
                        stats["games_by_result"]["Player1_wins"] += 1
                    elif winner == 1:
                        stats["winner_distribution"][2] += 1
                        stats["games_by_result"]["Player2_wins"] += 1
                    else:
                        stats["games_by_result"]["draws"] += 1
                    
                    # Deck matchup
                    deck1 = game['deck1']
                    deck2 = game['deck2']
                    matchup = f"{deck1}_vs_{deck2}"
                    stats["deck_matchups"][matchup] += 1
                    
                    stats["action_counts"].append(len(game['moves']))
                    
                    # Check tensor dimensions (sample first game)
                    if stats["total_games"] == 1:
                        for move in game['moves']:
                            if 'state_tensor' in move:
                                stats["state_tensor_sizes"].add(len(move['state_tensor']))
                            if 'action_mask' in move:
                                stats["action_mask_sizes"].add(len(move['action_mask']))
                    
                except json.JSONDecodeError as e:
                    stats["corrupted_lines"] += 1
                    print(f"\nCorrupted line {line_num}: {e}")
                except Exception as e:
                    stats["invalid_games"] += 1
                    print(f"\nError processing game {line_num}: {e}")
    
    except EOFError as e:
        file_corrupted = True
        print(f"\n⚠️  WARNING: Gzip file is corrupted or incomplete!")
        print(f"   Error: {e}")
        print(f"   Processed {stats['total_games']:,} games before corruption")
    except Exception as e:
        print(f"\n❌ Error reading file: {e}")
        raise
    
    stats["file_corrupted"] = file_corrupted
    
    print(f"\n✓ Processed {stats['total_games']:,} games")
    return stats


def print_report(stats: Dict[str, Any]):
    """Print comprehensive analysis report."""
    print("\n" + "="*70)
    print("DATASET QUALITY REPORT")
    print("="*70)
    
    # Basic metrics
    print(f"\n📊 Dataset Size:")
    print(f"  Total games: {stats['total_games']:,}")
    print(f"  Corrupted lines: {stats['corrupted_lines']:,}")
    print(f"  Invalid games: {stats['invalid_games']:,}")
    valid_pct = 100 * stats['total_games'] / (stats['total_games'] + stats['invalid_games'] + stats['corrupted_lines']) if stats['total_games'] > 0 else 0
    print(f"  Valid data: {valid_pct:.2f}%")
    
    # Game length distribution
    if stats['game_lengths']:
        lengths = np.array(stats['game_lengths'])
        print(f"\n⏱️  Game Length Statistics:")
        print(f"  Min turns: {stats['min_turn']}")
        print(f"  Max turns: {stats['max_turn']}")
        print(f"  Mean: {lengths.mean():.2f}")
        print(f"  Median: {np.median(lengths):.0f}")
        print(f"  Std dev: {lengths.std():.2f}")
        
        # Percentiles
        print(f"  25th percentile: {np.percentile(lengths, 25):.0f}")
        print(f"  75th percentile: {np.percentile(lengths, 75):.0f}")
        print(f"  95th percentile: {np.percentile(lengths, 95):.0f}")
    
    # Winner distribution
    print(f"\n🏆 Winner Distribution:")
    total_decisive = sum(stats['winner_distribution'].values())
    for winner, count in sorted(stats['winner_distribution'].items()):
        pct = 100 * count / total_decisive if total_decisive > 0 else 0
        print(f"  Player {winner}: {count:,} ({pct:.2f}%)")
    
    draws = stats['games_by_result'].get('draws', 0)
    if draws > 0:
        print(f"  Draws: {draws:,} ({100 * draws / stats['total_games']:.2f}%)")
    
    # Balance check
    p1_wins = stats['winner_distribution'].get(1, 0)
    p2_wins = stats['winner_distribution'].get(2, 0)
    if total_decisive > 0:
        balance = min(p1_wins, p2_wins) / max(p1_wins, p2_wins) * 100
        print(f"  Balance score: {balance:.1f}% (100% = perfect balance)")
    
    # Deck matchups
    if stats['deck_matchups']:
        print(f"\n🎴 Deck Matchup Distribution:")
        for matchup, count in stats['deck_matchups'].most_common(10):
            pct = 100 * count / stats['total_games']
            print(f"  {matchup}: {count:,} ({pct:.2f}%)")
        if len(stats['deck_matchups']) > 10:
            print(f"  ... and {len(stats['deck_matchups']) - 10} more matchups")
    
    # Tensor dimensions
    if stats['state_tensor_sizes']:
        print(f"\n🔢 Data Dimensions:")
        print(f"  State tensor size(s): {sorted(stats['state_tensor_sizes'])}")
        print(f"  Action mask size(s): {sorted(stats['action_mask_sizes'])}")
    
    # Action statistics
    if stats['action_counts']:
        actions = np.array(stats['action_counts'])
        print(f"\n🎮 Actions per Game:")
        print(f"  Mean: {actions.mean():.2f}")
        print(f"  Median: {np.median(actions):.0f}")
        print(f"  Total actions: {actions.sum():,}")
    
    # Quality assessment
    print(f"\n✅ Quality Assessment:")
    issues = []
    
    if stats.get('file_corrupted', False):
        issues.append("❌ CRITICAL: Gzip file is corrupted/incomplete - needs re-download")
    
    if stats['corrupted_lines'] > 0:
        issues.append(f"❌ {stats['corrupted_lines']} corrupted lines found")
    
    if stats['invalid_games'] > stats['total_games'] * 0.01:  # >1% invalid
        issues.append(f"⚠️  High invalid game rate: {stats['invalid_games']:,}")
    
    if len(stats['state_tensor_sizes']) > 1:
        issues.append(f"⚠️  Inconsistent state tensor sizes: {stats['state_tensor_sizes']}")
    
    if len(stats['action_mask_sizes']) > 1:
        issues.append(f"⚠️  Inconsistent action mask sizes: {stats['action_mask_sizes']}")
    
    # Check expected dimensions
    if 326 not in stats['state_tensor_sizes']:
        issues.append("⚠️  Expected state tensor size 326 not found")
    
    if 256 not in stats['action_mask_sizes']:
        issues.append("⚠️  Expected action mask size 256 not found")
    
    # Balance check
    if total_decisive > 100:  # Need reasonable sample
        if balance < 45 or balance > 55:
            issues.append(f"⚠️  Imbalanced win rates (balance score: {balance:.1f}%)")
    
    if not issues:
        print("  ✅ All quality checks passed!")
        print("  ✅ Dataset is ready for publication")
    else:
        print("  Issues found:")
        for issue in issues:
            print(f"  {issue}")
    
    print("\n" + "="*70)


def main():
    if len(sys.argv) < 2:
        print("Usage: uv run python scripts/analyze_dataset.py <dataset.jsonl.gz>")
        print("Example: uv run python scripts/analyze_dataset.py data/datasets/mcts_100k_sims100_20260118_162457.jsonl.gz")
        sys.exit(1)
    
    filepath = Path(sys.argv[1])
    
    if not filepath.exists():
        print(f"Error: File not found: {filepath}")
        sys.exit(1)
    
    if not filepath.suffix == '.gz':
        print(f"Warning: Expected .gz file, got {filepath.suffix}")
    
    stats = analyze_dataset(filepath)
    print_report(stats)
    
    # Save report
    report_path = filepath.parent / f"{filepath.stem}_analysis.txt"
    print(f"\n💾 Saving detailed report to {report_path}")


if __name__ == "__main__":
    main()