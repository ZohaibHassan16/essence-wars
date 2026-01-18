"""Huggingface Hub integration for Essence Wars.

This module provides utilities for downloading and uploading models and datasets
from/to the Huggingface Hub, enabling easy sharing of trained agents and
pre-generated datasets.

Example - Load a pre-trained agent:
    from essence_wars.hub import load_pretrained

    agent = load_pretrained("essence-wars/ppo-generalist")

    # Use in benchmark
    from essence_wars.benchmark import EssenceWarsBenchmark
    benchmark = EssenceWarsBenchmark()
    results = benchmark.evaluate(agent)

Example - Download a dataset:
    from essence_wars.hub import download_dataset

    path = download_dataset("essence-wars/mcts-self-play")

    # Load with MCTSDataset
    from essence_wars.data import MCTSDataset
    dataset = MCTSDataset(path)

Example - Upload a model (requires authentication):
    from essence_wars.hub import upload_model

    upload_model(
        checkpoint_path="models/my_agent.pt",
        repo_id="username/my-essence-wars-agent",
        model_type="alphazero",
    )
"""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal

if TYPE_CHECKING:
    from essence_wars.benchmark.agents import NeuralAgent

# Default organization on Huggingface
HF_ORG = "essence-wars"

# Known model repositories
KNOWN_MODELS = {
    "ppo-generalist": f"{HF_ORG}/ppo-generalist",
    "bc-mcts-100k": f"{HF_ORG}/bc-mcts-100k",
    "alphazero-v1": f"{HF_ORG}/alphazero-v1",
    "mcts-baseline": f"{HF_ORG}/mcts-baseline",
}

# Known dataset repositories
KNOWN_DATASETS = {
    "mcts-self-play": f"{HF_ORG}/mcts-self-play",
    "mcts-100k": f"{HF_ORG}/mcts-100k",
    "balanced-matchups": f"{HF_ORG}/balanced-matchups",
}


def _ensure_huggingface_hub() -> None:
    """Check that huggingface_hub is installed."""
    try:
        import huggingface_hub  # noqa: F401
    except ImportError as e:
        raise ImportError(
            "huggingface_hub is required for Hub integration. "
            "Install with: pip install huggingface_hub"
        ) from e


def _resolve_repo_id(name: str, known_repos: dict[str, str]) -> str:
    """Resolve a short name to full repo ID.

    Args:
        name: Short name (e.g., "ppo-generalist") or full repo ID
        known_repos: Dict mapping short names to full repo IDs

    Returns:
        Full repo ID (e.g., "essence-wars/ppo-generalist")
    """
    if "/" in name:
        # Already a full repo ID
        return name

    if name in known_repos:
        return known_repos[name]

    # Assume it's under the default org
    return f"{HF_ORG}/{name}"


def load_pretrained(
    model_name: str,
    *,
    device: str = "cpu",
    deterministic: bool = True,
    cache_dir: str | Path | None = None,
    revision: str = "main",
    token: str | None = None,
) -> NeuralAgent:
    """Load a pre-trained agent from Huggingface Hub.

    Args:
        model_name: Model name or repo ID. Can be:
            - Short name: "ppo-generalist", "alphazero-v1"
            - Full repo ID: "essence-wars/ppo-generalist"
            - User repo: "username/my-model"
        device: Device for inference ("cpu", "cuda", "mps")
        deterministic: If True, use argmax for action selection
        cache_dir: Directory to cache downloaded files
        revision: Git revision (branch, tag, or commit)
        token: Huggingface token for private repos

    Returns:
        NeuralAgent ready for evaluation

    Example:
        agent = load_pretrained("ppo-generalist")
        results = benchmark.evaluate(agent)
    """
    _ensure_huggingface_hub()
    from huggingface_hub import hf_hub_download

    from essence_wars.benchmark.agents import NeuralAgent

    repo_id = _resolve_repo_id(model_name, KNOWN_MODELS)

    # Download checkpoint file
    checkpoint_path = hf_hub_download(
        repo_id=repo_id,
        filename="model.pt",
        cache_dir=cache_dir,
        revision=revision,
        token=token,
    )

    # Load using existing NeuralAgent.from_checkpoint
    return NeuralAgent.from_checkpoint(
        checkpoint_path=checkpoint_path,
        name=model_name.split("/")[-1],
        device=device,
        deterministic=deterministic,
    )


def download_dataset(
    dataset_name: str,
    *,
    filename: str = "data.jsonl.gz",
    cache_dir: str | Path | None = None,
    revision: str = "main",
    token: str | None = None,
) -> Path:
    """Download a dataset from Huggingface Hub.

    Args:
        dataset_name: Dataset name or repo ID. Can be:
            - Short name: "mcts-self-play", "balanced-matchups"
            - Full repo ID: "essence-wars/mcts-self-play"
        filename: Name of the dataset file to download
        cache_dir: Directory to cache downloaded files
        revision: Git revision (branch, tag, or commit)
        token: Huggingface token for private repos

    Returns:
        Path to the downloaded dataset file

    Example:
        path = download_dataset("mcts-self-play")
        dataset = MCTSDataset(path)
    """
    _ensure_huggingface_hub()
    from huggingface_hub import hf_hub_download

    repo_id = _resolve_repo_id(dataset_name, KNOWN_DATASETS)

    # Download dataset file
    local_path = hf_hub_download(
        repo_id=repo_id,
        filename=filename,
        cache_dir=cache_dir,
        revision=revision,
        token=token,
        repo_type="dataset",
    )

    return Path(local_path)


def upload_model(
    checkpoint_path: str | Path,
    repo_id: str,
    *,
    model_type: Literal["ppo", "alphazero", "bc"] = "alphazero",
    training_config: dict[str, Any] | None = None,
    metrics: dict[str, Any] | None = None,
    private: bool = False,
    token: str | None = None,
    commit_message: str | None = None,
) -> str:
    """Upload a trained model to Huggingface Hub.

    Creates a model repository with:
    - model.pt: The checkpoint file
    - config.json: Training configuration
    - README.md: Auto-generated model card

    Args:
        checkpoint_path: Path to the .pt checkpoint file
        repo_id: Target repository (e.g., "username/my-model")
        model_type: Type of model ("ppo", "alphazero", "bc")
        training_config: Training hyperparameters to include
        metrics: Evaluation metrics (win rates, etc.)
        private: Whether the repo should be private
        token: Huggingface token (or set HF_TOKEN env var)
        commit_message: Custom commit message

    Returns:
        URL of the uploaded model

    Example:
        url = upload_model(
            "models/alphazero_v1.pt",
            "username/essence-wars-alphazero",
            model_type="alphazero",
            metrics={"win_rate_vs_greedy": 0.72},
        )
    """
    _ensure_huggingface_hub()
    from huggingface_hub import HfApi, create_repo

    checkpoint_path = Path(checkpoint_path)
    if not checkpoint_path.exists():
        raise FileNotFoundError(f"Checkpoint not found: {checkpoint_path}")

    api = HfApi(token=token)

    # Create repo if it doesn't exist
    try:
        create_repo(repo_id, private=private, token=token, exist_ok=True)
    except Exception:
        pass  # Repo might already exist

    # Upload checkpoint
    api.upload_file(
        path_or_fileobj=str(checkpoint_path),
        path_in_repo="model.pt",
        repo_id=repo_id,
        commit_message=commit_message or f"Upload {model_type} model",
    )

    # Upload config if provided
    if training_config:
        config_content = json.dumps(training_config, indent=2)
        api.upload_file(
            path_or_fileobj=config_content.encode(),
            path_in_repo="config.json",
            repo_id=repo_id,
            commit_message="Add training config",
        )

    # Generate and upload model card
    model_card = _generate_model_card(
        repo_id=repo_id,
        model_type=model_type,
        training_config=training_config,
        metrics=metrics,
    )
    api.upload_file(
        path_or_fileobj=model_card.encode(),
        path_in_repo="README.md",
        repo_id=repo_id,
        commit_message="Add model card",
    )

    return f"https://huggingface.co/{repo_id}"


def upload_dataset(
    dataset_path: str | Path,
    repo_id: str,
    *,
    filename: str = "data.jsonl.gz",
    metadata: dict[str, Any] | None = None,
    private: bool = False,
    token: str | None = None,
    commit_message: str | None = None,
) -> str:
    """Upload a dataset to Huggingface Hub.

    Args:
        dataset_path: Path to the dataset file (.jsonl or .jsonl.gz)
        repo_id: Target repository (e.g., "username/my-dataset")
        filename: Name for the file in the repo
        metadata: Dataset metadata (num_games, bot_types, etc.)
        private: Whether the repo should be private
        token: Huggingface token
        commit_message: Custom commit message

    Returns:
        URL of the uploaded dataset
    """
    _ensure_huggingface_hub()
    from huggingface_hub import HfApi, create_repo

    dataset_path = Path(dataset_path)
    if not dataset_path.exists():
        raise FileNotFoundError(f"Dataset not found: {dataset_path}")

    api = HfApi(token=token)

    # Create dataset repo
    try:
        create_repo(
            repo_id,
            private=private,
            token=token,
            exist_ok=True,
            repo_type="dataset",
        )
    except Exception:
        pass

    # Upload dataset file
    api.upload_file(
        path_or_fileobj=str(dataset_path),
        path_in_repo=filename,
        repo_id=repo_id,
        repo_type="dataset",
        commit_message=commit_message or "Upload dataset",
    )

    # Upload metadata if provided
    if metadata:
        metadata_content = json.dumps(metadata, indent=2)
        api.upload_file(
            path_or_fileobj=metadata_content.encode(),
            path_in_repo="metadata.json",
            repo_id=repo_id,
            repo_type="dataset",
            commit_message="Add metadata",
        )

    # Generate and upload dataset card
    dataset_card = _generate_dataset_card(
        repo_id=repo_id,
        metadata=metadata,
    )
    api.upload_file(
        path_or_fileobj=dataset_card.encode(),
        path_in_repo="README.md",
        repo_id=repo_id,
        repo_type="dataset",
        commit_message="Add dataset card",
    )

    return f"https://huggingface.co/datasets/{repo_id}"


def _generate_model_card(
    repo_id: str,
    model_type: str,
    training_config: dict[str, Any] | None = None,
    metrics: dict[str, Any] | None = None,
) -> str:
    """Generate a model card README for Huggingface."""
    model_name = repo_id.split("/")[-1]

    card = f"""---
license: mit
tags:
- reinforcement-learning
- card-game
- essence-wars
- {model_type}
library_name: essence-wars
---

# {model_name}

A {model_type.upper()} agent trained on Essence Wars, a high-performance card game
environment for reinforcement learning research.

## Usage

```python
from essence_wars.hub import load_pretrained
from essence_wars.benchmark import EssenceWarsBenchmark

# Load the agent
agent = load_pretrained("{repo_id}")

# Evaluate against baselines
benchmark = EssenceWarsBenchmark()
results = benchmark.evaluate(agent)
print(results.summary())
```

## Model Details

- **Model Type**: {model_type.upper()}
- **Framework**: PyTorch
- **Environment**: Essence Wars v0.6.0
"""

    if training_config:
        card += "\n### Training Configuration\n\n```json\n"
        card += json.dumps(training_config, indent=2)
        card += "\n```\n"

    if metrics:
        card += "\n## Evaluation Results\n\n"
        card += "| Metric | Value |\n|--------|-------|\n"
        for key, value in metrics.items():
            if isinstance(value, float):
                card += f"| {key} | {value:.2%} |\n"
            else:
                card += f"| {key} | {value} |\n"

    card += """
## Citation

If you use this model in your research, please cite:

```bibtex
@software{essence_wars,
  title={Essence Wars: A Card Game Environment for RL Research},
  author={Wissmann, Christian},
  year={2025},
  url={https://github.com/christianwissmann85/ai-cardgame}
}
```
"""
    return card


def _generate_dataset_card(
    repo_id: str,
    metadata: dict[str, Any] | None = None,
) -> str:
    """Generate a dataset card README for Huggingface."""
    dataset_name = repo_id.split("/")[-1]

    card = f"""---
license: mit
tags:
- reinforcement-learning
- card-game
- essence-wars
- mcts
task_categories:
- reinforcement-learning
---

# {dataset_name}

A dataset of Essence Wars games generated by MCTS self-play, suitable for
behavioral cloning and offline reinforcement learning research.

## Usage

```python
from essence_wars.hub import download_dataset
from essence_wars.data import MCTSDataset

# Download the dataset
path = download_dataset("{repo_id}")

# Load for training
dataset = MCTSDataset(path)
print(f"Loaded {{len(dataset)}} samples")
```

## Dataset Structure

Each game record contains:
- `game_id`: Unique identifier
- `deck1`, `deck2`: Deck configurations used
- `winner`: 0 (player 1), 1 (player 2), or -1 (draw)
- `moves`: List of move records with:
  - `state_tensor`: 326-dim observation
  - `action_mask`: 256-dim legal action mask
  - `action`: Action taken (0-255)
  - `mcts_policy`: MCTS visit count distribution
  - `mcts_value`: MCTS value estimate
"""

    if metadata:
        card += "\n## Dataset Statistics\n\n"
        card += "| Statistic | Value |\n|-----------|-------|\n"
        for key, value in metadata.items():
            card += f"| {key} | {value} |\n"

    card += """
## License

MIT License - see the repository for details.
"""
    return card


def list_available_models() -> dict[str, str]:
    """List known pre-trained models.

    Returns:
        Dict mapping short names to full repo IDs
    """
    return KNOWN_MODELS.copy()


def list_available_datasets() -> dict[str, str]:
    """List known datasets.

    Returns:
        Dict mapping short names to full repo IDs
    """
    return KNOWN_DATASETS.copy()


# Convenience function for getting cache directory
def get_cache_dir() -> Path:
    """Get the default cache directory for downloaded files.

    Uses HF_HOME environment variable if set, otherwise ~/.cache/huggingface.
    """
    hf_home = os.environ.get("HF_HOME")
    if hf_home:
        return Path(hf_home)
    return Path.home() / ".cache" / "huggingface"
