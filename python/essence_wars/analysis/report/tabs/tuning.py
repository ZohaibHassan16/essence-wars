"""Tuning tab generator for HTML reports.

Displays training curves, experiment comparisons, and convergence analysis.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from ..charts import (
    create_convergence_status,
    create_tuning_comparison,
    create_tuning_curves,
)

if TYPE_CHECKING:
    from ..loaders.tuning import TuningData


class TuningTab:
    """Generator for the Tuning tab."""

    def __init__(self, experiments: list[TuningData]):
        """Initialize with a list of tuning experiments.

        Args:
            experiments: List of TuningData objects (sorted by timestamp desc)
        """
        self.experiments = experiments

    def render(self) -> str:
        """Render the Tuning tab content as HTML."""
        if not self.experiments:
            return '<div class="no-data">No tuning experiments found.</div>'

        html_parts = []

        # Summary metrics
        html_parts.append('<div class="metrics-grid">')
        html_parts.append(
            self._render_metric_card(
                "Experiments",
                str(len(self.experiments)),
                "fa-flask",
            )
        )

        # Calculate converged count
        converged_count = sum(
            1 for exp in self.experiments if exp.get_convergence_info()["converged"]
        )
        html_parts.append(
            self._render_metric_card(
                "Converged",
                f"{converged_count}/{len(self.experiments)}",
                "fa-check-circle",
                status="success" if converged_count == len(self.experiments) else "warning",
            )
        )

        # Best fitness across experiments
        best_exp = max(self.experiments, key=lambda e: e.final_fitness)
        html_parts.append(
            self._render_metric_card(
                "Best Fitness",
                f"{best_exp.final_fitness:.1f}",
                "fa-trophy",
            )
        )

        # Best win rate
        best_wr_exp = max(self.experiments, key=lambda e: e.final_win_rate)
        html_parts.append(
            self._render_metric_card(
                "Best Win Rate",
                f"{best_wr_exp.final_win_rate:.1f}%",
                "fa-percent",
            )
        )
        html_parts.append("</div>")  # End metrics-grid

        # Experiment selector dropdown (if multiple experiments)
        if len(self.experiments) > 1:
            html_parts.append('<div class="experiment-selector">')
            html_parts.append('<label for="exp-select">Select Experiment: </label>')
            html_parts.append('<select id="exp-select" onchange="showExperiment(this.value)">')
            for i, exp in enumerate(self.experiments):
                html_parts.append(f'<option value="{i}">{exp.tag} ({exp.mode})</option>')
            html_parts.append("</select>")
            html_parts.append("</div>")

        # Training curves for each experiment (hidden by default except first)
        html_parts.append('<div class="experiment-curves">')
        for i, exp in enumerate(self.experiments):
            display = "block" if i == 0 else "none"
            html_parts.append(f'<div class="experiment-panel" id="exp-{i}" style="display: {display};">')

            # Experiment info header
            html_parts.append('<div class="experiment-header">')
            html_parts.append(f"<h3>{exp.tag}</h3>")
            html_parts.append('<div class="experiment-meta">')
            html_parts.append(f'<span class="mode-badge">{exp.mode}</span>')
            html_parts.append(f'<span>{exp.total_generations} generations</span>')
            html_parts.append(f"<span>{self._format_duration(exp.total_time_secs)}</span>")
            if exp.version:
                html_parts.append(f'<span class="version">v{exp.version}</span>')
            html_parts.append("</div>")
            html_parts.append("</div>")

            # Training curves chart
            html_parts.append('<div class="chart-card full-width">')
            html_parts.append(create_tuning_curves(exp))
            html_parts.append("</div>")

            # Convergence analysis
            conv_info = exp.get_convergence_info()
            html_parts.append('<div class="convergence-analysis">')
            status_icon = "fa-check" if conv_info["converged"] else "fa-hourglass-half"
            status_class = "converged" if conv_info["converged"] else "in-progress"
            html_parts.append(
                f'<div class="convergence-status {status_class}">'
                f'<i class="fa-solid {status_icon}"></i> '
                f'{conv_info["reason"]}'
                f"</div>"
            )
            html_parts.append("</div>")

            # Final results summary
            html_parts.append('<div class="experiment-results">')
            html_parts.append("<h4>Final Results</h4>")
            html_parts.append('<div class="results-grid">')
            html_parts.append(f'<div class="result-item"><span class="label">Fitness:</span><span class="value">{exp.final_fitness:.1f}</span></div>')
            html_parts.append(f'<div class="result-item"><span class="label">Win Rate:</span><span class="value">{exp.final_win_rate:.1f}%</span></div>')
            html_parts.append(f'<div class="result-item"><span class="label">Final Sigma:</span><span class="value">{exp.final_sigma:.4f}</span></div>')
            html_parts.append("</div>")
            html_parts.append("</div>")

            html_parts.append("</div>")  # End experiment-panel
        html_parts.append("</div>")  # End experiment-curves

        # Comparison section (if multiple experiments)
        if len(self.experiments) > 1:
            html_parts.append('<div class="comparison-section">')
            html_parts.append("<h3>Experiment Comparison</h3>")

            html_parts.append('<div class="charts-row">')

            # Comparison chart
            html_parts.append('<div class="chart-card">')
            html_parts.append(create_tuning_comparison(self.experiments[:6]))  # Limit to 6 for readability
            html_parts.append("</div>")

            # Convergence status chart
            html_parts.append('<div class="chart-card">')
            html_parts.append(create_convergence_status(self.experiments[:6]))
            html_parts.append("</div>")

            html_parts.append("</div>")  # End charts-row

            # Experiments table
            html_parts.append('<div class="experiments-table-container">')
            html_parts.append('<table class="data-table sortable" id="experiments-table">')
            html_parts.append("<thead>")
            html_parts.append("<tr>")
            html_parts.append('<th data-sort="string">Tag</th>')
            html_parts.append('<th data-sort="string">Mode</th>')
            html_parts.append('<th data-sort="number">Gens</th>')
            html_parts.append('<th data-sort="number">Fitness</th>')
            html_parts.append('<th data-sort="number">Win Rate</th>')
            html_parts.append('<th data-sort="number">Sigma</th>')
            html_parts.append('<th data-sort="number">Duration</th>')
            html_parts.append('<th data-sort="string">Status</th>')
            html_parts.append("</tr>")
            html_parts.append("</thead>")
            html_parts.append("<tbody>")

            for exp in self.experiments:
                conv_info = exp.get_convergence_info()
                status = "Converged" if conv_info["converged"] else "In Progress"
                status_class = "status-success" if conv_info["converged"] else "status-warning"

                html_parts.append("<tr>")
                html_parts.append(f'<td class="tag-cell">{exp.tag}</td>')
                html_parts.append(f'<td><span class="mode-badge">{exp.mode}</span></td>')
                html_parts.append(f"<td>{exp.total_generations}</td>")
                html_parts.append(f"<td>{exp.final_fitness:.1f}</td>")
                html_parts.append(f"<td>{exp.final_win_rate:.1f}%</td>")
                html_parts.append(f"<td>{exp.final_sigma:.4f}</td>")
                html_parts.append(f"<td>{self._format_duration(exp.total_time_secs)}</td>")
                html_parts.append(f'<td class="{status_class}">{status}</td>')
                html_parts.append("</tr>")

            html_parts.append("</tbody>")
            html_parts.append("</table>")
            html_parts.append("</div>")

            html_parts.append("</div>")  # End comparison-section

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

    def _format_duration(self, seconds: float) -> str:
        """Format duration in human-readable form."""
        if seconds < 60:
            return f"{seconds:.0f}s"
        elif seconds < 3600:
            minutes = seconds / 60
            return f"{minutes:.1f}m"
        else:
            hours = seconds / 3600
            return f"{hours:.1f}h"
