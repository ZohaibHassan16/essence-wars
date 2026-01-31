"""Tab generators for HTML reports."""

from .overview import OverviewTab
from .validation import ValidationTab
from .tuning import TuningTab
from .elo import EloTab

__all__ = ["OverviewTab", "ValidationTab", "TuningTab", "EloTab"]
