"""HTML report generation for Essence Wars validation and tuning results.

Usage:
    python -m essence_wars.analysis.report --run-id latest
    python -m essence_wars.analysis.report --all --output reports/dashboard
"""

from .generator import ReportGenerator

__all__ = ["ReportGenerator"]
