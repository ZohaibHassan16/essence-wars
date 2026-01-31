"""Research tab generator for HTML reports.

Provides faction-level analysis including:
- Faction vs faction matchup matrix
- Faction win rates
- Game length distribution
- Combat efficiency metrics
"""

from __future__ import annotations

from collections import defaultdict
from typing import TYPE_CHECKING

from ..charts import (
    create_combat_efficiency_chart,
    create_faction_matchup_heatmap,
    create_faction_winrate_bar,
    create_game_length_histogram,
)

if TYPE_CHECKING:
    from ..loaders.validation import ValidationData


class ResearchTab:
    """Generator for the Research tab with faction-level analysis."""

    def __init__(self, data: "ValidationData"):
        self.data = data

    def render(self) -> str:
        """Render the Research tab content as HTML."""
        html_parts = []

        # Compute faction-level stats
        faction_matrix = self._compute_faction_matrix()
        faction_win_rates = self._compute_faction_win_rates()
        game_lengths = self._extract_game_lengths()
        combat_stats = self._compute_combat_stats()

        # Summary metrics row
        html_parts.append('<div class="metrics-grid">')
        for faction, win_rate in sorted(faction_win_rates.items()):
            status = "balanced" if 0.45 <= win_rate <= 0.55 else "warning"
            html_parts.append(
                self._render_metric_card(
                    f"{faction.title()} Win Rate",
                    f"{win_rate * 100:.1f}%",
                    f"fa-chess-{self._faction_icon(faction)}",
                    status=status,
                )
            )
        html_parts.append("</div>")

        # Faction matchup heatmap (full width)
        html_parts.append('<div class="chart-section">')
        html_parts.append("<h3>Faction vs Faction Matchups</h3>")
        html_parts.append(
            '<p class="chart-description">Win rates for row faction against column faction. '
            "Values >50% favor the row faction.</p>"
        )
        html_parts.append('<div class="chart-container">')
        html_parts.append(create_faction_matchup_heatmap(faction_matrix))
        html_parts.append("</div>")
        html_parts.append("</div>")

        # Two-column chart row
        html_parts.append('<div class="charts-row">')

        # Faction win rates bar chart
        html_parts.append('<div class="chart-card">')
        html_parts.append("<h4>Overall Faction Win Rates</h4>")
        html_parts.append(create_faction_winrate_bar(faction_win_rates))
        html_parts.append("</div>")

        # Game length distribution
        html_parts.append('<div class="chart-card">')
        html_parts.append("<h4>Game Length Distribution</h4>")
        html_parts.append(create_game_length_histogram(game_lengths))
        html_parts.append("</div>")

        html_parts.append("</div>")  # End charts-row

        # Combat efficiency chart (full width)
        if combat_stats:
            html_parts.append('<div class="chart-section">')
            html_parts.append("<h3>Combat Efficiency by Faction</h3>")
            html_parts.append(
                '<p class="chart-description">Trade ratio (damage dealt per damage received) '
                "and average face damage per game.</p>"
            )
            html_parts.append('<div class="chart-container">')
            html_parts.append(create_combat_efficiency_chart(combat_stats))
            html_parts.append("</div>")
            html_parts.append("</div>")

        # Detailed matchup tables by faction
        html_parts.append(self._render_faction_matchup_tables())

        return "\n".join(html_parts)

    def _compute_faction_matrix(self) -> dict[str, dict[str, float]]:
        """Compute faction vs faction win rate matrix from matchup data."""
        # Aggregate weighted win rates per faction pair
        counts: dict[str, dict[str, tuple[float, int]]] = defaultdict(
            lambda: defaultdict(lambda: (0.0, 0))
        )

        for matchup in self.data.matchups:
            f1 = matchup.faction1
            f2 = matchup.faction2
            total = matchup.total_games

            # Use win rates weighted by game count
            f1_wr = matchup.faction1_win_rate
            f2_wr = matchup.faction2_win_rate

            # f1 vs f2
            w, g = counts[f1][f2]
            counts[f1][f2] = (w + f1_wr * total, g + total)

            # f2 vs f1
            w, g = counts[f2][f1]
            counts[f2][f1] = (w + f2_wr * total, g + total)

        # Convert to average win rates
        matrix: dict[str, dict[str, float]] = {}
        for f1 in counts:
            matrix[f1] = {}
            for f2 in counts[f1]:
                weighted_sum, games = counts[f1][f2]
                matrix[f1][f2] = (weighted_sum / games) if games > 0 else 0.5

        return matrix

    def _compute_faction_win_rates(self) -> dict[str, float]:
        """Compute overall win rate for each faction."""
        faction_stats: dict[str, tuple[float, int]] = defaultdict(lambda: (0.0, 0))

        for matchup in self.data.matchups:
            f1 = matchup.faction1
            f2 = matchup.faction2
            f1_wr = matchup.faction1_win_rate
            f2_wr = matchup.faction2_win_rate
            total = matchup.total_games

            w, g = faction_stats[f1]
            faction_stats[f1] = (w + f1_wr * total, g + total)

            w, g = faction_stats[f2]
            faction_stats[f2] = (w + f2_wr * total, g + total)

        return {
            f: (weighted_sum / games) if games > 0 else 0.5
            for f, (weighted_sum, games) in faction_stats.items()
        }

    def _extract_game_lengths(self) -> list[float]:
        """Extract game length data from matchups for histogram."""
        lengths: list[float] = []

        for matchup in self.data.matchups:
            avg_turns = matchup.avg_turns or 15.0
            total_games = matchup.total_games

            # Use diagnostics if available for distribution
            diag = matchup.diagnostics or {}
            p10 = diag.get("game_length_p10", avg_turns - 3)
            p50 = diag.get("game_length_p50", avg_turns)
            p90 = diag.get("game_length_p90", avg_turns + 5)

            # Generate approximate distribution
            count = max(1, total_games // 10)
            lengths.extend([p10] * count)
            lengths.extend([p50] * (count * 5))
            lengths.extend([p90] * count)

            # Fill remainder with average
            remainder = total_games - (count * 7)
            if remainder > 0:
                lengths.extend([avg_turns] * remainder)

        # Limit to reasonable size for chart
        return lengths[:1000] if len(lengths) > 1000 else lengths

    def _compute_combat_stats(self) -> dict[str, dict[str, float]]:
        """Compute combat efficiency stats by faction."""
        faction_combat: dict[str, dict[str, list[float]]] = defaultdict(
            lambda: {"trade_ratio": [], "face_damage": []}
        )

        for matchup in self.data.matchups:
            f1 = matchup.faction1
            f2 = matchup.faction2
            diag = matchup.diagnostics or {}

            # Extract combat stats if available
            for faction, prefix in [(f1, "p1"), (f2, "p2")]:
                trade_ratio = diag.get(f"{prefix}_trade_ratio")
                face_damage = diag.get(f"{prefix}_face_damage_avg")

                if trade_ratio is not None:
                    faction_combat[faction]["trade_ratio"].append(trade_ratio)
                if face_damage is not None:
                    faction_combat[faction]["face_damage"].append(face_damage)

        # Compute averages
        result: dict[str, dict[str, float]] = {}
        for faction, stats in faction_combat.items():
            result[faction] = {
                "trade_ratio": (
                    sum(stats["trade_ratio"]) / len(stats["trade_ratio"])
                    if stats["trade_ratio"]
                    else 1.0
                ),
                "face_damage": (
                    sum(stats["face_damage"]) / len(stats["face_damage"])
                    if stats["face_damage"]
                    else 15.0
                ),
            }

        return result if result else {}

    def _render_faction_matchup_tables(self) -> str:
        """Render detailed matchup tables organized by faction."""
        factions = ["argentum", "symbiote", "obsidion"]

        # Group matchups by faction
        faction_matchups: dict[str, list] = {f: [] for f in factions}
        for matchup in self.data.matchups:
            f1 = matchup.faction1.lower() if matchup.faction1 else ""
            if f1 in faction_matchups:
                faction_matchups[f1].append(matchup)

        # Build tabs and tables
        tabs = []
        tables = []

        for i, faction in enumerate(factions):
            active = " active" if i == 0 else ""
            tabs.append(
                f'<button class="tab-btn research-tab{active}" '
                f'data-research-faction="{faction}">{faction.title()}</button>'
            )

            matchups = faction_matchups[faction]
            rows = []
            for m in matchups:
                # Get win rate for faction1 (already calculated)
                total = m.total_games
                wr = m.faction1_win_rate

                # Determine status class
                if 0.45 <= wr <= 0.55:
                    status = "balanced"
                elif 0.40 <= wr <= 0.60:
                    status = "warning"
                else:
                    status = "imbalanced"

                rows.append(
                    f"<tr>"
                    f"<td>{m.deck1_id}</td>"
                    f"<td>{m.deck2_id}</td>"
                    f'<td class="win-rate {status}">{wr * 100:.1f}%</td>'
                    f"<td>{total}</td>"
                    f"<td>{m.avg_turns:.1f}</td>"
                    f"</tr>"
                )

            table_content = "".join(rows) if rows else '<tr><td colspan="5">No matchups</td></tr>'
            tables.append(
                f'<div id="research-table-{faction}" class="research-panel{active}">'
                f"<table class='deck-table'>"
                f"<thead><tr>"
                f"<th>{faction.title()} Deck</th>"
                f"<th>Opponent</th>"
                f"<th>Win Rate</th>"
                f"<th>Games</th>"
                f"<th>Avg Turns</th>"
                f"</tr></thead>"
                f"<tbody>{table_content}</tbody>"
                f"</table></div>"
            )

        return f"""
        <div class="table-section">
            <h3>Detailed Matchup Analysis</h3>
            <div class="research-tabs">
                {"".join(tabs)}
            </div>
            <div class="research-panels">
                {"".join(tables)}
            </div>
        </div>
        """

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

    def _faction_icon(self, faction: str) -> str:
        """Get icon suffix for a faction."""
        icons = {
            "argentum": "knight",
            "symbiote": "pawn",
            "obsidion": "queen",
        }
        return icons.get(faction.lower(), "pawn")
