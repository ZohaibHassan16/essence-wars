"""Overview tab generator for HTML reports.

Provides high-level summary metrics and health indicators.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from ..charts import (
    create_faction_distribution,
    create_health_gauge,
    create_p1_p2_chart,
)

if TYPE_CHECKING:
    from ..loaders.validation import ValidationData


class OverviewTab:
    """Generator for the Overview tab."""

    def __init__(self, data: "ValidationData"):
        self.data = data

    def render(self) -> str:
        """Render the Overview tab content as HTML."""
        health_score = self.data.get_health_score()
        outliers = self.data.get_outlier_decks()

        # Build the HTML content
        html_parts = []

        # Header row with key metrics
        html_parts.append('<div class="metrics-grid">')

        # Health gauge
        html_parts.append('<div class="metric-card wide">')
        html_parts.append(create_health_gauge(health_score))
        html_parts.append("</div>")

        # Key metrics cards
        html_parts.append(self._render_metric_card("Total Games", str(self.data.total_games), "fa-gamepad"))
        html_parts.append(self._render_metric_card("Decks Tested", str(len(self.data.deck_stats)), "fa-layer-group"))
        html_parts.append(
            self._render_metric_card(
                "P1/P2 Balance",
                f"{self.data.p1_win_rate * 100:.1f}%",
                "fa-scale-balanced",
                status=self.data.p1_status,
            )
        )
        html_parts.append(
            self._render_metric_card(
                "Outlier Decks",
                str(len(outliers)),
                "fa-triangle-exclamation",
                status="warning" if outliers else "success",
            )
        )

        html_parts.append("</div>")  # End metrics-grid

        # Charts row
        html_parts.append('<div class="charts-row">')

        # P1/P2 chart
        html_parts.append('<div class="chart-card">')
        html_parts.append(create_p1_p2_chart(self.data))
        html_parts.append("</div>")

        # Faction distribution
        html_parts.append('<div class="chart-card">')
        html_parts.append(create_faction_distribution(self.data.deck_stats))
        html_parts.append("</div>")

        html_parts.append("</div>")  # End charts-row

        # Warnings section
        if outliers or self.data.summary.get("warnings"):
            html_parts.append('<div class="warnings-section">')
            html_parts.append("<h3>Issues Requiring Attention</h3>")

            # General warnings
            for warning in self.data.summary.get("warnings", []):
                html_parts.append(f'<div class="warning-item">{warning}</div>')

            # Outlier decks
            if outliers:
                html_parts.append("<h4>Outlier Decks</h4>")
                html_parts.append('<div class="outlier-list">')
                for deck in outliers:
                    indicator = "high" if deck.win_rate > 0.6 else "low"
                    html_parts.append(
                        f'<div class="outlier-item {indicator}">'
                        f'<span class="deck-name">{deck.commander_name}</span>'
                        f'<span class="faction-badge faction-{deck.faction}">{deck.faction.title()}</span>'
                        f'<span class="win-rate">{deck.win_rate * 100:.1f}%</span>'
                        f"</div>"
                    )
                html_parts.append("</div>")

            html_parts.append("</div>")  # End warnings-section

        # Run info
        html_parts.append('<div class="run-info">')
        html_parts.append("<h3>Run Information</h3>")
        html_parts.append(f"<p><strong>Run ID:</strong> {self.data.run_id}</p>")
        html_parts.append(f"<p><strong>Timestamp:</strong> {self.data.timestamp.strftime('%Y-%m-%d %H:%M')}</p>")
        html_parts.append(f"<p><strong>Version:</strong> {self.data.version} ({self.data.git_hash})</p>")

        config = self.data.config
        if config:
            html_parts.append(f"<p><strong>Games per Matchup:</strong> {config.get('games_per_matchup', 'N/A')}</p>")
            if "mcts_simulations" in config:
                html_parts.append(f"<p><strong>MCTS Simulations:</strong> {config['mcts_simulations']}</p>")
            if "matchup_filter" in config:
                html_parts.append(f"<p><strong>Matchup Filter:</strong> {config['matchup_filter']}</p>")

        html_parts.append("</div>")  # End run-info

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
