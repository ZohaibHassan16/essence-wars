"""Placeholder tests - will be expanded when PyO3 bindings are ready."""

from __future__ import annotations


def test_imports() -> None:
    """Test that basic imports work."""
    import cardgame

    assert cardgame.__version__ == "0.1.0"


def test_gymnasium_available() -> None:
    """Test that gymnasium is available."""
    import gymnasium

    assert gymnasium is not None
