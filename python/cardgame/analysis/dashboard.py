#!/usr/bin/env python3
"""
Interactive HTML dashboard generator for MCTS training analysis.
Uses Plotly for interactive charts and Jinja2 for templating.
"""

import logging
from pathlib import Path
from datetime import datetime
from typing import Optional

import pandas as pd
import plotly.graph_objects as go
import plotly.express as px
from plotly.subplots import make_subplots
from jinja2 import Template

logger = logging.getLogger(__name__)


class MCTSDashboard:
    """Generate interactive HTML dashboard for MCTS training analysis."""

    def __init__(self, df: pd.DataFrame, summary_df: pd.DataFrame):
        """
        Initialize dashboard generator.

        Args:
            df: Full generation-by-generation data
            summary_df: Summary statistics per experiment
        """
        self.df = df
        self.summary_df = summary_df
        self.figures = []

    def create_all_plots(self) -> list[go.Figure]:
        """Create all interactive plots."""
        self.figures = []

        # Core training curves
        self.figures.append(self._plot_fitness_evolution())
        self.figures.append(self._plot_winrate_evolution())
        self.figures.append(self._plot_sigma_adaptation())

        # Performance analysis
        self.figures.append(self._plot_final_performance())
        self.figures.append(self._plot_convergence_speed())

        # Mode comparison (if multiple modes exist)
        if self.df["mode"].nunique() > 1:
            self.figures.append(self._plot_mode_comparison())

        # Time analysis
        self.figures.append(self._plot_training_efficiency())

        # Exploration dynamics
        self.figures.append(self._plot_exploration_trajectory())

        logger.info(f"Generated {len(self.figures)} interactive plots")
        return self.figures

    def _plot_fitness_evolution(self) -> go.Figure:
        """Plot fitness evolution across all experiments."""
        fig = go.Figure()

        for exp_id in self.df["experiment_id"].unique():
            exp_data = self.df[self.df["experiment_id"] == exp_id]
            tag = exp_data["tag"].iloc[0]
            mode = exp_data["mode"].iloc[0]

            fig.add_trace(
                go.Scatter(
                    x=exp_data["generation"],
                    y=exp_data["best_fitness"],
                    mode="lines",
                    name=f"{tag} ({mode})",
                    hovertemplate="<b>%{fullData.name}</b><br>"
                    + "Gen: %{x}<br>"
                    + "Fitness: %{y:.2f}<br>"
                    + "<extra></extra>",
                )
            )

        fig.update_layout(
            title="Fitness Evolution Across All Experiments",
            xaxis_title="Generation",
            yaxis_title="Best Fitness",
            hovermode="x unified",
            legend=dict(yanchor="top", y=0.99, xanchor="left", x=0.01),
            template="plotly_white",
            height=600,
        )

        return fig

    def _plot_winrate_evolution(self) -> go.Figure:
        """Plot win rate evolution."""
        fig = go.Figure()

        for exp_id in self.df["experiment_id"].unique():
            exp_data = self.df[self.df["experiment_id"] == exp_id]
            tag = exp_data["tag"].iloc[0]

            fig.add_trace(
                go.Scatter(
                    x=exp_data["generation"],
                    y=exp_data["best_winrate"],
                    mode="lines",
                    name=tag,
                    hovertemplate="<b>%{fullData.name}</b><br>"
                    + "Gen: %{x}<br>"
                    + "Win Rate: %{y:.1f}%<br>"
                    + "<extra></extra>",
                )
            )

        # Add 50% baseline
        fig.add_hline(
            y=50,
            line_dash="dash",
            line_color="red",
            opacity=0.5,
            annotation_text="50% Baseline",
        )

        fig.update_layout(
            title="Win Rate Evolution (vs Opponents)",
            xaxis_title="Generation",
            yaxis_title="Win Rate (%)",
            hovermode="x unified",
            template="plotly_white",
            height=600,
        )

        return fig

    def _plot_sigma_adaptation(self) -> go.Figure:
        """Plot CMA-ES sigma adaptation."""
        fig = go.Figure()

        for exp_id in self.df["experiment_id"].unique():
            exp_data = self.df[self.df["experiment_id"] == exp_id]
            tag = exp_data["tag"].iloc[0]

            fig.add_trace(
                go.Scatter(
                    x=exp_data["generation"],
                    y=exp_data["sigma"],
                    mode="lines",
                    name=tag,
                    hovertemplate="<b>%{fullData.name}</b><br>"
                    + "Gen: %{x}<br>"
                    + "Sigma: %{y:.4f}<br>"
                    + "<extra></extra>",
                )
            )

        fig.update_layout(
            title="CMA-ES Step Size Adaptation (Sigma)",
            xaxis_title="Generation",
            yaxis_title="Sigma (Exploration Parameter)",
            hovermode="x unified",
            template="plotly_white",
            height=600,
        )

        return fig

    def _plot_final_performance(self) -> go.Figure:
        """Bar chart of final performance by experiment."""
        summary = self.summary_df.sort_values("final_fitness", ascending=False)

        fig = go.Figure(
            data=[
                go.Bar(
                    x=summary["tag"],
                    y=summary["final_fitness"],
                    text=summary["final_fitness"].round(2),
                    textposition="auto",
                    marker=dict(
                        color=summary["final_fitness"],
                        colorscale="Viridis",
                        showscale=True,
                        colorbar=dict(title="Fitness"),
                    ),
                    hovertemplate="<b>%{x}</b><br>"
                    + "Fitness: %{y:.2f}<br>"
                    + "Win Rate: %{customdata:.1f}%<br>"
                    + "<extra></extra>",
                    customdata=summary["final_winrate"],
                )
            ]
        )

        fig.update_layout(
            title="Final Performance Comparison",
            xaxis_title="Experiment",
            yaxis_title="Final Fitness",
            template="plotly_white",
            height=500,
            xaxis_tickangle=-45,
        )

        return fig

    def _plot_convergence_speed(self) -> go.Figure:
        """Plot convergence speed (generations to reach 90% of final)."""
        convergence_data = []

        for exp_id in self.df["experiment_id"].unique():
            exp_data = self.df[self.df["experiment_id"] == exp_id].sort_values("generation")
            tag = exp_data["tag"].iloc[0]
            final_fitness = exp_data["final_fitness"].iloc[0]

            # Find generation where 90% of final fitness reached
            threshold = 0.9 * final_fitness
            converged = exp_data[exp_data["best_fitness"] >= threshold]

            if not converged.empty:
                gen_90 = converged["generation"].iloc[0]
                convergence_data.append({"tag": tag, "generations_to_90": gen_90})

        if convergence_data:
            conv_df = pd.DataFrame(convergence_data).sort_values("generations_to_90")

            fig = go.Figure(
                data=[
                    go.Bar(
                        x=conv_df["tag"],
                        y=conv_df["generations_to_90"],
                        text=conv_df["generations_to_90"],
                        textposition="auto",
                        marker=dict(color="lightblue"),
                    )
                ]
            )

            fig.update_layout(
                title="Convergence Speed (Generations to 90% of Final Fitness)",
                xaxis_title="Experiment",
                yaxis_title="Generations",
                template="plotly_white",
                height=500,
                xaxis_tickangle=-45,
            )

            return fig

        return go.Figure()  # Return empty figure if no data

    def _plot_mode_comparison(self) -> go.Figure:
        """Box plot comparing modes."""
        fig = go.Figure()

        for mode in self.df["mode"].unique():
            mode_data = self.summary_df[self.summary_df["mode"] == mode]

            fig.add_trace(
                go.Box(
                    y=mode_data["final_fitness"],
                    name=mode,
                    boxmean="sd",
                    hovertemplate="<b>%{fullData.name}</b><br>"
                    + "Fitness: %{y:.2f}<br>"
                    + "<extra></extra>",
                )
            )

        fig.update_layout(
            title="Performance Distribution by Training Mode",
            yaxis_title="Final Fitness",
            template="plotly_white",
            height=500,
        )

        return fig

    def _plot_training_efficiency(self) -> go.Figure:
        """Plot fitness per hour for each experiment."""
        summary = self.summary_df.copy()
        summary["fitness_per_hour"] = summary["final_fitness"] / (
            summary["total_time_min"] / 60.0
        )
        summary = summary.sort_values("fitness_per_hour", ascending=False)

        fig = go.Figure(
            data=[
                go.Bar(
                    x=summary["tag"],
                    y=summary["fitness_per_hour"],
                    text=summary["fitness_per_hour"].round(1),
                    textposition="auto",
                    marker=dict(color="coral"),
                )
            ]
        )

        fig.update_layout(
            title="Training Efficiency (Fitness per Hour)",
            xaxis_title="Experiment",
            yaxis_title="Fitness / Hour",
            template="plotly_white",
            height=500,
            xaxis_tickangle=-45,
        )

        return fig

    def _plot_exploration_trajectory(self) -> go.Figure:
        """3D plot of exploration trajectory (generation, sigma, fitness)."""
        # Limit to top 5 experiments for clarity
        top_experiments = (
            self.summary_df.nlargest(5, "final_fitness")["experiment_id"].tolist()
        )

        fig = go.Figure()

        for exp_id in top_experiments:
            exp_data = self.df[self.df["experiment_id"] == exp_id]
            tag = exp_data["tag"].iloc[0]

            fig.add_trace(
                go.Scatter3d(
                    x=exp_data["generation"],
                    y=exp_data["sigma"],
                    z=exp_data["best_fitness"],
                    mode="lines+markers",
                    name=tag,
                    marker=dict(size=4),
                    line=dict(width=2),
                    hovertemplate="<b>%{fullData.name}</b><br>"
                    + "Gen: %{x}<br>"
                    + "Sigma: %{y:.4f}<br>"
                    + "Fitness: %{z:.2f}<br>"
                    + "<extra></extra>",
                )
            )

        fig.update_layout(
            title="Exploration Trajectory (Top 5 Experiments)",
            scene=dict(
                xaxis_title="Generation",
                yaxis_title="Sigma",
                zaxis_title="Fitness",
            ),
            template="plotly_white",
            height=700,
        )

        return fig

    def generate_html(
        self, output_path: Path, title: str = "MCTS Training Analysis Dashboard"
    ) -> None:
        """
        Generate complete HTML dashboard.

        Args:
            output_path: Path to save HTML file
            title: Dashboard title
        """
        # Create all plots
        self.create_all_plots()

        # Convert plots to HTML divs
        plot_htmls = []
        for i, fig in enumerate(self.figures):
            html_div = fig.to_html(full_html=False, include_plotlyjs=False, div_id=f"plot_{i}")
            plot_htmls.append(html_div)

        # Generate summary statistics HTML
        summary_html = self._generate_summary_html()

        # HTML template
        template_str = """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }}</title>
    <script src="https://cdn.plot.ly/plotly-2.27.0.min.js" charset="utf-8"></script>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 15px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            overflow: hidden;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }
        .header h1 {
            font-size: 2.5em;
            margin-bottom: 10px;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }
        .header p {
            font-size: 1.1em;
            opacity: 0.9;
        }
        .summary {
            padding: 40px;
            background: #f8f9fa;
            border-bottom: 3px solid #e9ecef;
        }
        .summary h2 {
            color: #495057;
            margin-bottom: 20px;
            font-size: 1.8em;
        }
        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-top: 20px;
        }
        .stat-card {
            background: white;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            border-left: 4px solid #667eea;
        }
        .stat-card h3 {
            color: #6c757d;
            font-size: 0.9em;
            text-transform: uppercase;
            margin-bottom: 10px;
        }
        .stat-card .value {
            color: #212529;
            font-size: 2em;
            font-weight: bold;
        }
        .plots {
            padding: 40px;
        }
        .plot-container {
            margin-bottom: 50px;
            background: #f8f9fa;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.05);
        }
        .footer {
            background: #212529;
            color: white;
            padding: 30px;
            text-align: center;
        }
        .footer p {
            opacity: 0.7;
        }
        table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
            background: white;
            border-radius: 10px;
            overflow: hidden;
        }
        th, td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #e9ecef;
        }
        th {
            background: #667eea;
            color: white;
            font-weight: 600;
        }
        tr:hover {
            background: #f8f9fa;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{{ title }}</h1>
            <p>Generated: {{ timestamp }}</p>
            <p>{{ num_experiments }} experiments analyzed</p>
        </div>

        <div class="summary">
            {{ summary_html|safe }}
        </div>

        <div class="plots">
            {% for plot_html in plot_htmls %}
            <div class="plot-container">
                {{ plot_html|safe }}
            </div>
            {% endfor %}
        </div>

        <div class="footer">
            <p>Essence Wars - MCTS Training Analysis</p>
            <p>AI Card Game Engine - Rust + Python</p>
        </div>
    </div>
</body>
</html>
        """

        template = Template(template_str)
        html_content = template.render(
            title=title,
            timestamp=datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            num_experiments=len(self.summary_df),
            summary_html=summary_html,
            plot_htmls=plot_htmls,
        )

        # Write to file
        output_path.write_text(html_content)
        logger.info(f"Dashboard saved to: {output_path}")

    def _generate_summary_html(self) -> str:
        """Generate HTML for summary statistics section."""
        total_experiments = len(self.summary_df)
        best_fitness = self.summary_df["final_fitness"].max()
        avg_fitness = self.summary_df["final_fitness"].mean()
        best_winrate = self.summary_df["final_winrate"].max()
        total_hours = self.summary_df["total_time_min"].sum() / 60.0

        # Top 5 experiments table
        top5 = self.summary_df.nlargest(5, "final_fitness")
        table_rows = ""
        for _, row in top5.iterrows():
            table_rows += f"""
            <tr>
                <td>{row['tag']}</td>
                <td>{row['mode']}</td>
                <td>{row['final_fitness']:.2f}</td>
                <td>{row['final_winrate']:.1f}%</td>
                <td>{row['num_generations']}</td>
                <td>{row['total_time_min']:.1f} min</td>
            </tr>
            """

        html = f"""
        <h2>📊 Summary Statistics</h2>
        <div class="stats-grid">
            <div class="stat-card">
                <h3>Total Experiments</h3>
                <div class="value">{total_experiments}</div>
            </div>
            <div class="stat-card">
                <h3>Best Fitness</h3>
                <div class="value">{best_fitness:.2f}</div>
            </div>
            <div class="stat-card">
                <h3>Average Fitness</h3>
                <div class="value">{avg_fitness:.2f}</div>
            </div>
            <div class="stat-card">
                <h3>Best Win Rate</h3>
                <div class="value">{best_winrate:.1f}%</div>
            </div>
            <div class="stat-card">
                <h3>Total Training Time</h3>
                <div class="value">{total_hours:.1f}h</div>
            </div>
        </div>

        <h3 style="margin-top: 40px; margin-bottom: 20px;">🏆 Top 5 Experiments</h3>
        <table>
            <thead>
                <tr>
                    <th>Experiment</th>
                    <th>Mode</th>
                    <th>Fitness</th>
                    <th>Win Rate</th>
                    <th>Generations</th>
                    <th>Duration</th>
                </tr>
            </thead>
            <tbody>
                {table_rows}
            </tbody>
        </table>
        """

        return html
