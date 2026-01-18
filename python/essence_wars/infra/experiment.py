"""Experiment management for reproducible ML/AI research."""

from __future__ import annotations

import csv
import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Any


class Experiment:
    """
    Manages experiment directory creation, configuration saving, and metrics logging.

    Usage:
        exp = Experiment("tuning", tag="mcts_baseline")
        exp.save_config({"lr": 3e-4, "steps": 1000})
        exp.log_metric("loss", 0.5, step=1)
        exp.log("Training started")
    """

    def __init__(
        self,
        category: str,
        tag: str = "default",
        root_dir: str | Path = "experiments",
    ) -> None:
        """Initialize experiment with timestamped directory.

        Args:
            category: Experiment category (tuning, training, eval)
            tag: Short descriptive tag for this run
            root_dir: Root directory for all experiments
        """
        self.timestamp = datetime.now().strftime("%Y-%m-%d_%H%M")
        self.id = f"{self.timestamp}_{tag}"
        self.category = category
        self.root_dir = Path(root_dir)

        # Create directory structure
        self.dir = self.root_dir / category / self.id
        self.dir.mkdir(parents=True, exist_ok=True)

        # Subdirectories
        (self.dir / "checkpoints").mkdir(exist_ok=True)
        (self.dir / "plots").mkdir(exist_ok=True)

        # Setup logging
        self._logger: logging.Logger | None = None
        self._metrics: list[dict[str, Any]] = []

    @property
    def logger(self) -> logging.Logger:
        """Lazily create logger on first access."""
        if self._logger is None:
            self._logger = logging.getLogger(f"exp.{self.id}")
            self._logger.setLevel(logging.INFO)

            # File handler
            log_file = self.dir / "run.log"
            fh = logging.FileHandler(log_file)
            fh.setFormatter(
                logging.Formatter("%(asctime)s - %(levelname)s - %(message)s")
            )
            self._logger.addHandler(fh)

            # Console handler (if not already present)
            if not any(isinstance(h, logging.StreamHandler) for h in self._logger.handlers):
                ch = logging.StreamHandler()
                ch.setFormatter(logging.Formatter("%(message)s"))
                self._logger.addHandler(ch)

        return self._logger

    def save_config(self, config: dict[str, Any]) -> Path:
        """Save configuration to JSON file.

        Args:
            config: Configuration dictionary

        Returns:
            Path to saved config file
        """
        config_path = self.dir / "config.json"
        with config_path.open("w") as f:
            json.dump(config, f, indent=2, default=str)
        self.log(f"Config saved: {config_path}")
        return config_path

    def log(self, msg: str) -> None:
        """Log a message to file and console."""
        self.logger.info(msg)

    def log_metric(
        self,
        name: str,
        value: float,
        step: int | None = None,
    ) -> None:
        """Log a single metric value.

        Args:
            name: Metric name
            value: Metric value
            step: Optional step/iteration number
        """
        entry: dict[str, Any] = {"name": name, "value": value}
        if step is not None:
            entry["step"] = step
        entry["timestamp"] = datetime.now().isoformat()
        self._metrics.append(entry)

    def save_metrics(self, filename: str = "metrics.csv") -> Path | None:
        """Save accumulated metrics to CSV file.

        Args:
            filename: Output filename

        Returns:
            Path to saved file, or None if no metrics
        """
        if not self._metrics:
            return None

        path = self.dir / filename
        with path.open("w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=self._metrics[0].keys())
            writer.writeheader()
            writer.writerows(self._metrics)

        return path

    def save_artifact(self, name: str, data: Any) -> Path:
        """Save arbitrary data as JSON artifact.

        Args:
            name: Artifact name (will be saved as {name}.json)
            data: Data to save (must be JSON serializable)

        Returns:
            Path to saved artifact
        """
        path = self.dir / f"{name}.json"
        with path.open("w") as f:
            json.dump(data, f, indent=2, default=str)
        return path

    def get_checkpoint_path(self, name: str) -> Path:
        """Get path for a checkpoint file.

        Args:
            name: Checkpoint filename

        Returns:
            Full path to checkpoint file
        """
        return self.dir / "checkpoints" / name

    def __repr__(self) -> str:
        return f"Experiment(id={self.id!r}, dir={self.dir})"
