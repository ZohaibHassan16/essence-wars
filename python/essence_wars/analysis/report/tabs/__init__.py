"""Tab generators for HTML reports."""

from .elo import EloTab
from .overview import OverviewTab
from .research import ResearchTab
from .tuning import TuningTab
from .validation import ValidationTab

__all__ = ["EloTab", "OverviewTab", "ResearchTab", "TuningTab", "ValidationTab"]
