"""Tests for the Experiment class."""

from __future__ import annotations

import json
import tempfile

from essence_wars.infra import Experiment


def test_experiment_creates_directories() -> None:
    """Test that Experiment creates the expected directory structure."""
    with tempfile.TemporaryDirectory() as tmpdir:
        exp = Experiment("tuning", tag="test", root_dir=tmpdir)

        assert exp.dir.exists()
        assert (exp.dir / "checkpoints").exists()
        assert (exp.dir / "plots").exists()
        assert "test" in exp.id


def test_experiment_saves_config() -> None:
    """Test that config is saved as JSON."""
    with tempfile.TemporaryDirectory() as tmpdir:
        exp = Experiment("training", tag="config_test", root_dir=tmpdir)
        config = {"learning_rate": 0.001, "epochs": 100}

        path = exp.save_config(config)

        assert path.exists()
        with path.open() as f:
            loaded = json.load(f)
        assert loaded == config


def test_experiment_logs_metrics() -> None:
    """Test metric logging and saving."""
    with tempfile.TemporaryDirectory() as tmpdir:
        exp = Experiment("eval", tag="metrics_test", root_dir=tmpdir)

        exp.log_metric("loss", 0.5, step=1)
        exp.log_metric("loss", 0.3, step=2)
        exp.log_metric("accuracy", 0.9, step=2)

        path = exp.save_metrics()

        assert path is not None
        assert path.exists()
        content = path.read_text()
        assert "loss" in content
        assert "0.5" in content


def test_experiment_saves_artifact() -> None:
    """Test saving arbitrary artifacts."""
    with tempfile.TemporaryDirectory() as tmpdir:
        exp = Experiment("tuning", tag="artifact_test", root_dir=tmpdir)
        data = {"weights": [1.0, 2.0, 3.0], "metadata": {"version": 1}}

        path = exp.save_artifact("model_info", data)

        assert path.exists()
        with path.open() as f:
            loaded = json.load(f)
        assert loaded == data


def test_experiment_id_format() -> None:
    """Test that experiment ID follows the expected format."""
    with tempfile.TemporaryDirectory() as tmpdir:
        exp = Experiment("tuning", tag="baseline", root_dir=tmpdir)

        # ID should be: YYYY-MM-DD_HHMM_tag
        assert "baseline" in exp.id
        assert exp.id.endswith("_baseline")
