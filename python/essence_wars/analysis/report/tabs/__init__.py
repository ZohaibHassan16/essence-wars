"""Tab generators for HTML reports."""

from .overview import OverviewTab
from .validation import ValidationTab
from .tuning import TuningTab
from .elo import EloTab
from .research import ResearchTab

__all__ = ["OverviewTab", "ValidationTab", "TuningTab", "EloTab", "ResearchTab"]
