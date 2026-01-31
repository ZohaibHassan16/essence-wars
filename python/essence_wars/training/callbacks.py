"""
Training callbacks for Essence Wars agents.

Provides a structured callback system for training hooks, including
auto-evaluation and auto-report generation.

Example:
    >>> from essence_wars.training.callbacks import (
    ...     CallbackList, CheckpointCallback, AutoEvaluateCallback
    ... )
    >>> callbacks = CallbackList([
    ...     CheckpointCallback(save_path, save_freq=1000),
    ...     AutoEvaluateCallback(eval_games=200),
    ... ])
    >>> results = trainer.train(callback=callbacks.as_functional())
    >>> callbacks.on_train_complete(trainer, results)
"""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import TYPE_CHECKING, Any, Callable

if TYPE_CHECKING:
    # Avoid circular imports
    from essence_wars.agents.ppo import PPOTrainer


@dataclass
class CallbackContext:
    """Context passed to callbacks.

    Provides access to training state and configuration.
    """

    trainer: Any  # PPOTrainer or AlphaZeroTrainer
    experiment_dir: Path | None = None
    config: dict = field(default_factory=dict)
    start_time: datetime = field(default_factory=datetime.now)

    @property
    def elapsed_seconds(self) -> float:
        """Seconds elapsed since training started."""
        return (datetime.now() - self.start_time).total_seconds()


class TrainingCallback(ABC):
    """Abstract base class for training callbacks.

    Callbacks provide hooks at various points in the training lifecycle:
    - on_train_start: Called once before training begins
    - on_step: Called after each training step/update
    - on_train_complete: Called once after training finishes

    The on_step method can return True to stop training early.
    """

    def on_train_start(self, context: CallbackContext) -> None:
        """Called when training starts.

        Override to perform initialization or logging.
        """
        pass

    @abstractmethod
    def on_step(self, step: int, info: dict, context: CallbackContext) -> bool:
        """Called after each training step.

        Args:
            step: Current global step
            info: Dictionary with training metrics
            context: Training context

        Returns:
            True to stop training, False to continue.
        """
        pass

    def on_train_complete(
        self,
        context: CallbackContext,
        results: dict
    ) -> None:
        """Called when training completes.

        Override to perform cleanup, final evaluation, or reporting.

        Args:
            context: Training context
            results: Dictionary of training results
        """
        pass


class CallbackList:
    """Container for multiple callbacks.

    Runs all callbacks in order, stopping if any callback requests stop.

    Example:
        >>> callbacks = CallbackList([
        ...     CheckpointCallback(save_path, save_freq=1000),
        ...     LoggingCallback(),
        ... ])
        >>> results = trainer.train(callback=callbacks.as_functional())
        >>> callbacks.on_train_complete(trainer, results)
    """

    def __init__(self, callbacks: list[TrainingCallback] | None = None):
        self.callbacks = callbacks or []
        self.context: CallbackContext | None = None

    def add(self, callback: TrainingCallback) -> None:
        """Add a callback to the list."""
        self.callbacks.append(callback)

    def on_train_start(self, context: CallbackContext) -> None:
        """Call on_train_start for all callbacks."""
        self.context = context
        for callback in self.callbacks:
            callback.on_train_start(context)

    def on_step(self, step: int, info: dict) -> bool:
        """Call on_step for all callbacks.

        Returns True if any callback requests stop.
        """
        if self.context is None:
            raise RuntimeError("on_train_start must be called before on_step")

        for callback in self.callbacks:
            if callback.on_step(step, info, self.context):
                return True
        return False

    def on_train_complete(self, results: dict) -> None:
        """Call on_train_complete for all callbacks."""
        if self.context is None:
            raise RuntimeError("on_train_start must be called before on_train_complete")

        for callback in self.callbacks:
            callback.on_train_complete(self.context, results)

    def as_functional(self) -> Callable[[int, dict], bool]:
        """Convert to functional callback for trainer.train().

        Note: You must call on_train_start() before training and
        on_train_complete() after training manually.

        Returns:
            Callable compatible with trainer.train(callback=...)
        """
        return self.on_step


class CheckpointCallback(TrainingCallback):
    """Save model checkpoints during training.

    Example:
        >>> callback = CheckpointCallback(
        ...     save_path=Path("experiments/run1"),
        ...     save_freq=5000,
        ...     save_best=True,
        ... )
    """

    def __init__(
        self,
        save_path: Path | str,
        save_freq: int = 5000,
        save_best: bool = True,
        metric: str = "win_rate",
    ):
        """Initialize checkpoint callback.

        Args:
            save_path: Directory to save checkpoints
            save_freq: Save every N steps
            save_best: Also save best model based on metric
            metric: Metric to use for best model selection
        """
        self.save_path = Path(save_path)
        self.save_freq = save_freq
        self.save_best = save_best
        self.metric = metric
        self.best_metric_value = float("-inf")
        self.last_save_step = 0

    def on_train_start(self, context: CallbackContext) -> None:
        """Ensure save directory exists."""
        self.save_path.mkdir(parents=True, exist_ok=True)

    def on_step(self, step: int, info: dict, context: CallbackContext) -> bool:
        """Save checkpoint if save_freq steps have passed."""
        trainer = context.trainer

        # Periodic checkpoint
        if step - self.last_save_step >= self.save_freq:
            checkpoint_path = self.save_path / f"checkpoint_{step}.pt"
            if hasattr(trainer, "save"):
                trainer.save(str(checkpoint_path))
            self.last_save_step = step

        # Best model checkpoint
        if self.save_best and self.metric in info:
            metric_value = info[self.metric]
            if metric_value > self.best_metric_value:
                self.best_metric_value = metric_value
                best_path = self.save_path / "best_model.pt"
                if hasattr(trainer, "save"):
                    trainer.save(str(best_path))

        return False  # Never stop training


class EvaluationCallback(TrainingCallback):
    """Run evaluation during training.

    Periodically evaluates the agent against baseline opponents.

    Example:
        >>> callback = EvaluationCallback(
        ...     eval_freq=10000,
        ...     eval_games=100,
        ... )
    """

    def __init__(
        self,
        eval_freq: int = 10000,
        eval_games: int = 100,
        verbose: bool = True,
    ):
        """Initialize evaluation callback.

        Args:
            eval_freq: Evaluate every N steps
            eval_games: Games per evaluation
            verbose: Print evaluation results
        """
        self.eval_freq = eval_freq
        self.eval_games = eval_games
        self.verbose = verbose
        self.last_eval_step = 0
        self.eval_history: list[dict] = []

    def on_step(self, step: int, info: dict, context: CallbackContext) -> bool:
        """Run evaluation if eval_freq steps have passed."""
        if step - self.last_eval_step < self.eval_freq:
            return False

        trainer = context.trainer
        self.last_eval_step = step

        # Run evaluations
        results = {"step": step}

        if hasattr(trainer, "evaluate_vs_greedy"):
            results["vs_greedy"] = trainer.evaluate_vs_greedy(self.eval_games)

        if hasattr(trainer, "evaluate_vs_random"):
            results["vs_random"] = trainer.evaluate_vs_random(self.eval_games)

        self.eval_history.append(results)

        if self.verbose:
            parts = [f"Step {step:,}"]
            if "vs_greedy" in results:
                parts.append(f"vs Greedy: {results['vs_greedy']:.1%}")
            if "vs_random" in results:
                parts.append(f"vs Random: {results['vs_random']:.1%}")
            print(" | ".join(parts))

        return False


class AutoEvaluateCallback(TrainingCallback):
    """Run final evaluation after training completes.

    Automatically evaluates the trained agent against baseline opponents
    and optionally updates ELO ratings.

    Example:
        >>> callback = AutoEvaluateCallback(
        ...     eval_games=200,
        ...     update_elo=True,
        ... )
    """

    def __init__(
        self,
        eval_games: int = 200,
        update_elo: bool = False,
        elo_file: Path | str | None = None,
        verbose: bool = True,
    ):
        """Initialize auto-evaluate callback.

        Args:
            eval_games: Number of games for final evaluation
            update_elo: Whether to update agent ELO ratings
            elo_file: Path to agent_elo.json (default: data/ratings/agent_elo.json)
            verbose: Print evaluation results
        """
        self.eval_games = eval_games
        self.update_elo = update_elo
        self.elo_file = Path(elo_file) if elo_file else None
        self.verbose = verbose
        self.results: dict = {}

    def on_step(self, step: int, info: dict, context: CallbackContext) -> bool:
        """No-op during training."""
        return False

    def on_train_complete(
        self,
        context: CallbackContext,
        results: dict
    ) -> None:
        """Run final evaluation."""
        trainer = context.trainer

        if self.verbose:
            print("\n" + "=" * 60)
            print("Auto-Evaluation (Post-Training)")
            print("=" * 60)

        # Run evaluations
        if hasattr(trainer, "evaluate_vs_greedy"):
            self.results["vs_greedy"] = trainer.evaluate_vs_greedy(self.eval_games)
            if self.verbose:
                print(f"  vs GreedyBot: {self.results['vs_greedy']:.1%} win rate")

        if hasattr(trainer, "evaluate_vs_random"):
            self.results["vs_random"] = trainer.evaluate_vs_random(self.eval_games)
            if self.verbose:
                print(f"  vs RandomBot: {self.results['vs_random']:.1%} win rate")

        # Update ELO ratings if requested
        if self.update_elo and self.results:
            self._update_elo_ratings(context)

    def _update_elo_ratings(self, context: CallbackContext) -> None:
        """Update agent ELO ratings based on evaluation results."""
        try:
            from essence_wars.ratings import AgentRatings

            elo_file = self.elo_file or Path("data/ratings/agent_elo.json")

            # Load or create ratings
            try:
                ratings = AgentRatings.load(elo_file)
            except FileNotFoundError:
                ratings = AgentRatings()

            # Determine agent ID
            agent_id = context.config.get("agent_id", "ppo_trained")
            agent_type = context.config.get("agent_type", "ppo")

            # Record games against greedy
            if "vs_greedy" in self.results:
                win_rate = self.results["vs_greedy"]
                wins = int(win_rate * self.eval_games)
                losses = self.eval_games - wins

                for _ in range(wins):
                    ratings.update(agent_id, "greedy", winner=0,
                                   agent1_type=agent_type, agent2_type="greedy")
                for _ in range(losses):
                    ratings.update(agent_id, "greedy", winner=1,
                                   agent1_type=agent_type, agent2_type="greedy")

            # Save updated ratings
            ratings.save(elo_file)

            if self.verbose:
                agent_rating = ratings.get(agent_id)
                if agent_rating:
                    print(f"  ELO Rating: {agent_rating.rating:.0f}")

        except Exception as e:
            if self.verbose:
                print(f"  Warning: Could not update ELO ratings: {e}")


class AutoReportCallback(TrainingCallback):
    """Generate HTML report after training completes.

    Automatically generates a training report with metrics, charts,
    and evaluation results.

    Example:
        >>> callback = AutoReportCallback(
        ...     output_dir=Path("reports/training"),
        ...     include_eval=True,
        ... )
    """

    def __init__(
        self,
        output_dir: Path | str | None = None,
        include_eval: bool = True,
        open_browser: bool = False,
        verbose: bool = True,
    ):
        """Initialize auto-report callback.

        Args:
            output_dir: Directory for report output
            include_eval: Include evaluation results in report
            open_browser: Open report in browser after generation
            verbose: Print report generation status
        """
        self.output_dir = Path(output_dir) if output_dir else None
        self.include_eval = include_eval
        self.open_browser = open_browser
        self.verbose = verbose
        self.report_path: Path | None = None

    def on_step(self, step: int, info: dict, context: CallbackContext) -> bool:
        """No-op during training."""
        return False

    def on_train_complete(
        self,
        context: CallbackContext,
        results: dict
    ) -> None:
        """Generate training report."""
        output_dir = self.output_dir or context.experiment_dir
        if output_dir is None:
            if self.verbose:
                print("Warning: No output directory for report, skipping")
            return

        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        if self.verbose:
            print("\n" + "=" * 60)
            print("Generating Training Report")
            print("=" * 60)

        try:
            self._generate_report(context, results, output_dir)
        except Exception as e:
            if self.verbose:
                print(f"  Warning: Could not generate report: {e}")

    def _generate_report(
        self,
        context: CallbackContext,
        results: dict,
        output_dir: Path
    ) -> None:
        """Generate the HTML report."""
        trainer = context.trainer

        # Collect training info
        training_info = {
            "timesteps": getattr(trainer, "global_step", 0),
            "elapsed_time": context.elapsed_seconds,
            "best_win_rate": results.get("best_win_rate", 0),
            "final_loss": results.get("final_loss"),
        }

        # Add config
        training_info["config"] = context.config

        # Generate simple HTML report
        report_html = self._build_report_html(training_info, results)

        self.report_path = output_dir / "training_report.html"
        self.report_path.write_text(report_html)

        if self.verbose:
            print(f"  Report saved to: {self.report_path}")

        if self.open_browser:
            import webbrowser
            webbrowser.open(f"file://{self.report_path.absolute()}")

    def _build_report_html(self, info: dict, results: dict) -> str:
        """Build HTML report content."""
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

        # Format duration
        elapsed = info.get("elapsed_time", 0)
        hours, remainder = divmod(int(elapsed), 3600)
        minutes, seconds = divmod(remainder, 60)
        duration_str = f"{hours}h {minutes}m {seconds}s"

        # Build config table
        config = info.get("config", {})
        config_rows = "\n".join(
            f"<tr><td>{k}</td><td>{v}</td></tr>"
            for k, v in config.items()
            if not k.startswith("_")
        )

        return f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Training Report - {timestamp}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 900px;
            margin: 0 auto;
            padding: 20px;
            background: #1a1a2e;
            color: #eaeaea;
        }}
        h1, h2 {{ color: #e94560; }}
        .metric-card {{
            background: #16213e;
            border-radius: 8px;
            padding: 16px;
            margin: 10px 0;
            display: inline-block;
            min-width: 150px;
        }}
        .metric-value {{
            font-size: 2em;
            font-weight: bold;
            color: #00d26a;
        }}
        .metric-label {{ color: #888; font-size: 0.9em; }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            padding: 10px;
            text-align: left;
            border-bottom: 1px solid #333;
        }}
        th {{ background: #16213e; color: #e94560; }}
        .footer {{ color: #666; font-size: 0.8em; margin-top: 40px; }}
    </style>
</head>
<body>
    <h1>Training Report</h1>
    <p>Generated: {timestamp}</p>

    <h2>Summary</h2>
    <div class="metric-card">
        <div class="metric-value">{info.get('timesteps', 0):,}</div>
        <div class="metric-label">Total Timesteps</div>
    </div>
    <div class="metric-card">
        <div class="metric-value">{info.get('best_win_rate', 0):.1%}</div>
        <div class="metric-label">Best Win Rate</div>
    </div>
    <div class="metric-card">
        <div class="metric-value">{duration_str}</div>
        <div class="metric-label">Training Duration</div>
    </div>

    <h2>Configuration</h2>
    <table>
        <tr><th>Parameter</th><th>Value</th></tr>
        {config_rows if config_rows else "<tr><td colspan='2'>No config available</td></tr>"}
    </table>

    <div class="footer">
        Generated by Essence Wars Training Pipeline
    </div>
</body>
</html>"""


class LoggingCallback(TrainingCallback):
    """Log training metrics to console or file.

    Example:
        >>> callback = LoggingCallback(log_freq=100, log_file="training.log")
    """

    def __init__(
        self,
        log_freq: int = 100,
        log_file: Path | str | None = None,
        metrics: list[str] | None = None,
    ):
        """Initialize logging callback.

        Args:
            log_freq: Log every N steps
            log_file: Optional file to write logs
            metrics: Specific metrics to log (default: all)
        """
        self.log_freq = log_freq
        self.log_file = Path(log_file) if log_file else None
        self.metrics = metrics
        self.last_log_step = 0
        self._file_handle = None

    def on_train_start(self, context: CallbackContext) -> None:
        """Open log file if specified."""
        if self.log_file:
            self.log_file.parent.mkdir(parents=True, exist_ok=True)
            self._file_handle = open(self.log_file, "w")
            self._file_handle.write("step,timestamp," + ",".join(self.metrics or []) + "\n")

    def on_step(self, step: int, info: dict, context: CallbackContext) -> bool:
        """Log metrics if log_freq steps have passed."""
        if step - self.last_log_step < self.log_freq:
            return False

        self.last_log_step = step

        # Filter metrics
        if self.metrics:
            logged = {k: v for k, v in info.items() if k in self.metrics}
        else:
            logged = info

        # Format log line
        timestamp = datetime.now().isoformat()
        values = [str(logged.get(m, "")) for m in (self.metrics or logged.keys())]

        if self._file_handle:
            self._file_handle.write(f"{step},{timestamp}," + ",".join(values) + "\n")
            self._file_handle.flush()

        return False

    def on_train_complete(
        self,
        context: CallbackContext,
        results: dict
    ) -> None:
        """Close log file."""
        if self._file_handle:
            self._file_handle.close()
            self._file_handle = None


# Convenience function for simple use cases
def make_callback(
    save_path: Path | str | None = None,
    save_freq: int = 5000,
    eval_freq: int = 10000,
    eval_games: int = 100,
    auto_evaluate: bool = True,
    auto_report: bool = False,
    update_elo: bool = False,
) -> CallbackList:
    """Create a callback list with common settings.

    Args:
        save_path: Directory for checkpoints and reports
        save_freq: Checkpoint save frequency
        eval_freq: Evaluation frequency during training
        eval_games: Games per evaluation
        auto_evaluate: Run final evaluation after training
        auto_report: Generate HTML report after training
        update_elo: Update ELO ratings after evaluation

    Returns:
        Configured CallbackList ready for training.

    Example:
        >>> callbacks = make_callback(
        ...     save_path="experiments/run1",
        ...     auto_evaluate=True,
        ...     auto_report=True,
        ... )
        >>> context = CallbackContext(trainer=trainer, experiment_dir=save_path)
        >>> callbacks.on_train_start(context)
        >>> results = trainer.train(callback=callbacks.as_functional())
        >>> callbacks.on_train_complete(results)
    """
    callbacks = CallbackList()

    if save_path:
        callbacks.add(CheckpointCallback(
            save_path=save_path,
            save_freq=save_freq,
            save_best=True,
        ))

    if eval_freq > 0:
        callbacks.add(EvaluationCallback(
            eval_freq=eval_freq,
            eval_games=eval_games,
        ))

    if auto_evaluate:
        callbacks.add(AutoEvaluateCallback(
            eval_games=eval_games * 2,  # More games for final eval
            update_elo=update_elo,
        ))

    if auto_report and save_path:
        callbacks.add(AutoReportCallback(
            output_dir=save_path,
        ))

    return callbacks
