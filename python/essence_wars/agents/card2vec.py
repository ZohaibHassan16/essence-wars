"""
Card2Vec: Pre-trained Card Embeddings for Essence Wars.

This module provides infrastructure for pre-training card embeddings using
multiple objectives:

1. **Co-occurrence Learning**: Cards that appear together (in decks, hands,
   or on the board) should have similar embeddings.

2. **Attribute Prediction**: Embeddings should encode card properties
   (cost, attack, health, keywords, card type).

3. **Faction Clustering**: Cards from the same faction should be closer
   in embedding space.

The pre-trained embeddings can be loaded into EmbeddedPPONetwork or
EmbeddedAlphaZeroNetwork for transfer learning.

Usage:
    # Train Card2Vec embeddings
    python scripts/train_card2vec.py --dataset data/datasets/mcts_100k.jsonl.gz

    # Load in network
    network = EmbeddedPPONetwork(
        pretrained_embeds=torch.load("models/card2vec.pt"),
        freeze_embeds=False,  # Fine-tune during RL
    )

See docs/embedding-design.md for full design documentation.
"""

from __future__ import annotations

import gzip
import json
import random
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Any

import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.utils.data import DataLoader, Dataset

if TYPE_CHECKING:
    from collections.abc import Iterator

# Card ID ranges
ARGENTUM_RANGE = (1000, 1074)
SYMBIOTE_RANGE = (2000, 2074)
OBSIDION_RANGE = (3000, 3074)
NEUTRAL_RANGE = (4000, 4074)

# Card ID positions in state tensor (from embeddings.py)
P1_HAND_START, P1_HAND_END = 11, 21
P2_HAND_START, P2_HAND_END = 86, 96

# Keywords (14 total)
KEYWORD_NAMES = [
    "Rush", "Ranged", "Piercing", "Guard", "Lifesteal", "Lethal",
    "Shield", "Quick", "Ephemeral", "Regenerate", "Stealth", "Charge",
    "Frenzy", "Volatile"
]
NUM_KEYWORDS = len(KEYWORD_NAMES)

# Maximum card ID (for embedding table size)
MAX_CARD_ID = 5000


# =============================================================================
# Card Database
# =============================================================================

@dataclass
class CardInfo:
    """Information about a single card."""
    card_id: int
    name: str
    cost: int
    card_type: str  # "creature", "spell", "support"
    attack: int = 0
    health: int = 0
    keywords: list[str] = field(default_factory=list)
    faction: str = "neutral"
    rarity: str = "Common"


class CardDatabase:
    """
    Database of all cards with their attributes.

    Loads card definitions from YAML files.
    """

    def __init__(self, cards_dir: str | Path = "data/cards/core_set"):
        self.cards_dir = Path(cards_dir)
        self.cards: dict[int, CardInfo] = {}
        self._load_cards()

    def _load_cards(self) -> None:
        """Load all card definitions from YAML files.

        Supports two directory structures:
        1. Flat: cards_dir/argentum.yaml, symbiote.yaml, etc.
        2. Nested: cards_dir/argentum/{creatures,spells,supports}.yaml
        """
        try:
            import yaml  # type: ignore[import-untyped]
        except ImportError:
            print("Warning: PyYAML not installed, using fallback card loading")
            self._load_cards_fallback()
            return

        factions = ["argentum", "symbiote", "obsidion", "neutral"]

        for faction in factions:
            # Try flat structure first: faction.yaml
            flat_path = self.cards_dir / f"{faction}.yaml"
            if flat_path.exists():
                self._load_yaml_file(flat_path, faction, yaml)
                continue

            # Try nested structure: faction/{creatures,spells,supports}.yaml
            faction_dir = self.cards_dir / faction
            if faction_dir.is_dir():
                for card_type_file in ["creatures.yaml", "spells.yaml", "supports.yaml"]:
                    filepath = faction_dir / card_type_file
                    if filepath.exists():
                        self._load_yaml_file(filepath, faction, yaml)

    def _load_yaml_file(self, filepath: Path, faction: str, yaml: Any) -> None:
        """Load cards from a single YAML file."""
        with filepath.open() as f:
            data = yaml.safe_load(f)

        if data is None:
            return

        for card_data in data.get("cards", []):
            card = CardInfo(
                card_id=card_data["id"],
                name=card_data["name"],
                cost=card_data.get("cost", 0),
                card_type=card_data.get("card_type", "creature"),
                attack=card_data.get("attack", 0),
                health=card_data.get("health", 0),
                keywords=card_data.get("keywords", []),
                faction=faction,
                rarity=card_data.get("rarity", "Common"),
            )
            self.cards[card.card_id] = card

    def _load_cards_fallback(self) -> None:
        """Fallback: create dummy cards for all ID ranges."""
        for faction, (start, end) in [
            ("argentum", ARGENTUM_RANGE),
            ("symbiote", SYMBIOTE_RANGE),
            ("obsidion", OBSIDION_RANGE),
            ("neutral", NEUTRAL_RANGE),
        ]:
            for card_id in range(start, end + 1):
                self.cards[card_id] = CardInfo(
                    card_id=card_id,
                    name=f"Card_{card_id}",
                    cost=random.randint(1, 8),
                    card_type="creature",
                    attack=random.randint(1, 6),
                    health=random.randint(1, 8),
                    keywords=[],
                    faction=faction,
                )

    def get_card(self, card_id: int) -> CardInfo | None:
        """Get card info by ID."""
        return self.cards.get(card_id)

    def get_faction(self, card_id: int) -> str:
        """Get faction for a card ID."""
        if ARGENTUM_RANGE[0] <= card_id <= ARGENTUM_RANGE[1]:
            return "argentum"
        elif SYMBIOTE_RANGE[0] <= card_id <= SYMBIOTE_RANGE[1]:
            return "symbiote"
        elif OBSIDION_RANGE[0] <= card_id <= OBSIDION_RANGE[1]:
            return "obsidion"
        else:
            return "neutral"

    def get_all_card_ids(self) -> list[int]:
        """Get all valid card IDs."""
        return list(self.cards.keys())

    def __len__(self) -> int:
        return len(self.cards)


# =============================================================================
# Data Extraction
# =============================================================================

def extract_card_ids_from_tensor(tensor: list[float]) -> list[int]:
    """
    Extract card IDs from a state tensor.

    Returns card IDs from player hands (positions 11-20 and 86-95).
    Filters out zeros (empty slots) and invalid IDs.
    """
    card_ids = []

    # Player 1 hand
    for i in range(P1_HAND_START, P1_HAND_END):
        if i < len(tensor):
            card_id = int(tensor[i])
            if 1000 <= card_id <= 4999:
                card_ids.append(card_id)

    # Player 2 hand
    for i in range(P2_HAND_START, P2_HAND_END):
        if i < len(tensor):
            card_id = int(tensor[i])
            if 1000 <= card_id <= 4999:
                card_ids.append(card_id)

    return card_ids


def load_deck_cards(decks_dir: str | Path = "data/decks") -> list[list[int]]:
    """
    Load card lists from all deck files.

    Returns a list of decks, where each deck is a list of card IDs.
    """
    decks_dir = Path(decks_dir)
    decks = []

    try:
        import tomllib
    except ImportError:
        import tomli as tomllib  # type: ignore[import-not-found,no-redef]

    for deck_file in decks_dir.glob("**/*.toml"):
        try:
            with deck_file.open("rb") as f:
                deck_data = tomllib.load(f)
            if "cards" in deck_data:
                decks.append(deck_data["cards"])
        except Exception as e:
            print(f"Warning: Could not load {deck_file}: {e}")

    return decks


def generate_cooccurrence_pairs(
    card_ids: list[int],
    window_size: int = 5,
) -> Iterator[tuple[int, int]]:
    """
    Generate (center, context) pairs from a list of card IDs.

    Uses skip-gram style sampling: for each card, sample context cards
    within a window.
    """
    for i, center in enumerate(card_ids):
        # Context window
        start = max(0, i - window_size)
        end = min(len(card_ids), i + window_size + 1)

        for j in range(start, end):
            if i != j:
                yield (center, card_ids[j])


# =============================================================================
# Dataset Classes
# =============================================================================

class CooccurrenceDataset(Dataset[tuple[int, int, float]]):
    """
    Dataset for card co-occurrence learning.

    Each sample is a (center_card_id, context_card_id, label) tuple.
    Positive pairs come from actual co-occurrences, negative pairs are sampled.
    """

    def __init__(
        self,
        positive_pairs: list[tuple[int, int]],
        all_card_ids: list[int],
        negative_ratio: int = 5,
    ):
        self.positive_pairs = positive_pairs
        self.all_card_ids = all_card_ids
        self.negative_ratio = negative_ratio

        # Build negative sampling distribution (uniform for now)
        self.card_id_to_idx = {cid: i for i, cid in enumerate(all_card_ids)}

    def __len__(self) -> int:
        return len(self.positive_pairs) * (1 + self.negative_ratio)

    def __getitem__(self, idx: int) -> tuple[int, int, float]:
        num_positive = len(self.positive_pairs)

        if idx < num_positive:
            # Positive sample
            center, context = self.positive_pairs[idx]
            return (center, context, 1.0)
        else:
            # Negative sample
            pos_idx = (idx - num_positive) % num_positive
            center, _ = self.positive_pairs[pos_idx]
            # Sample random negative context
            context = random.choice(self.all_card_ids)
            return (center, context, 0.0)


class AttributeDataset(Dataset[dict[str, torch.Tensor]]):
    """
    Dataset for card attribute prediction.

    Each sample is a card ID with its attributes as tensor targets.
    """

    def __init__(self, card_db: CardDatabase):
        self.card_db = card_db
        self.card_ids = card_db.get_all_card_ids()

        # Precompute attribute tensors for efficient loading
        self._precompute_tensors()

    def _precompute_tensors(self) -> None:
        """Precompute all attribute tensors."""
        n = len(self.card_ids)

        self.costs = torch.zeros(n, dtype=torch.float32)
        self.attacks = torch.zeros(n, dtype=torch.float32)
        self.healths = torch.zeros(n, dtype=torch.float32)
        self.card_types = torch.zeros(n, dtype=torch.long)  # Class index
        self.factions = torch.zeros(n, dtype=torch.long)    # Class index
        self.keywords = torch.zeros(n, NUM_KEYWORDS, dtype=torch.float32)

        card_type_map = {"creature": 0, "spell": 1, "support": 2}
        faction_map = {"argentum": 0, "symbiote": 1, "obsidion": 2, "neutral": 3}

        for i, card_id in enumerate(self.card_ids):
            card = self.card_db.get_card(card_id)
            if card:
                self.costs[i] = card.cost / 10.0
                self.attacks[i] = card.attack / 10.0
                self.healths[i] = card.health / 10.0
                self.card_types[i] = card_type_map.get(card.card_type, 0)
                self.factions[i] = faction_map.get(card.faction, 3)

                for kw in card.keywords:
                    if kw in KEYWORD_NAMES:
                        self.keywords[i, KEYWORD_NAMES.index(kw)] = 1.0

    def __len__(self) -> int:
        return len(self.card_ids)

    def __getitem__(self, idx: int) -> dict[str, torch.Tensor]:
        return {
            "card_id": torch.tensor(self.card_ids[idx], dtype=torch.long),
            "cost": self.costs[idx],
            "attack": self.attacks[idx],
            "health": self.healths[idx],
            "card_type": self.card_types[idx],
            "faction": self.factions[idx],
            "keywords": self.keywords[idx],
        }


# =============================================================================
# Card2Vec Model
# =============================================================================

class Card2Vec(nn.Module):
    """
    Card2Vec embedding model with multiple training objectives.

    Architecture:
    - Embedding layer: card_id -> embed_dim vector
    - Co-occurrence head: predict if two cards appear together
    - Attribute heads: predict card properties from embedding

    Args:
        num_cards: Size of embedding table
        embed_dim: Embedding dimension
        num_keywords: Number of keywords to predict
    """

    def __init__(
        self,
        num_cards: int = MAX_CARD_ID,
        embed_dim: int = 64,
        num_keywords: int = NUM_KEYWORDS,
    ):
        super().__init__()

        self.num_cards = num_cards
        self.embed_dim = embed_dim

        # Main embedding layer
        self.embeddings = nn.Embedding(num_cards, embed_dim, padding_idx=0)

        # Context embedding for skip-gram (separate from main embeddings)
        self.context_embeddings = nn.Embedding(num_cards, embed_dim, padding_idx=0)

        # Attribute prediction heads
        self.cost_head = nn.Linear(embed_dim, 1)
        self.attack_head = nn.Linear(embed_dim, 1)
        self.health_head = nn.Linear(embed_dim, 1)
        self.card_type_head = nn.Linear(embed_dim, 3)  # creature, spell, support
        self.faction_head = nn.Linear(embed_dim, 4)    # argentum, symbiote, obsidion, neutral
        self.keyword_head = nn.Linear(embed_dim, num_keywords)

        self._init_weights()

    def _init_weights(self) -> None:
        """Initialize weights."""
        nn.init.normal_(self.embeddings.weight, mean=0.0, std=0.1)
        nn.init.normal_(self.context_embeddings.weight, mean=0.0, std=0.1)

        # Zero out padding embeddings
        self.embeddings.weight.data[0].zero_()
        self.context_embeddings.weight.data[0].zero_()

        for module in [self.cost_head, self.attack_head, self.health_head,
                       self.card_type_head, self.faction_head, self.keyword_head]:
            nn.init.xavier_uniform_(module.weight)
            nn.init.zeros_(module.bias)

    def forward(self, card_ids: torch.Tensor) -> torch.Tensor:
        """Get embeddings for card IDs."""
        return self.embeddings(card_ids)

    def cooccurrence_score(
        self,
        center_ids: torch.Tensor,
        context_ids: torch.Tensor,
    ) -> torch.Tensor:
        """
        Compute co-occurrence score (dot product similarity).

        Args:
            center_ids: Center card IDs (batch,)
            context_ids: Context card IDs (batch,)

        Returns:
            Similarity scores (batch,)
        """
        center_embeds = self.embeddings(center_ids)       # (batch, embed_dim)
        context_embeds = self.context_embeddings(context_ids)  # (batch, embed_dim)

        # Dot product similarity
        scores = (center_embeds * context_embeds).sum(dim=-1)
        return scores

    def predict_attributes(
        self,
        card_ids: torch.Tensor,
    ) -> dict[str, torch.Tensor]:
        """
        Predict card attributes from embeddings.

        Args:
            card_ids: Card IDs (batch,)

        Returns:
            Dictionary of attribute predictions
        """
        embeds = self.embeddings(card_ids)

        return {
            "cost": self.cost_head(embeds).squeeze(-1),
            "attack": self.attack_head(embeds).squeeze(-1),
            "health": self.health_head(embeds).squeeze(-1),
            "card_type": self.card_type_head(embeds),
            "faction": self.faction_head(embeds),
            "keywords": torch.sigmoid(self.keyword_head(embeds)),
        }

    def get_embeddings(self) -> torch.Tensor:
        """Get the embedding weight matrix."""
        return self.embeddings.weight.detach()


# =============================================================================
# Training Utilities
# =============================================================================

@dataclass
class Card2VecConfig:
    """Configuration for Card2Vec training."""

    # Model
    embed_dim: int = 64
    num_cards: int = MAX_CARD_ID

    # Data
    mcts_dataset_path: str | None = None
    decks_dir: str = "data/decks"
    cards_dir: str = "data/cards/core_set"
    negative_ratio: int = 5
    window_size: int = 5
    max_pairs: int = 500_000  # Memory limit for co-occurrence pairs

    # Training
    epochs: int = 100
    batch_size: int = 256
    learning_rate: float = 1e-3
    weight_decay: float = 1e-4

    # Loss weights
    cooccurrence_weight: float = 1.0
    attribute_weight: float = 0.5

    # Output
    output_path: str = "models/card2vec.pt"
    save_interval: int = 10


def collect_cooccurrence_pairs(
    decks_dir: str = "data/decks",
    mcts_dataset_path: str | None = None,
    window_size: int = 5,
    max_games: int | None = None,
    max_pairs: int = 500_000,
) -> list[tuple[int, int]]:
    """
    Collect co-occurrence pairs from decks and MCTS games.

    Uses reservoir sampling to limit memory usage when collecting from
    large datasets. This prevents OOM crashes on 10k+ game datasets.

    Args:
        decks_dir: Directory containing deck files
        mcts_dataset_path: Path to MCTS dataset (optional)
        window_size: Window size for skip-gram sampling
        max_games: Maximum games to process from MCTS dataset
        max_pairs: Maximum pairs to collect (uses reservoir sampling)

    Returns:
        List of (center_card_id, context_card_id) pairs
    """
    pairs: list[tuple[int, int]] = []
    total_seen = 0

    def _add_pair(pair: tuple[int, int]) -> None:
        """Add pair using reservoir sampling if at capacity."""
        nonlocal total_seen
        total_seen += 1

        if len(pairs) < max_pairs:
            pairs.append(pair)
        else:
            # Reservoir sampling: replace random element with decreasing probability
            idx = random.randint(0, total_seen - 1)
            if idx < max_pairs:
                pairs[idx] = pair

    # Collect from deck files
    print("Loading deck co-occurrences...")
    decks = load_deck_cards(decks_dir)
    deck_pairs = 0
    for deck in decks:
        for pair in generate_cooccurrence_pairs(deck, window_size):
            _add_pair(pair)
            deck_pairs += 1
    print(f"  Saw {deck_pairs:,} pairs from {len(decks)} decks")

    # Collect from MCTS dataset
    if mcts_dataset_path and Path(mcts_dataset_path).exists():
        print(f"Loading MCTS co-occurrences from {mcts_dataset_path}...")
        print(f"  (max_pairs={max_pairs:,}, using reservoir sampling)")
        mcts_pairs = 0

        opener = gzip.open if mcts_dataset_path.endswith(".gz") else open
        with opener(mcts_dataset_path, "rt") as f:
            for i, line in enumerate(f):
                if max_games and i >= max_games:
                    break

                # Progress indicator every 1000 games
                if (i + 1) % 1000 == 0:
                    print(f"    Processed {i + 1:,} games, {len(pairs):,} pairs collected...")

                try:
                    game = json.loads(line)
                    for move in game.get("moves", []):
                        tensor = move.get("state_tensor", [])
                        card_ids = extract_card_ids_from_tensor(tensor)
                        if len(card_ids) >= 2:
                            for pair in generate_cooccurrence_pairs(card_ids, window_size):
                                _add_pair(pair)
                                mcts_pairs += 1
                except json.JSONDecodeError:
                    continue

        print(f"  Saw {mcts_pairs:,} pairs from MCTS games")

    print(f"Total pairs seen: {total_seen:,}, kept: {len(pairs):,} (reservoir sampling)")
    return pairs


def train_card2vec(
    config: Card2VecConfig,
    verbose: bool = True,
) -> Card2Vec:
    """
    Train Card2Vec embeddings.

    Args:
        config: Training configuration
        verbose: Print progress

    Returns:
        Trained Card2Vec model
    """
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    if verbose:
        print(f"Training on {device}")

    # Load card database
    card_db = CardDatabase(config.cards_dir)
    all_card_ids = card_db.get_all_card_ids()
    if verbose:
        print(f"Loaded {len(card_db)} cards")

    # Collect co-occurrence pairs
    pairs = collect_cooccurrence_pairs(
        decks_dir=config.decks_dir,
        mcts_dataset_path=config.mcts_dataset_path,
        window_size=config.window_size,
        max_games=None,  # Process all games, but use max_pairs for memory
        max_pairs=config.max_pairs,
    )

    # Create datasets
    cooccur_dataset = CooccurrenceDataset(
        positive_pairs=pairs,
        all_card_ids=all_card_ids,
        negative_ratio=config.negative_ratio,
    )
    attr_dataset = AttributeDataset(card_db)

    if verbose:
        print(f"Co-occurrence samples: {len(cooccur_dataset):,}")
        print(f"Attribute samples: {len(attr_dataset):,}")

    # Create model
    model = Card2Vec(
        num_cards=config.num_cards,
        embed_dim=config.embed_dim,
    ).to(device)

    # Optimizer
    optimizer = torch.optim.AdamW(
        model.parameters(),
        lr=config.learning_rate,
        weight_decay=config.weight_decay,
    )

    # Data loaders (only create if dataset is non-empty)
    cooccur_loader = None
    attr_loader = None

    if len(cooccur_dataset) > 0:
        cooccur_loader = DataLoader(
            cooccur_dataset,
            batch_size=config.batch_size,
            shuffle=True,
            num_workers=0,
        )
    else:
        print("Warning: No co-occurrence data, skipping co-occurrence training")

    if len(attr_dataset) > 0:
        attr_loader = DataLoader(
            attr_dataset,
            batch_size=config.batch_size,
            shuffle=True,
            num_workers=0,
        )
    else:
        print("Warning: No attribute data, skipping attribute prediction training")

    # Check if we have any data to train on
    if cooccur_loader is None and attr_loader is None:
        raise ValueError(
            "No training data available. Ensure either:\n"
            "  - Deck files exist in the decks directory for co-occurrence learning\n"
            "  - Card YAML files exist for attribute prediction\n"
            "  - An MCTS dataset is provided via --dataset"
        )

    # Training loop
    output_path = Path(config.output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    for epoch in range(config.epochs):
        model.train()
        total_cooccur_loss = 0.0
        total_attr_loss = 0.0
        num_cooccur_batches = 0
        num_attr_batches = 0

        # Co-occurrence training
        if cooccur_loader is not None:
            for center, context, label in cooccur_loader:
                center = center.to(device)
                context = context.to(device)
                label = label.float().to(device)

                optimizer.zero_grad()

                scores = model.cooccurrence_score(center, context)
                loss = F.binary_cross_entropy_with_logits(scores, label)
                loss = loss * config.cooccurrence_weight

                loss.backward()
                optimizer.step()

                total_cooccur_loss += loss.item()
                num_cooccur_batches += 1

        # Attribute training
        if attr_loader is not None:
            for batch in attr_loader:
                card_ids = batch["card_id"].to(device)
                cost_target = batch["cost"].to(device)
                attack_target = batch["attack"].to(device)
                health_target = batch["health"].to(device)
                card_type_target = batch["card_type"].to(device)
                faction_target = batch["faction"].to(device)
                keywords_target = batch["keywords"].to(device)

                optimizer.zero_grad()

                preds = model.predict_attributes(card_ids)

                # Compute attribute losses
                attr_loss: torch.Tensor = torch.tensor(0.0, device=device)

                # Continuous attributes (MSE)
                attr_loss = attr_loss + F.mse_loss(preds["cost"], cost_target)
                attr_loss = attr_loss + F.mse_loss(preds["attack"], attack_target)
                attr_loss = attr_loss + F.mse_loss(preds["health"], health_target)

                # Categorical attributes (CE)
                attr_loss = attr_loss + F.cross_entropy(preds["card_type"], card_type_target)
                attr_loss = attr_loss + F.cross_entropy(preds["faction"], faction_target)

                # Keywords (BCE)
                attr_loss = attr_loss + F.binary_cross_entropy(preds["keywords"], keywords_target)

                attr_loss = attr_loss * config.attribute_weight

                attr_loss.backward()
                optimizer.step()

                total_attr_loss += attr_loss.item()
                num_attr_batches += 1

        # Logging
        avg_cooccur = total_cooccur_loss / max(num_cooccur_batches, 1)
        avg_attr = total_attr_loss / max(num_attr_batches, 1)

        if verbose and (epoch + 1) % 10 == 0:
            print(f"Epoch {epoch + 1}/{config.epochs} | "
                  f"Cooccur: {avg_cooccur:.4f} | Attr: {avg_attr:.4f}")

        # Save checkpoint
        if (epoch + 1) % config.save_interval == 0:
            save_card2vec(model, config, output_path)

    # Final save
    save_card2vec(model, config, output_path)
    if verbose:
        print(f"Saved embeddings to {output_path}")

    return model


def save_card2vec(model: Card2Vec, config: Card2VecConfig, path: Path) -> None:
    """Save Card2Vec model and embeddings."""
    torch.save({
        "embeddings": model.get_embeddings(),
        "model_state_dict": model.state_dict(),
        "config": {
            "embed_dim": config.embed_dim,
            "num_cards": config.num_cards,
        },
    }, path)


def load_card2vec_embeddings(path: str | Path) -> torch.Tensor:
    """
    Load pre-trained embeddings from a Card2Vec checkpoint.

    Args:
        path: Path to saved Card2Vec model

    Returns:
        Embedding weight tensor of shape (num_cards, embed_dim)
    """
    checkpoint = torch.load(path, map_location="cpu", weights_only=False)

    if isinstance(checkpoint, dict):
        if "embeddings" in checkpoint:
            return checkpoint["embeddings"]
        elif "model_state_dict" in checkpoint:
            return checkpoint["model_state_dict"]["embeddings.weight"]

    # Assume it's just the embeddings tensor
    return checkpoint


# =============================================================================
# Visualization Utilities
# =============================================================================

def visualize_embeddings(
    model: Card2Vec,
    card_db: CardDatabase,
    output_path: str | None = None,
    method: str = "tsne",
) -> None:
    """
    Visualize card embeddings using dimensionality reduction.

    Args:
        model: Trained Card2Vec model
        card_db: Card database for labels
        output_path: Path to save plot (optional)
        method: "tsne" or "umap"
    """
    try:
        import matplotlib.pyplot as plt
        from sklearn.manifold import TSNE  # type: ignore[import-not-found]
    except ImportError:
        print("matplotlib and sklearn required for visualization")
        return

    # Get embeddings for all cards
    card_ids = card_db.get_all_card_ids()
    card_ids_tensor = torch.tensor(card_ids, dtype=torch.long)

    with torch.no_grad():
        embeddings = model(card_ids_tensor).numpy()

    # Reduce to 2D
    if method == "tsne":
        reducer = TSNE(n_components=2, random_state=42, perplexity=30)
    else:
        try:
            from umap import UMAP  # type: ignore[import-not-found]
            reducer = UMAP(n_components=2, random_state=42)
        except ImportError:
            print("UMAP not installed, using t-SNE")
            reducer = TSNE(n_components=2, random_state=42, perplexity=30)

    reduced = reducer.fit_transform(embeddings)

    # Plot by faction
    faction_colors = {
        "argentum": "blue",
        "symbiote": "green",
        "obsidion": "purple",
        "neutral": "gray",
    }

    plt.figure(figsize=(12, 10))

    for faction, color in faction_colors.items():
        mask = [card_db.get_faction(cid) == faction for cid in card_ids]
        faction_points = reduced[mask]
        if len(faction_points) > 0:
            plt.scatter(
                faction_points[:, 0],
                faction_points[:, 1],
                c=color,
                label=faction.title(),
                alpha=0.6,
                s=50,
            )

    plt.legend()
    plt.title("Card2Vec Embeddings by Faction")
    plt.xlabel("Dimension 1")
    plt.ylabel("Dimension 2")

    if output_path:
        plt.savefig(output_path, dpi=150, bbox_inches="tight")
        print(f"Saved visualization to {output_path}")
    else:
        plt.show()

    plt.close()
