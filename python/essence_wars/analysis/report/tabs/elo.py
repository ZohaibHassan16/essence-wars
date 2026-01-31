"""ELO ratings tab generator for HTML reports.

Displays deck rankings, rating history, and matchup predictions.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from ..charts import (
    create_elo_rankings_bar,
    create_elo_timeline,
    create_elo_prediction_heatmap,
    create_faction_elo_comparison,
)

if TYPE_CHECKING:
    from ..loaders.elo import EloData


class EloTab:
    """Generator for the ELO Ratings tab."""

    def __init__(self, elo_data: "EloData"):
        """Initialize with ELO data.

        Args:
            elo_data: EloData object with ratings
        """
        self.data = elo_data

    def render(self) -> str:
        """Render the ELO tab content as HTML."""
        if not self.data.ratings:
            return '<div class="no-data">No ELO ratings data available.</div>'

        html_parts = []

        # Get ranking data
        ranked_decks = self.data.get_ranked_decks()
        top_deck = ranked_decks[0] if ranked_decks else None
        bottom_deck = ranked_decks[-1] if ranked_decks else None

        # Summary metrics
        html_parts.append('<div class="metrics-grid">')
        html_parts.append(
            self._render_metric_card(
                "Rated Decks",
                str(len(self.data.ratings)),
                "fa-ranking-star",
            )
        )

        if top_deck:
            html_parts.append(
                self._render_metric_card(
                    "Top Rated",
                    f"{top_deck.commander_name} ({top_deck.rating:.0f})",
                    "fa-crown",
                    status="success",
                )
            )

        # Total games
        total_games = sum(d.games for d in ranked_decks) // 2  # Divide by 2 since each game counted twice
        html_parts.append(
            self._render_metric_card(
                "Total Games",
                str(total_games),
                "fa-gamepad",
            )
        )

        # Rating spread
        if top_deck and bottom_deck:
            spread = top_deck.rating - bottom_deck.rating
            html_parts.append(
                self._render_metric_card(
                    "Rating Spread",
                    f"{spread:.0f}",
                    "fa-arrows-up-down",
                )
            )

        html_parts.append("</div>")  # End metrics-grid

        # Charts row - rankings and timeline
        html_parts.append('<div class="charts-row">')

        # ELO rankings bar chart
        html_parts.append('<div class="chart-card">')
        html_parts.append(create_elo_rankings_bar(self.data))
        html_parts.append("</div>")

        # Rating timeline
        html_parts.append('<div class="chart-card">')
        html_parts.append(create_elo_timeline(self.data, max_decks=6))
        html_parts.append("</div>")

        html_parts.append("</div>")  # End charts-row

        # Faction comparison
        html_parts.append('<div class="chart-section">')
        html_parts.append("<h3>Faction Performance</h3>")
        html_parts.append(create_faction_elo_comparison(self.data))
        html_parts.append("</div>")

        # Rankings table
        html_parts.append('<div class="table-section">')
        html_parts.append("<h3>Current Rankings</h3>")
        html_parts.append('<div class="table-wrapper">')
        html_parts.append('<table class="deck-table" id="elo-rankings-table">')
        html_parts.append("<thead>")
        html_parts.append("<tr>")
        html_parts.append("<th>#</th>")
        html_parts.append("<th>Commander</th>")
        html_parts.append("<th>Faction</th>")
        html_parts.append("<th>ELO</th>")
        html_parts.append("<th>W-L-D</th>")
        html_parts.append("<th>Win Rate</th>")
        html_parts.append("<th>Recent</th>")
        html_parts.append("</tr>")
        html_parts.append("</thead>")
        html_parts.append("<tbody>")

        for i, deck in enumerate(ranked_decks, 1):
            # Determine rating trend from recent history
            trend = self._get_trend(deck)
            trend_icon = {
                "up": '<span class="trend-up">+</span>',
                "down": '<span class="trend-down">-</span>',
                "stable": '<span class="trend-stable">=</span>',
            }.get(trend, "")

            # Recent form (last 5 results)
            recent = self._get_recent_form(deck, 5)

            html_parts.append("<tr>")
            html_parts.append(f"<td>{i}</td>")
            html_parts.append(f'<td class="deck-cell">')
            html_parts.append(f'<span class="commander-name">{deck.commander_name}</span>')
            html_parts.append(f'<span class="deck-id">{deck.deck_id}</span>')
            html_parts.append("</td>")
            html_parts.append(f'<td><span class="faction-badge faction-{deck.faction}">{(deck.faction or "unknown").title()}</span></td>')
            html_parts.append(f"<td><strong>{deck.rating:.0f}</strong> {trend_icon}</td>")
            html_parts.append(f"<td>{deck.wins}-{deck.losses}-{deck.draws}</td>")
            html_parts.append(f"<td>{deck.win_rate * 100:.1f}%</td>")
            html_parts.append(f"<td>{recent}</td>")
            html_parts.append("</tr>")

        html_parts.append("</tbody>")
        html_parts.append("</table>")
        html_parts.append("</div>")
        html_parts.append("</div>")

        # Matchup predictions heatmap
        html_parts.append('<div class="chart-section">')
        html_parts.append("<h3>Expected Matchup Win Rates</h3>")
        html_parts.append('<p class="chart-description">Predicted win probabilities based on current ELO ratings</p>')
        html_parts.append(create_elo_prediction_heatmap(self.data))
        html_parts.append("</div>")

        # Match history section (collapsible per deck)
        html_parts.append('<div class="history-section">')
        html_parts.append("<h3>Match History</h3>")
        html_parts.append('<div class="history-grid">')

        for deck in ranked_decks[:6]:  # Show top 6 for brevity
            if not deck.history:
                continue

            html_parts.append('<div class="history-card">')
            html_parts.append(f'<div class="history-header">')
            html_parts.append(f'<span class="deck-name">{deck.commander_name}</span>')
            html_parts.append(f'<span class="current-rating">{deck.rating:.0f}</span>')
            html_parts.append("</div>")
            html_parts.append('<div class="history-list">')

            for change in reversed(deck.history[-5:]):  # Last 5 matches
                result_class = {
                    "win": "history-win",
                    "loss": "history-loss",
                    "draw": "history-draw",
                }.get(change.result, "")
                delta_str = f"+{change.delta}" if change.delta > 0 else str(change.delta)

                html_parts.append(f'<div class="history-item {result_class}">')
                html_parts.append(f'<span class="opponent">vs {self._deck_id_to_name(change.opponent)}</span>')
                html_parts.append(f'<span class="delta">{delta_str}</span>')
                html_parts.append("</div>")

            html_parts.append("</div>")
            html_parts.append("</div>")

        html_parts.append("</div>")
        html_parts.append("</div>")

        # Last updated info
        html_parts.append('<div class="run-info">')
        html_parts.append(f"<p><strong>Last Updated:</strong> {self.data.last_updated.strftime('%Y-%m-%d %H:%M')}</p>")
        html_parts.append("</div>")

        return "\n".join(html_parts)

    def _render_metric_card(
        self, title: str, value: str, icon: str, status: str | None = None
    ) -> str:
        """Render a single metric card."""
        status_class = f" status-{status}" if status else ""
        return f"""
        <div class="metric-card{status_class}">
            <div class="metric-icon"><i class="fa-solid {icon}"></i></div>
            <div class="metric-content">
                <div class="metric-value">{value}</div>
                <div class="metric-title">{title}</div>
            </div>
        </div>
        """

    def _get_trend(self, deck: "DeckRating") -> str:
        """Determine rating trend from recent history."""
        if not deck.history:
            return "stable"

        # Look at last 3 matches
        recent_deltas = [h.delta for h in deck.history[-3:]]
        total_delta = sum(recent_deltas)

        if total_delta > 5:
            return "up"
        elif total_delta < -5:
            return "down"
        return "stable"

    def _get_recent_form(self, deck: "DeckRating", n: int = 5) -> str:
        """Get recent form as W/L/D symbols."""
        if not deck.history:
            return "-"

        symbols = {
            "win": '<span class="form-win">W</span>',
            "loss": '<span class="form-loss">L</span>',
            "draw": '<span class="form-draw">D</span>',
        }

        recent = deck.history[-n:]
        return "".join(symbols.get(h.result, "?") for h in recent)

    def _deck_id_to_name(self, deck_id: str) -> str:
        """Convert deck_id to commander name."""
        if deck_id in self.data.ratings:
            return self.data.ratings[deck_id].commander_name or deck_id
        return " ".join(word.title() for word in deck_id.split("_"))
