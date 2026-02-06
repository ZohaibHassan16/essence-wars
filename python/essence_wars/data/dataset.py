"""MCTS dataset for behavioral cloning training.

This module provides PyTorch Dataset classes for loading pre-generated
MCTS vs MCTS game data for behavioral cloning (supervised learning).

The dataset format is JSONL with each line containing:
- game_id: Unique game identifier
- deck1, deck2: Deck IDs used
- winner: 0, 1, or -1 (draw)
- moves: List of move records
- metadata: Generation metadata

Each move record contains:
- turn: Turn number
- player: Player who made the move (0 or 1)
- state_tensor: 326 floats
- action_mask: 256 bools (as 0/1)
- action: Action index taken (0-255)
- mcts_policy: 256 floats (visit count distribution)
- mcts_value: Float (MCTS value estimate)
"""

from __future__ import annotations

import gzip
import json
import random
from dataclasses import dataclass
from pathlib import Path
from typing import IO, TYPE_CHECKING, Any

import numpy as np
import torch
from numpy.typing import NDArray
from torch.utils.data import DataLoader, Dataset, IterableDataset

if TYPE_CHECKING:
    from collections.abc import Callable, Iterator


@dataclass
class MCTSSample:
    """A single training sample from MCTS data."""

    state_tensor: NDArray[np.float32]  # (326,)
    action_mask: NDArray[np.float32]   # (256,) bool
    action: int               # 0-255
    mcts_policy: NDArray[np.float32]   # (256,) visit count distribution
    value_target: float       # +1 if player won, -1 if lost, 0 if draw

    def to_tensors(self) -> dict[str, torch.Tensor]:
        """Convert to PyTorch tensors."""
        return {
            "obs": torch.from_numpy(self.state_tensor).float(),
            "mask": torch.from_numpy(self.action_mask).float(),
            "action": torch.tensor(self.action, dtype=torch.long),
            "policy_target": torch.from_numpy(self.mcts_policy).float(),
            "value_target": torch.tensor(self.value_target, dtype=torch.float32),
        }


class MCTSDataset(Dataset[dict[str, torch.Tensor]]):  # type: ignore[misc]
    """PyTorch Dataset for MCTS self-play data.

    Loads pre-generated MCTS vs MCTS games for behavioral cloning.
    Each sample is a single move with:
    - obs: State tensor (326,)
    - mask: Action mask (256,)
    - action: Action taken (scalar)
    - policy_target: MCTS policy (256,) - visit count distribution
    - value_target: Game outcome from this player's perspective (+1/-1/0)

    Example:
        >>> dataset = MCTSDataset("data/datasets/mcts_100k.jsonl")
        >>> loader = DataLoader(dataset, batch_size=256, shuffle=True)
        >>> for batch in loader:
        ...     obs = batch["obs"]  # (256, 326)
        ...     policy_target = batch["policy_target"]  # (256, 256)
    """

    def __init__(
        self,
        path: str | Path,
        *,
        max_games: int | None = None,
        player_perspective: bool = True,
        normalize: bool = False,
        use_mcts_value: bool = False,
    ) -> None:
        """Initialize MCTS dataset.

        Args:
            path: Path to JSONL or JSONL.gz file
            max_games: Maximum number of games to load (None = all)
            player_perspective: If True, flip value targets based on player
            normalize: If True, normalize state tensors (subtract mean, divide std)
            use_mcts_value: If True, use MCTS value estimates instead of game outcome.
                           This gives nuanced position evaluations (better for MCTS search)
                           rather than binary win/loss predictions.
        """
        self.path = Path(path)
        self.player_perspective = player_perspective
        self.normalize = normalize
        self.use_mcts_value = use_mcts_value

        # Load all samples
        self.samples: list[MCTSSample] = []
        self._load_data(max_games)

        # Compute normalization stats if needed
        self.mean: NDArray[np.float32] | None = None
        self.std: NDArray[np.float32] | None = None
        if self.normalize and len(self.samples) > 0:
            self._compute_normalization_stats()

    def _load_data(self, max_games: int | None) -> None:
        """Load all samples from JSONL file."""
        games_loaded = 0

        # Handle gzipped files
        def open_gzip(p: Path) -> IO[str]:
            return gzip.open(p, "rt", encoding="utf-8")

        def open_regular(p: Path) -> IO[str]:
            return p.open(encoding="utf-8")

        open_fn: Callable[[Path], IO[str]] = open_gzip if self.path.suffix == ".gz" else open_regular

        with open_fn(self.path) as f:
            for line in f:
                if max_games is not None and games_loaded >= max_games:
                    break

                game = json.loads(line)
                self._process_game(game)
                games_loaded += 1

        print(f"Loaded {games_loaded} games, {len(self.samples)} samples")

    def _process_game(self, game: dict[str, Any]) -> None:
        """Process a game record and extract samples."""
        winner = game["winner"]

        for move in game["moves"]:
            player = move["player"]

            if self.use_mcts_value:
                # Use MCTS value estimate (nuanced, good for MCTS search)
                # MCTS value is in [0, 1] (win probability), convert to [-1, 1]
                mcts_value = move.get("mcts_value", 0.5)
                value_target = 2.0 * mcts_value - 1.0
            else:
                # Use game outcome (binary, simpler but less nuanced)
                if winner == -1:
                    value_target = 0.0  # Draw
                elif self.player_perspective:
                    value_target = 1.0 if winner == player else -1.0
                else:
                    value_target = 1.0 if winner == 0 else -1.0

            sample = MCTSSample(
                state_tensor=np.array(move["state_tensor"], dtype=np.float32),
                action_mask=np.array(move["action_mask"], dtype=np.float32),
                action=move["action"],
                mcts_policy=np.array(move["mcts_policy"], dtype=np.float32),
                value_target=value_target,
            )
            self.samples.append(sample)

    def _compute_normalization_stats(self) -> None:
        """Compute mean and std for normalization."""
        all_obs = np.stack([s.state_tensor for s in self.samples])
        self.mean = all_obs.mean(axis=0)
        std = all_obs.std(axis=0)
        # Avoid division by zero
        std[std < 1e-6] = 1.0
        self.std = std

    def __len__(self) -> int:
        return len(self.samples)

    def __getitem__(self, idx: int) -> dict[str, torch.Tensor]:
        sample = self.samples[idx]
        tensors = sample.to_tensors()

        if self.normalize and self.mean is not None and self.std is not None:
            tensors["obs"] = (tensors["obs"] - torch.from_numpy(self.mean)) / torch.from_numpy(self.std)

        return tensors


class StreamingMCTSDataset:
    """Memory-efficient streaming dataset for very large files.

    Instead of loading all samples into memory, this iterates through
    the file and yields samples on demand. Useful for datasets that
    don't fit in memory.

    Note: This is NOT a PyTorch Dataset - use MCTSDataset for DataLoader
    compatibility. This is for manual iteration or pre-processing.
    """

    def __init__(
        self,
        path: str | Path,
        player_perspective: bool = True,
    ) -> None:
        self.path = Path(path)
        self.player_perspective = player_perspective

    def __iter__(self) -> Iterator[MCTSSample]:
        """Iterate through all samples in the dataset."""
        # Handle gzipped files
        def open_gzip(p: Path) -> IO[str]:
            return gzip.open(p, "rt", encoding="utf-8")

        def open_regular(p: Path) -> IO[str]:
            return p.open(encoding="utf-8")

        open_fn: Callable[[Path], IO[str]] = open_gzip if self.path.suffix == ".gz" else open_regular

        with open_fn(self.path) as f:
            for line in f:
                game = json.loads(line)
                yield from self._process_game(game)

    def _process_game(self, game: dict[str, Any]) -> Iterator[MCTSSample]:
        """Process a game record and yield samples."""
        winner = game["winner"]

        for move in game["moves"]:
            player = move["player"]

            if winner == -1:
                value_target = 0.0
            elif self.player_perspective:
                value_target = 1.0 if winner == player else -1.0
            else:
                value_target = 1.0 if winner == 0 else -1.0

            yield MCTSSample(
                state_tensor=np.array(move["state_tensor"], dtype=np.float32),
                action_mask=np.array(move["action_mask"], dtype=np.float32),
                action=move["action"],
                mcts_policy=np.array(move["mcts_policy"], dtype=np.float32),
                value_target=value_target,
            )


class ChunkedMCTSDataset(IterableDataset[dict[str, torch.Tensor]]):
    """Memory-efficient chunked dataset for large MCTS files.

    Streams data in fixed-size chunks with in-chunk shuffling.
    Memory is bounded to chunk_size x ~3.5 KB per sample.

    This is the recommended loader for large datasets (10k+ games)
    as it avoids loading everything into memory at once.

    Features:
    - Bounded memory usage (configurable via chunk_size)
    - In-chunk shuffling for training randomness
    - Multi-worker safe (workers process different chunks)
    - Compatible with standard PyTorch DataLoader

    Example:
        >>> dataset = ChunkedMCTSDataset(
        ...     "data/datasets/mcts_100k.jsonl.gz",
        ...     chunk_size=50_000,  # ~175 MB max memory
        ...     shuffle=True,
        ... )
        >>> loader = DataLoader(dataset, batch_size=512, num_workers=4)
        >>> for batch in loader:
        ...     train_step(batch)

    Memory estimates by chunk_size:
        - 10,000 samples: ~35 MB
        - 50,000 samples: ~175 MB
        - 100,000 samples: ~350 MB
    """

    def __init__(
        self,
        path: str | Path,
        *,
        chunk_size: int = 50_000,
        shuffle: bool = True,
        player_perspective: bool = True,
        seed: int | None = None,
    ) -> None:
        """Initialize chunked MCTS dataset.

        Args:
            path: Path to JSONL or JSONL.gz file
            chunk_size: Maximum samples to hold in memory at once
            shuffle: Whether to shuffle within each chunk
            player_perspective: If True, flip value targets based on player
            seed: Random seed for shuffling (None = random each epoch)
        """
        self.path = Path(path)
        self.chunk_size = chunk_size
        self.shuffle = shuffle
        self.player_perspective = player_perspective
        self.seed = seed

    def __iter__(self) -> Iterator[dict[str, torch.Tensor]]:
        """Iterate through dataset in chunks."""
        # Get worker info for multi-worker DataLoader
        worker_info = torch.utils.data.get_worker_info()
        worker_id = worker_info.id if worker_info else 0
        num_workers = worker_info.num_workers if worker_info else 1

        # Set random seed for reproducibility
        if self.seed is not None:
            random.seed(self.seed + worker_id)

        # Open file
        def open_gzip(p: Path) -> IO[str]:
            return gzip.open(p, "rt", encoding="utf-8")

        def open_regular(p: Path) -> IO[str]:
            return p.open(encoding="utf-8")

        open_fn: Callable[[Path], IO[str]] = open_gzip if self.path.suffix == ".gz" else open_regular

        chunk: list[MCTSSample] = []
        game_idx = 0

        with open_fn(self.path) as f:
            for line in f:
                # Multi-worker: each worker handles different games
                if game_idx % num_workers != worker_id:
                    game_idx += 1
                    continue
                game_idx += 1

                game = json.loads(line)
                samples = list(self._process_game(game))
                chunk.extend(samples)

                # Yield chunk when full
                if len(chunk) >= self.chunk_size:
                    yield from self._yield_chunk(chunk)
                    chunk = []

        # Yield remaining samples
        if chunk:
            yield from self._yield_chunk(chunk)

    def _process_game(self, game: dict[str, Any]) -> Iterator[MCTSSample]:
        """Process a game record and yield samples."""
        winner = game["winner"]

        for move in game["moves"]:
            player = move["player"]

            if winner == -1:
                value_target = 0.0
            elif self.player_perspective:
                value_target = 1.0 if winner == player else -1.0
            else:
                value_target = 1.0 if winner == 0 else -1.0

            yield MCTSSample(
                state_tensor=np.array(move["state_tensor"], dtype=np.float32),
                action_mask=np.array(move["action_mask"], dtype=np.float32),
                action=move["action"],
                mcts_policy=np.array(move["mcts_policy"], dtype=np.float32),
                value_target=value_target,
            )

    def _yield_chunk(
        self, chunk: list[MCTSSample]
    ) -> Iterator[dict[str, torch.Tensor]]:
        """Shuffle and yield samples from a chunk."""
        if self.shuffle:
            random.shuffle(chunk)

        for sample in chunk:
            yield sample.to_tensors()


def load_chunked_mcts_dataset(
    path: str | Path,
    *,
    batch_size: int = 256,
    chunk_size: int = 50_000,
    shuffle: bool = True,
    num_workers: int = 4,
    seed: int | None = None,
) -> tuple[ChunkedMCTSDataset, DataLoader[dict[str, torch.Tensor]]]:
    """Load MCTS dataset with memory-efficient chunked streaming.

    Recommended for large datasets (10k+ games) to avoid OOM crashes.

    Args:
        path: Path to JSONL or JSONL.gz file
        batch_size: Batch size for DataLoader
        chunk_size: Samples per chunk (controls memory usage)
        shuffle: Whether to shuffle within chunks
        num_workers: Number of DataLoader workers
        seed: Random seed for reproducibility

    Returns:
        Tuple of (dataset, dataloader)

    Example:
        >>> dataset, loader = load_chunked_mcts_dataset(
        ...     "data/mcts_100k.jsonl.gz",
        ...     batch_size=512,
        ...     chunk_size=50_000,  # ~175 MB max
        ... )
        >>> for epoch in range(10):
        ...     for batch in loader:
        ...         train_step(batch)
    """
    dataset = ChunkedMCTSDataset(
        path,
        chunk_size=chunk_size,
        shuffle=shuffle,
        seed=seed,
    )
    loader = DataLoader(
        dataset,
        batch_size=batch_size,
        num_workers=num_workers,
        pin_memory=torch.cuda.is_available(),
        # Note: shuffle=False because IterableDataset handles its own shuffling
    )
    return dataset, loader


def load_mcts_dataset(
    path: str | Path,
    *,
    batch_size: int = 256,
    shuffle: bool = True,
    num_workers: int = 0,
    max_games: int | None = None,
    normalize: bool = False,
) -> tuple[MCTSDataset, DataLoader[dict[str, torch.Tensor]]]:
    """Convenience function to load MCTS dataset with DataLoader.

    Args:
        path: Path to JSONL or JSONL.gz file
        batch_size: Batch size for DataLoader
        shuffle: Whether to shuffle data
        num_workers: Number of worker processes
        max_games: Maximum games to load
        normalize: Whether to normalize observations

    Returns:
        Tuple of (dataset, dataloader)

    Example:
        >>> dataset, loader = load_mcts_dataset("data/mcts_100k.jsonl", batch_size=512)
        >>> for batch in loader:
        ...     train_step(batch)
    """
    dataset = MCTSDataset(path, max_games=max_games, normalize=normalize)
    loader = DataLoader(
        dataset,
        batch_size=batch_size,
        shuffle=shuffle,
        num_workers=num_workers,
        pin_memory=torch.cuda.is_available(),
    )
    return dataset, loader


def get_dataset_stats(path: str | Path, max_games: int | None = None) -> dict[str, Any]:
    """Get statistics about a dataset without loading all samples.

    Returns dictionary with:
    - total_games: Number of games
    - total_moves: Number of moves (samples)
    - avg_moves_per_game: Average moves per game
    - win_rate_p1: Player 1 win rate
    - unique_decks: Set of deck IDs used
    """
    path = Path(path)

    total_games = 0
    total_moves = 0
    p1_wins = 0
    p2_wins = 0
    draws = 0
    decks: set[str] = set()

    def open_gzip(p: Path) -> IO[str]:
        return gzip.open(p, "rt", encoding="utf-8")

    def open_regular(p: Path) -> IO[str]:
        return p.open(encoding="utf-8")

    open_fn: Callable[[Path], IO[str]] = open_gzip if path.suffix == ".gz" else open_regular

    with open_fn(path) as f:
        for line in f:
            if max_games is not None and total_games >= max_games:
                break

            game = json.loads(line)
            total_games += 1
            total_moves += len(game["moves"])

            # Handle both original MCTS format (deck1/deck2) and ExIt format (game_seed)
            if "deck1" in game:
                decks.add(game["deck1"])
            if "deck2" in game:
                decks.add(game["deck2"])
            if "game_seed" in game:
                decks.add(f"seed_{game['game_seed']}")

            if game["winner"] == 0:
                p1_wins += 1
            elif game["winner"] == 1:
                p2_wins += 1
            else:
                draws += 1

    return {
        "total_games": total_games,
        "total_moves": total_moves,
        "avg_moves_per_game": total_moves / total_games if total_games > 0 else 0,
        "win_rate_p1": p1_wins / total_games if total_games > 0 else 0,
        "win_rate_p2": p2_wins / total_games if total_games > 0 else 0,
        "draw_rate": draws / total_games if total_games > 0 else 0,
        "unique_decks": decks,
    }
