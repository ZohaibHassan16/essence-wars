# Learned Card Embeddings Design

> **Status**: Design Phase
> **Author**: Christian Wissmann + Claude
> **Created**: January 2026

## Overview

This document describes the design for learned card embeddings in Essence Wars, comparing flat (one-hot/raw ID) vs embedded observation modes.

**Research Question**: Does replacing raw card IDs with learned embedding vectors improve:
1. Training sample efficiency?
2. Generalization to unseen deck matchups?
3. Transfer to new cards (future expansions)?

---

## Current State Tensor Structure

The current 326-float state tensor contains raw card IDs as floats:

```
Index   | Content                    | Size
--------|----------------------------|------
0-5     | Global state               | 6
6-80    | Player 1 state             | 75
  6-10  |   Base stats               | 5
  11-20 |   Hand card IDs (raw)      | 10
  21-70 |   Creature slots           | 50
  71-80 |   Support slots            | 10
81-155  | Player 2 state             | 75
  81-85 |   Base stats               | 5
  86-95 |   Hand card IDs (raw)      | 10
  96-145|   Creature slots           | 50
  146-155|  Support slots            | 10
156-325 | Card embedding IDs section | 170
```

### Card ID Positions

| Location | Tensor Indices | Description |
|----------|----------------|-------------|
| P1 Hand | 11-20 | 10 hand card IDs |
| P1 Support | 73, 78 (inside slots) | Support card IDs at offset +2 within each 5-float slot |
| P2 Hand | 86-95 | 10 hand card IDs |
| P2 Support | 148, 153 (inside slots) | Support card IDs at offset +2 within each 5-float slot |
| Embedding Section | 156-325 | Additional card IDs for context |

**Note**: Creature slots (10 floats each) do NOT store card_id directly - they store stats/keywords. Creature identity must be inferred from stats.

### Card ID Ranges

| Faction | ID Range |
|---------|----------|
| Argentum | 1000-1074 |
| Symbiote | 2000-2074 |
| Obsidion | 3000-3074 |
| Neutral | 4000-4074 |

**Total cards**: 300 (75 per faction × 4)

---

## Problem with Raw Card IDs

Raw card IDs (stored as floats) have several issues:

1. **Arbitrary Ordering**: Card 1000 and 1001 are numerically close but may be completely different (e.g., 2/4 Guard vs 4/2 Rush)

2. **Sparse Input**: With 300 cards, the network must learn 300 separate mappings from arbitrary integers to card properties

3. **No Semantic Similarity**: Similar cards (e.g., all Rush creatures) have no representation proximity

4. **Poor Generalization**: A new card (ID 1075) would be completely unknown to the network

---

## Embedding Architecture

### Approach 1: End-to-End Learning

Learn embeddings jointly with the policy network during RL training.

```python
class EmbeddedNetwork(nn.Module):
    def __init__(self, embed_dim=64, num_cards=5000, hidden_dim=256):
        super().__init__()

        # Card embedding table
        # num_cards=5000 to handle ID ranges (1000-4074)
        self.card_embedding = nn.Embedding(num_cards, embed_dim, padding_idx=0)

        # Non-card features: 326 - (card_id_positions) = ~106 floats
        # After embedding: 106 + (num_card_slots × embed_dim)

        self.trunk = nn.Sequential(
            nn.Linear(computed_input_dim, hidden_dim),
            nn.ReLU(),
            ...
        )
```

**Pros**:
- Simple implementation
- Embeddings optimized for the RL task
- Single training process

**Cons**:
- Embeddings may overfit to training distribution
- No knowledge transfer between training runs
- May not generalize to unseen cards

### Approach 2: Pre-trained + Fine-tune (Card2Vec)

Pre-train card embeddings separately, then fine-tune during RL.

**Phase 1: Pre-train Card2Vec**

Train embeddings on game co-occurrence data:
- Cards that appear in winning decks together → closer embeddings
- Cards with similar stats/keywords → closer embeddings
- Similar to Word2Vec skip-gram model

```python
class Card2Vec(nn.Module):
    """Pre-train card embeddings on deck/game data."""

    def __init__(self, num_cards=5000, embed_dim=64):
        super().__init__()
        self.embeddings = nn.Embedding(num_cards, embed_dim)
        self.output = nn.Linear(embed_dim, num_cards)

    def forward(self, center_card_id, context_card_ids):
        # Skip-gram: predict context cards from center card
        embed = self.embeddings(center_card_id)
        logits = self.output(embed)
        return logits
```

**Training data sources**:
1. **Deck composition**: Cards that appear together in decks
2. **Game states**: Cards played in same turn/game
3. **MCTS self-play data**: Cards in winning vs losing games

**Phase 2: Fine-tune in RL**

```python
class EmbeddedNetworkFineTune(nn.Module):
    def __init__(self, pretrained_embeds, hidden_dim=256):
        super().__init__()

        # Load pre-trained embeddings
        self.card_embedding = nn.Embedding.from_pretrained(
            pretrained_embeds,
            freeze=False,  # Allow fine-tuning
        )
        ...
```

**Pros**:
- Embeddings capture card semantics before RL
- Better generalization to new cards (if pre-trained on full card set)
- Can share embeddings across experiments

**Cons**:
- Two-phase training
- Pre-training requires additional data pipeline
- May lose task-specific specialization

---

## Implementation Plan

### Phase A: Observation Transformation (Python-side)

Keep the Rust tensor as-is. Transform in Python before feeding to network.

```python
class ObservationTransformer:
    """Transform flat observation to embedded observation."""

    # Card ID positions in the 326-float tensor
    CARD_ID_POSITIONS = {
        'p1_hand': list(range(11, 21)),        # indices 11-20
        'p1_support_0': [73],                   # offset +2 in 5-float slot
        'p1_support_1': [78],
        'p2_hand': list(range(86, 96)),        # indices 86-95
        'p2_support_0': [148],
        'p2_support_1': [153],
        'embedding_section': list(range(156, 326)),  # trailing section
    }

    # Non-card feature positions
    NON_CARD_POSITIONS = [...compute all other indices...]

    def __init__(self, card_embedding: nn.Embedding, max_card_id=5000):
        self.card_embedding = card_embedding
        self.max_card_id = max_card_id

    def transform(self, obs: torch.Tensor) -> torch.Tensor:
        """
        Transform flat observation to embedded observation.

        Args:
            obs: Shape (batch, 326) or (326,)

        Returns:
            embedded_obs: Shape (batch, new_dim) or (new_dim,)
        """
        batch = obs.dim() == 2
        if not batch:
            obs = obs.unsqueeze(0)

        # Extract non-card features
        non_card_features = obs[:, self.NON_CARD_POSITIONS]

        # Extract and embed card IDs
        card_embeds = []
        for name, positions in self.CARD_ID_POSITIONS.items():
            card_ids = obs[:, positions].long().clamp(0, self.max_card_id - 1)
            embed = self.card_embedding(card_ids)  # (batch, num_cards, embed_dim)
            card_embeds.append(embed.flatten(start_dim=1))  # (batch, num_cards * embed_dim)

        # Concatenate all features
        embedded_obs = torch.cat([non_card_features] + card_embeds, dim=1)

        if not batch:
            embedded_obs = embedded_obs.squeeze(0)

        return embedded_obs
```

### Phase B: Network Architecture with Embeddings

```python
class EmbeddedEssenceWarsNetwork(nn.Module):
    """
    Network with learned card embeddings.

    Observation structure after transformation:
    - Non-card features: ~106 floats (stats, flags, etc.)
    - Embedded card IDs: num_card_slots × embed_dim

    Total input dim depends on embed_dim and which card positions we embed.
    """

    def __init__(
        self,
        embed_dim: int = 64,
        num_cards: int = 5000,  # Max card ID + buffer
        hidden_dim: int = 256,
        num_blocks: int = 4,
        pretrained_embeds: torch.Tensor | None = None,
    ):
        super().__init__()

        # Card embedding layer
        if pretrained_embeds is not None:
            self.card_embedding = nn.Embedding.from_pretrained(
                pretrained_embeds, freeze=False, padding_idx=0
            )
            embed_dim = pretrained_embeds.shape[1]
        else:
            self.card_embedding = nn.Embedding(num_cards, embed_dim, padding_idx=0)

        self.embed_dim = embed_dim
        self.transformer = ObservationTransformer(self.card_embedding)

        # Calculate input dimension
        # Non-card features + (embedded card slots)
        num_card_slots = len(self._get_all_card_positions())
        input_dim = self._non_card_dim() + num_card_slots * embed_dim

        # Input projection
        self.input_proj = nn.Sequential(
            nn.Linear(input_dim, hidden_dim),
            nn.ReLU(),
        )

        # Residual tower (same as AlphaZeroNetwork)
        self.residual_tower = nn.ModuleList([
            ResidualBlock(hidden_dim) for _ in range(num_blocks)
        ])

        # Heads (same as AlphaZeroNetwork)
        self.policy_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 256),
        )

        self.value_head = nn.Sequential(
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
            nn.Tanh(),
        )

    def forward(self, obs: torch.Tensor, action_mask: torch.Tensor | None = None):
        # Transform observation to embed card IDs
        embedded_obs = self.transformer.transform(obs)

        # Standard forward pass
        x = self.input_proj(embedded_obs)
        for block in self.residual_tower:
            x = block(x)

        logits = self.policy_head(x)
        value = self.value_head(x).squeeze(-1)

        if action_mask is not None:
            logits = logits.masked_fill(~action_mask, -1e8)

        return logits, value
```

### Phase C: Environment Integration

Add `observation_mode` parameter to the environment:

```python
class EssenceWarsEnv(gym.Env):
    def __init__(
        self,
        ...,
        observation_mode: str = "flat",  # "flat" or "embedded"
        embed_dim: int = 64,
    ):
        self.observation_mode = observation_mode

        if observation_mode == "embedded":
            # Observation space changes when using embeddings
            # This is handled by the network, so we keep raw obs space
            # and let the network transform it
            pass
```

**Alternative**: Keep environment unchanged, handle embedding in the network (cleaner separation).

---

## Experiment Design

### Experiment 1: Sample Efficiency Comparison

**Setup**:
- Train PPO-Generalist with `observation_mode="flat"`
- Train PPO-Generalist with `observation_mode="embedded"` (end-to-end)
- Train PPO-Generalist with `observation_mode="embedded"` (pre-trained Card2Vec)

**Metrics**:
- Win rate vs MCTS-100 at training step milestones (100k, 500k, 1M, 5M, 10M)
- Final win rate
- Training wall time

**Hypothesis**: Embedded approaches reach 50% win rate faster due to better feature representation.

### Experiment 2: Generalization to Unseen Matchups

**Setup**:
- Train on 80% of deck matchups (random subset)
- Evaluate on held-out 20% of matchups

**Metrics**:
- Win rate on training matchups
- Win rate on held-out matchups
- Generalization gap (training - held-out)

**Hypothesis**: Embeddings reduce the generalization gap by learning card semantics rather than memorizing matchup-specific patterns.

### Experiment 3: Transfer to New Cards (Simulated)

**Setup**:
- Train on 3 factions (225 cards)
- Evaluate on 4th faction (75 unseen cards)

**Metrics**:
- Win rate against seen factions
- Win rate against unseen faction
- Embedding similarity analysis (do similar cards cluster?)

**Hypothesis**: Pre-trained embeddings show better transfer because they capture card semantics from deck composition data.

### Experiment 4: Embedding Dimension Ablation

**Setup**:
- Train with embed_dim = 16, 32, 64, 128, 256

**Metrics**:
- Final win rate
- Training speed (steps/sec)
- Memory usage

**Hypothesis**: 64-dimensional embeddings provide best balance of expressiveness and efficiency.

---

## Pre-training Data Pipeline (Card2Vec)

### Data Source 1: Deck Composition

Use the 12 commander decks + generate random decks following faction rules.

```python
def generate_deck_pairs(deck):
    """Generate (center, context) pairs from a deck."""
    cards = deck.cards
    for i, center in enumerate(cards):
        for j, context in enumerate(cards):
            if i != j:
                yield (center.card_id, context.card_id)
```

### Data Source 2: MCTS Game Data

Use the 100k MCTS dataset to extract co-occurrence patterns.

```python
def extract_game_pairs(game_record):
    """Extract (center, context) pairs from game states."""
    for state in game_record.states:
        # Cards in same hand are context for each other
        for hand_card in state.p1_hand:
            for other_card in state.p1_hand:
                if hand_card != other_card:
                    yield (hand_card, other_card)

        # Cards on board are context for each other
        for board_card in state.p1_board:
            for other_board in state.p1_board:
                if board_card != other_board:
                    yield (board_card, other_board)
```

### Data Source 3: Card Attributes

Encode card attributes (stats, keywords, effects) as supervised signal.

```python
class CardAttributePredictor(nn.Module):
    """Predict card attributes from embedding."""

    def __init__(self, embed_dim, num_keywords=14):
        super().__init__()
        self.embed = nn.Embedding(5000, embed_dim)
        self.cost_head = nn.Linear(embed_dim, 1)
        self.attack_head = nn.Linear(embed_dim, 1)
        self.health_head = nn.Linear(embed_dim, 1)
        self.keyword_head = nn.Linear(embed_dim, num_keywords)

    def forward(self, card_ids):
        embed = self.embed(card_ids)
        return {
            'cost': self.cost_head(embed),
            'attack': self.attack_head(embed),
            'health': self.health_head(embed),
            'keywords': torch.sigmoid(self.keyword_head(embed)),
        }
```

Training this auxiliary task ensures embeddings encode card semantics.

---

## Implementation Milestones

### Milestone 1: Basic Infrastructure
- [ ] Create `ObservationTransformer` class
- [ ] Create `EmbeddedEssenceWarsNetwork` class
- [ ] Add unit tests for observation transformation
- [ ] Verify dimension calculations

### Milestone 2: End-to-End Training
- [ ] Integrate with `train_ppo.py`
- [ ] Add `--observation-mode` flag
- [ ] Run baseline comparison (flat vs embedded)
- [ ] Document initial results

### Milestone 3: Card2Vec Pre-training
- [ ] Create `Card2Vec` training script
- [ ] Generate pre-training data from decks + MCTS games
- [ ] Train Card2Vec embeddings
- [ ] Visualize embeddings (t-SNE/UMAP)

### Milestone 4: Pre-trained + Fine-tune
- [ ] Load pre-trained embeddings in `EmbeddedEssenceWarsNetwork`
- [ ] Run comparison with end-to-end approach
- [ ] Document results

### Milestone 5: Full Comparison Study
- [ ] Run all experiments (sample efficiency, generalization, transfer)
- [ ] Generate figures and tables
- [ ] Write results section for Paper 1

---

## File Locations

| File | Purpose |
|------|---------|
| `python/essence_wars/agents/embeddings.py` | ObservationTransformer, EmbeddedNetwork |
| `python/essence_wars/agents/card2vec.py` | Card2Vec pre-training |
| `python/scripts/train_card2vec.py` | Pre-training script |
| `python/scripts/train_ppo.py` | Add --observation-mode flag |
| `docs/embedding-design.md` | This document |

---

## Open Questions

1. **Creature Card IDs**: Currently creature slots don't store card_id directly. Should we add creature card IDs to the tensor, or infer identity from stats/keywords?

2. **Embedding Section**: The trailing 170-float section contains more card IDs. Should we embed all of them, or focus on hand/board only?

3. **Position Encoding**: Should we add positional encoding for card slots (e.g., hand position matters)?

4. **Frozen vs Fine-tuned**: For pre-trained embeddings, what's the best approach - fully frozen, partially frozen (first N epochs), or always trainable?

---

## References

- Mikolov et al., "Distributed Representations of Words and Phrases" (Word2Vec)
- Makarov et al., "Learning Embeddings for Card Games" (if exists)
- OpenAI, "Mastering the Game of Go with Deep Neural Networks" (AlphaGo embedding design)
