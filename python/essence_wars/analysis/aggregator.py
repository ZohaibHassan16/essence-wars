#!/usr/bin/env python3
"""
Aggregator for MCTS training experiments.
Scans experiments/mcts/ directory and builds consolidated datasets.
"""

import hashlib
import logging
import pickle
import re
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path

import pandas as pd

logger = logging.getLogger(__name__)


@dataclass
class GenerationMetrics:
    """Metrics for a single generation."""

    generation: int
    best_fitness: float
    best_winrate: float
    sigma: float
    time_seconds: float


@dataclass
class ExperimentMetadata:
    """Metadata for a complete experiment run."""

    experiment_id: str
    timestamp: str
    tag: str
    mode: str
    generations: int
    population: int
    games_per_eval: int
    initial_sigma: float
    seed: int
    final_fitness: float
    final_winrate: float
    total_time_seconds: float
    total_time_minutes: float
    stop_reason: str
    log_file: str
    config_file: str | None
    stats_file: str | None

    @property
    def datetime(self) -> datetime:
        """Parse timestamp to datetime object."""
        try:
            return datetime.strptime(self.timestamp, "%Y-%m-%d_%H%M")
        except ValueError:
            # Fallback for different formats
            return datetime.strptime(self.timestamp, "%Y-%m-%d_%H%M%S")


@dataclass
class ExperimentRun:
    """Complete experiment data with metadata and generation metrics."""

    metadata: ExperimentMetadata
    generations: list[GenerationMetrics]

    def to_dataframe(self) -> pd.DataFrame:
        """Convert to pandas DataFrame with metadata columns."""
        df = pd.DataFrame([asdict(g) for g in self.generations])

        # Add metadata columns
        df["experiment_id"] = self.metadata.experiment_id
        df["timestamp"] = self.metadata.timestamp
        df["tag"] = self.metadata.tag
        df["mode"] = self.metadata.mode
        df["seed"] = self.metadata.seed
        df["final_fitness"] = self.metadata.final_fitness
        df["final_winrate"] = self.metadata.final_winrate

        # Calculate cumulative time
        df["cumulative_time_seconds"] = df["time_seconds"].cumsum()
        df["cumulative_time_minutes"] = df["cumulative_time_seconds"] / 60.0

        # Calculate improvement metrics
        df["fitness_change"] = df["best_fitness"].diff()
        df["winrate_change"] = df["best_winrate"].diff()

        return df


class ExperimentAggregator:
    """Aggregate and analyze MCTS experiment runs."""

    def __init__(self, experiments_root: Path, cache_dir: Path | None = None):
        """
        Initialize aggregator with experiments directory path.
        
        Args:
            experiments_root: Root directory containing experiments
            cache_dir: Directory for caching parsed experiments (default: experiments_root/.cache)
        """
        self.experiments_root = Path(experiments_root)
        self.cache_dir = cache_dir or (self.experiments_root / ".cache")
        self.cache_dir.mkdir(exist_ok=True)
        self.runs: list[ExperimentRun] = []
        self._cache_hits = 0
        self._cache_misses = 0

    def scan_experiments(
        self,
        min_generations: int = 0,
        mode_filter: str | None = None,
        tag_filter: str | None = None,
    ) -> list[Path]:
        """
        Scan experiments directory for training runs.

        Args:
            min_generations: Minimum number of generations to include
            mode_filter: Filter by mode (e.g., 'multi-opponent', 'generalist')
            tag_filter: Filter by tag substring

        Returns:
            List of experiment directories
        """
        mcts_dir = self.experiments_root / "mcts"

        if not mcts_dir.exists():
            logger.warning(f"MCTS experiments directory not found: {mcts_dir}")
            return []

        # Find all experiment directories (format: YYYY-MM-DD_HHMM_tag)
        experiment_dirs = []
        pattern = re.compile(r"^\d{4}-\d{2}-\d{2}_\d{4}")

        for item in sorted(mcts_dir.iterdir(), reverse=True):
            if not item.is_dir():
                continue

            if not pattern.match(item.name):
                logger.debug(f"Skipping non-experiment directory: {item.name}")
                continue

            # Check for log file
            log_files = list(item.glob("*.log"))
            if not log_files:
                logger.debug(f"No log file found in: {item.name}")
                continue

            # Apply filters
            if tag_filter and tag_filter.lower() not in item.name.lower():
                continue

            experiment_dirs.append(item)

        logger.info(f"Found {len(experiment_dirs)} experiment directories")
        return experiment_dirs

    def _get_cache_key(self, exp_dir: Path) -> str:
        """Generate cache key based on experiment directory and modification time."""
        log_files = list(exp_dir.glob("*.log"))
        if not log_files:
            return ""

        log_file = log_files[0]
        mtime = log_file.stat().st_mtime
        cache_key = f"{exp_dir.name}_{mtime}"
        return hashlib.md5(cache_key.encode()).hexdigest()

    def _load_from_cache(self, cache_key: str) -> ExperimentRun | None:
        """Load experiment from cache if available."""
        cache_file = self.cache_dir / f"{cache_key}.pkl"
        if cache_file.exists():
            try:
                with open(cache_file, 'rb') as f:
                    return pickle.load(f)
            except Exception as e:
                logger.debug(f"Cache load failed: {e}")
                cache_file.unlink(missing_ok=True)
        return None

    def _save_to_cache(self, cache_key: str, run: ExperimentRun) -> None:
        """Save experiment to cache."""
        cache_file = self.cache_dir / f"{cache_key}.pkl"
        try:
            with open(cache_file, 'wb') as f:
                pickle.dump(run, f)
        except Exception as e:
            logger.debug(f"Cache save failed: {e}")

    def parse_experiment(self, exp_dir: Path, use_cache: bool = True) -> ExperimentRun | None:
        """
        Parse a single experiment directory.

        Args:
            exp_dir: Path to experiment directory
            use_cache: Whether to use cached results

        Returns:
            ExperimentRun object or None if parsing fails
        """
        # Try cache first
        if use_cache:
            cache_key = self._get_cache_key(exp_dir)
            if cache_key:
                cached = self._load_from_cache(cache_key)
                if cached:
                    self._cache_hits += 1
                    logger.debug(f"Cache hit: {exp_dir.name}")
                    return cached
                self._cache_misses += 1

        try:
            # Find log file
            log_files = list(exp_dir.glob("*.log"))
            if not log_files:
                logger.warning(f"No log file in {exp_dir.name}")
                return None

            log_file = log_files[0]

            # Parse log file
            with open(log_file) as f:
                content = f.read()

            # Extract experiment ID and metadata
            exp_id_match = re.search(r"Experiment ID:\s*([^\n]+)", content)
            mode_match = re.search(r"Mode:\s*([^\n]+)", content)

            if not exp_id_match:
                logger.warning(f"Could not parse experiment ID from {log_file.name}")
                return None

            experiment_id = exp_id_match.group(1).strip()
            mode = mode_match.group(1).strip() if mode_match else "unknown"

            # Parse timestamp and tag from directory name
            parts = exp_dir.name.split("_", 2)
            if len(parts) >= 3:
                timestamp = f"{parts[0]}_{parts[1]}"
                tag = parts[2]
            else:
                timestamp = exp_dir.name
                tag = "unknown"

            # Parse configuration
            generations = self._extract_int(content, r"Generations:\s*(\d+)", 0)
            population = self._extract_int(content, r"Population:\s*(\d+)", 0)
            games_per_eval = self._extract_int(content, r"Games/eval:\s*(\d+)", 0)
            initial_sigma = self._extract_float(content, r"Initial sigma:\s*([\d.]+)", 0.0)
            seed = self._extract_int(content, r"Seed:\s*(\d+)", 0)

            # Parse generation data
            gen_pattern = re.compile(
                r"Gen\s+(\d+):\s+best_fit=\s*([\d.]+),\s+"
                r"best_wr\s*=\s*([\d.]+)%,\s+sigma=([\d.]+),\s+time=([\d.]+)s"
            )

            generation_metrics = []
            for match in gen_pattern.finditer(content):
                gen_num = int(match.group(1))
                fitness = float(match.group(2))
                winrate = float(match.group(3))
                sigma = float(match.group(4))
                time_sec = float(match.group(5))

                generation_metrics.append(
                    GenerationMetrics(gen_num, fitness, winrate, sigma, time_sec)
                )

            if not generation_metrics:
                logger.warning(f"No generation data found in {log_file.name}")
                return None

            # Parse final results
            final_fitness = self._extract_float(content, r"Best fitness:\s*([\d.]+)", 0.0)
            final_winrate = self._extract_float(content, r"Best win rate:\s*([\d.]+)%", 0.0)
            total_time = self._extract_float(content, r"Total time:\s*([\d.]+)s", 0.0)

            stop_match = re.search(r"Stop reason:\s*([^\n]+)", content)
            stop_reason = stop_match.group(1).strip() if stop_match else "unknown"

            # Check for config and stats files
            config_file = exp_dir / "config.yaml"
            stats_file = exp_dir / "stats.csv"

            metadata = ExperimentMetadata(
                experiment_id=experiment_id,
                timestamp=timestamp,
                tag=tag,
                mode=mode,
                generations=generations,
                population=population,
                games_per_eval=games_per_eval,
                initial_sigma=initial_sigma,
                seed=seed,
                final_fitness=final_fitness,
                final_winrate=final_winrate,
                total_time_seconds=total_time,
                total_time_minutes=total_time / 60.0,
                stop_reason=stop_reason,
                log_file=str(log_file),
                config_file=str(config_file) if config_file.exists() else None,
                stats_file=str(stats_file) if stats_file.exists() else None,
            )

            run = ExperimentRun(metadata=metadata, generations=generation_metrics)

            # Save to cache
            if use_cache:
                cache_key = self._get_cache_key(exp_dir)
                if cache_key:
                    self._save_to_cache(cache_key, run)

            return run

        except Exception as e:
            logger.error(f"Error parsing {exp_dir.name}: {e}", exc_info=True)
            return None

    def aggregate_all(
        self,
        min_generations: int = 0,
        mode_filter: str | None = None,
        tag_filter: str | None = None,
        use_cache: bool = True,
    ) -> pd.DataFrame:
        """
        Aggregate all experiments into a single DataFrame.

        Args:
            min_generations: Minimum generations to include
            mode_filter: Filter by mode
            tag_filter: Filter by tag
            use_cache: Whether to use cached parsing results

        Returns:
            Consolidated DataFrame with all experiment data
        """
        exp_dirs = self.scan_experiments(min_generations, mode_filter, tag_filter)

        self.runs = []
        failed = []
        self._cache_hits = 0
        self._cache_misses = 0

        for exp_dir in exp_dirs:
            run = self.parse_experiment(exp_dir, use_cache=use_cache)
            if run:
                if len(run.generations) >= min_generations:
                    self.runs.append(run)
                else:
                    logger.debug(
                        f"Skipping {exp_dir.name}: only {len(run.generations)} generations"
                    )
            else:
                failed.append(exp_dir.name)

        if failed:
            logger.warning(f"Failed to parse {len(failed)} experiments: {failed[:5]}...")

        cache_msg = f" (cache: {self._cache_hits} hits, {self._cache_misses} misses)" if use_cache else ""
        logger.info(f"Successfully parsed {len(self.runs)} experiments{cache_msg}")

        # Combine all runs into single DataFrame
        if not self.runs:
            return pd.DataFrame()

        dfs = [run.to_dataframe() for run in self.runs]
        combined = pd.concat(dfs, ignore_index=True)

        return combined

    def get_summary_stats(self) -> pd.DataFrame:
        """Get summary statistics for all runs."""
        if not self.runs:
            return pd.DataFrame()

        summary_data = []
        for run in self.runs:
            meta = run.metadata
            summary_data.append(
                {
                    "experiment_id": meta.experiment_id,
                    "timestamp": meta.timestamp,
                    "tag": meta.tag,
                    "mode": meta.mode,
                    "num_generations": len(run.generations),
                    "final_fitness": meta.final_fitness,
                    "final_winrate": meta.final_winrate,
                    "total_time_min": meta.total_time_minutes,
                    "avg_time_per_gen": meta.total_time_seconds / len(run.generations),
                    "stop_reason": meta.stop_reason,
                }
            )

        return pd.DataFrame(summary_data)

    @staticmethod
    def _extract_int(text: str, pattern: str, default: int = 0) -> int:
        """Extract integer from text using regex pattern."""
        match = re.search(pattern, text)
        return int(match.group(1)) if match else default

    @staticmethod
    def _extract_float(text: str, pattern: str, default: float = 0.0) -> float:
        """Extract float from text using regex pattern."""
        match = re.search(pattern, text)
        return float(match.group(1)) if match else default
