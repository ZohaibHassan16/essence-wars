"""Validation tab generator for HTML reports.

Provides detailed deck performance tables and matchup analysis.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from ..charts import create_deck_winrate_bar, create_matchup_heatmap

if TYPE_CHECKING:
    from ..loaders.validation import ValidationData


class ValidationTab:
    """Generator for the Validation tab."""

    def __init__(self, data: "ValidationData"):
        self.data = data

    def render(self) -> str:
        """Render the Validation tab content as HTML."""
        html_parts = []

        # Deck performance bar chart
        html_parts.append('<div class="chart-section">')
        html_parts.append("<h3>Deck Win Rates</h3>")
        html_parts.append('<div class="chart-container">')
        html_parts.append(create_deck_winrate_bar(self.data.deck_stats))
        html_parts.append("</div>")
        html_parts.append("</div>")

        # Deck performance table
        html_parts.append('<div class="table-section">')
        html_parts.append("<h3>Deck Performance Details</h3>")
        html_parts.append(self._render_deck_table())
        html_parts.append("</div>")

        # Matchup heatmap
        html_parts.append('<div class="chart-section">')
        html_parts.append("<h3>Matchup Analysis</h3>")
        html_parts.append('<div class="chart-container">')
        html_parts.append(create_matchup_heatmap(self.data))
        html_parts.append("</div>")
        html_parts.append("</div>")

        return "\n".join(html_parts)

    def _render_deck_table(self) -> str:
        """Render the deck performance table."""
        sorted_stats = sorted(self.data.deck_stats, key=lambda d: d.win_rate, reverse=True)

        rows = []
        for i, deck in enumerate(sorted_stats, 1):
            # Determine status indicator
            if deck.win_rate > 0.6:
                indicator = '<span class="indicator high">&#9650;</span>'
            elif deck.win_rate < 0.4:
                indicator = '<span class="indicator low">&#9660;</span>'
            else:
                indicator = '<span class="indicator balanced">&#8226;</span>'

            # Format confidence interval
            ci = f"[{deck.win_rate_ci_lower * 100:.1f}%, {deck.win_rate_ci_upper * 100:.1f}%]"

            # Format best/worst matchups
            best_opponent, best_rate = deck.best_matchup
            worst_opponent, worst_rate = deck.worst_matchup

            rows.append(
                f"""
                <tr>
                    <td>{i}</td>
                    <td class="deck-cell">
                        <span class="commander-name">{deck.commander_name}</span>
                        <span class="deck-id">({deck.deck_id})</span>
                    </td>
                    <td><span class="faction-badge faction-{deck.faction}">{deck.faction.title()}</span></td>
                    <td class="win-rate-cell">{indicator} {deck.win_rate * 100:.1f}%</td>
                    <td class="ci-cell">{ci}</td>
                    <td>{deck.total_games}</td>
                    <td class="matchup-cell">
                        <span class="matchup-good">{best_opponent}</span>
                        <span class="matchup-rate">{best_rate * 100:.0f}%</span>
                    </td>
                    <td class="matchup-cell">
                        <span class="matchup-bad">{worst_opponent}</span>
                        <span class="matchup-rate">{worst_rate * 100:.0f}%</span>
                    </td>
                </tr>
                """
            )

        return f"""
        <div class="table-wrapper">
            <table class="deck-table">
                <thead>
                    <tr>
                        <th>#</th>
                        <th>Commander</th>
                        <th>Faction</th>
                        <th>Win Rate</th>
                        <th>95% CI</th>
                        <th>Games</th>
                        <th>Best Matchup</th>
                        <th>Worst Matchup</th>
                    </tr>
                </thead>
                <tbody>
                    {"".join(rows)}
                </tbody>
            </table>
        </div>
        """
