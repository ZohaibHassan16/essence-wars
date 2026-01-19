#!/usr/bin/env python3
"""
Train Card2Vec embeddings for Essence Wars.

This script pre-trains card embeddings using multiple objectives:
1. Co-occurrence: Cards that appear together should have similar embeddings
2. Attribute prediction: Embeddings should encode card properties

The trained embeddings can be used with --observation-mode embedded_pretrained
in train_ppo.py or train_alphazero.py.

Usage:
    # Basic training (uses deck files only)
    python train_card2vec.py

    # Training with MCTS game data
    python train_card2vec.py --dataset data/datasets/mcts_100k.jsonl.gz

    # Custom embedding dimension
    python train_card2vec.py --embed-dim 128 --epochs 200

    # Quick test run
    python train_card2vec.py --epochs 10 --embed-dim 32

Example:
    # Train embeddings
    python train_card2vec.py --dataset data/datasets/mcts_10k_sims100_*.jsonl.gz

    # Use in PPO training
    python train_ppo.py --observation-mode embedded_pretrained \\
        --pretrained-embeds models/card2vec.pt
"""

import argparse
import sys
from datetime import datetime
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))


def parse_args():
    parser = argparse.ArgumentParser(
        description="Train Card2Vec embeddings for Essence Wars",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    # Data
    parser.add_argument(
        "--dataset",
        type=str,
        default=None,
        help="Path to MCTS dataset (.jsonl or .jsonl.gz) for co-occurrence learning",
    )
    parser.add_argument(
        "--decks-dir",
        type=str,
        default="data/decks",
        help="Directory containing deck files (default: data/decks)",
    )
    parser.add_argument(
        "--cards-dir",
        type=str,
        default="data/cards/core_set",
        help="Directory containing card YAML files (default: data/cards/core_set)",
    )

    # Model
    parser.add_argument(
        "--embed-dim",
        type=int,
        default=64,
        help="Embedding dimension (default: 64)",
    )

    # Training
    parser.add_argument(
        "--epochs",
        type=int,
        default=100,
        help="Number of training epochs (default: 100)",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=256,
        help="Batch size (default: 256)",
    )
    parser.add_argument(
        "--lr",
        type=float,
        default=1e-3,
        help="Learning rate (default: 1e-3)",
    )
    parser.add_argument(
        "--negative-ratio",
        type=int,
        default=5,
        help="Negative sampling ratio (default: 5)",
    )
    parser.add_argument(
        "--window-size",
        type=int,
        default=5,
        help="Co-occurrence window size (default: 5)",
    )
    parser.add_argument(
        "--max-pairs",
        type=int,
        default=500_000,
        help="Maximum co-occurrence pairs to keep (uses reservoir sampling, default: 500000)",
    )

    # Loss weights
    parser.add_argument(
        "--cooccur-weight",
        type=float,
        default=1.0,
        help="Weight for co-occurrence loss (default: 1.0)",
    )
    parser.add_argument(
        "--attr-weight",
        type=float,
        default=0.5,
        help="Weight for attribute prediction loss (default: 0.5)",
    )

    # Output
    parser.add_argument(
        "--output",
        type=str,
        default=None,
        help="Output path for embeddings (default: models/card2vec_<timestamp>.pt)",
    )
    parser.add_argument(
        "--save-interval",
        type=int,
        default=10,
        help="Save checkpoint every N epochs (default: 10)",
    )

    # Visualization
    parser.add_argument(
        "--visualize",
        action="store_true",
        help="Visualize embeddings after training",
    )
    parser.add_argument(
        "--viz-method",
        type=str,
        default="tsne",
        choices=["tsne", "umap"],
        help="Visualization method (default: tsne)",
    )

    return parser.parse_args()


def main():
    args = parse_args()

    # Setup output path
    if args.output is None:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = f"models/card2vec_{timestamp}.pt"
    else:
        output_path = args.output

    Path(output_path).parent.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("Card2Vec Pre-training")
    print("=" * 60)
    print(f"  Embedding dim:    {args.embed_dim}")
    print(f"  Epochs:           {args.epochs}")
    print(f"  Batch size:       {args.batch_size}")
    print(f"  Learning rate:    {args.lr}")
    print(f"  Window size:      {args.window_size}")
    print(f"  Negative ratio:   {args.negative_ratio}")
    print(f"  Max pairs:        {args.max_pairs:,}")
    print(f"  Cooccur weight:   {args.cooccur_weight}")
    print(f"  Attribute weight: {args.attr_weight}")
    if args.dataset:
        print(f"  MCTS dataset:     {args.dataset}")
    print(f"  Output:           {output_path}")
    print("=" * 60)

    # Create config
    from essence_wars.agents.card2vec import Card2VecConfig, train_card2vec, CardDatabase

    config = Card2VecConfig(
        embed_dim=args.embed_dim,
        mcts_dataset_path=args.dataset,
        decks_dir=args.decks_dir,
        cards_dir=args.cards_dir,
        epochs=args.epochs,
        batch_size=args.batch_size,
        learning_rate=args.lr,
        negative_ratio=args.negative_ratio,
        window_size=args.window_size,
        max_pairs=args.max_pairs,
        cooccurrence_weight=args.cooccur_weight,
        attribute_weight=args.attr_weight,
        output_path=output_path,
        save_interval=args.save_interval,
    )

    # Train
    print("\nStarting training...")
    model = train_card2vec(config, verbose=True)

    # Print some statistics
    print("\n" + "=" * 60)
    print("Training Complete")
    print("=" * 60)

    embeddings = model.get_embeddings()
    print(f"  Embedding shape: {embeddings.shape}")
    print(f"  Embedding norm (mean): {embeddings.norm(dim=1).mean():.4f}")
    print(f"  Saved to: {output_path}")

    # Visualize if requested
    if args.visualize:
        print("\nGenerating visualization...")
        from essence_wars.agents.card2vec import visualize_embeddings

        card_db = CardDatabase(args.cards_dir)
        viz_path = Path(output_path).with_suffix(".png")
        visualize_embeddings(model, card_db, str(viz_path), method=args.viz_method)

    # Show example similarities
    print("\n" + "=" * 60)
    print("Example Card Similarities")
    print("=" * 60)

    import torch
    device = next(model.parameters()).device

    card_db = CardDatabase(args.cards_dir)

    # Find most similar cards to a few examples
    example_ids = [1000, 2000, 3000, 4000]  # One from each faction

    for example_id in example_ids:
        card = card_db.get_card(example_id)
        if not card:
            continue

        # Get embedding
        with torch.no_grad():
            example_embed = model(torch.tensor([example_id], device=device)).squeeze()

            # Compute similarities to all cards
            all_ids = card_db.get_all_card_ids()
            all_embeds = model(torch.tensor(all_ids, device=device))
            similarities = F.cosine_similarity(
                example_embed.unsqueeze(0),
                all_embeds,
                dim=1
            )

        # Get top 5 most similar (excluding self)
        top_indices = similarities.argsort(descending=True)[1:6]
        top_ids = [all_ids[i] for i in top_indices]
        top_sims = [similarities[i].item() for i in top_indices]

        print(f"\n{card.name} (ID: {example_id}, {card.faction.title()}):")
        for sim_id, sim in zip(top_ids, top_sims):
            sim_card = card_db.get_card(sim_id)
            if sim_card:
                print(f"  {sim:.3f} - {sim_card.name} ({sim_card.faction.title()})")

    print("\n" + "=" * 60)
    print(f"Use these embeddings with:")
    print(f"  python train_ppo.py --observation-mode embedded_pretrained \\")
    print(f"    --pretrained-embeds {output_path}")
    print("=" * 60)


if __name__ == "__main__":
    # Import F for cosine_similarity in example section
    import torch.nn.functional as F
    main()
