"""Placeholder tests - will be expanded when PyO3 bindings are ready."""

from __future__ import annotations


def test_imports() -> None:
    """Test that basic imports work."""
    import essence_wars

    # Version should be a valid semver string (e.g., "0.8.3")
    assert isinstance(essence_wars.__version__, str)
    parts = essence_wars.__version__.split(".")
    assert len(parts) >= 2, f"Invalid version format: {essence_wars.__version__}"
    assert all(p.isdigit() for p in parts[:2]), f"Invalid version format: {essence_wars.__version__}"


def test_gymnasium_available() -> None:
    """Test that gymnasium is available."""
    import gymnasium

    assert gymnasium is not None
