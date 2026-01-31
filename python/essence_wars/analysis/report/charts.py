"""Plotly chart builders for HTML reports.

Provides reusable chart building functions with consistent styling.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots

if TYPE_CHECKING:
    from .loaders.validation import DeckStats, ValidationData
    from .loaders.tuning import TuningData

# Faction colors matching the game's aesthetic
FACTION_COLORS = {
    "argentum": "#D4AF37",  # Gold
    "symbiote": "#7FFF00",  # Lime green
    "obsidion": "#00FFFF",  # Cyan
    "neutral": "#B87333",  # Copper
}

# Dark theme colors
DARK_THEME = {
    "bg_primary": "#1a1a2e",
    "bg_secondary": "#16213e",
    "bg_card": "#0f3460",
    "text_primary": "#eaeaea",
    "text_secondary": "#a0a0a0",
    "accent": "#e94560",
    "success": "#00d26a",
    "warning": "#ffc107",
    "danger": "#dc3545",
    "grid": "#2a2a4e",
}


def create_health_gauge(score: float) -> str:
    """Create a health score gauge chart.

    Args:
        score: Health score from 0-100

    Returns:
        HTML string with embedded Plotly chart
    """
    # Determine color based on score
    if score >= 75:
        bar_color = DARK_THEME["success"]
    elif score >= 50:
        bar_color = DARK_THEME["warning"]
    else:
        bar_color = DARK_THEME["danger"]

    fig = go.Figure(
        go.Indicator(
            mode="gauge+number",
            value=score,
            domain={"x": [0, 1], "y": [0, 1]},
            title={"text": "Balance Health", "font": {"color": DARK_THEME["text_primary"]}},
            number={"suffix": "", "font": {"color": DARK_THEME["text_primary"], "size": 48}},
            gauge={
                "axis": {
                    "range": [0, 100],
                    "tickwidth": 1,
                    "tickcolor": DARK_THEME["text_secondary"],
                    "tickfont": {"color": DARK_THEME["text_secondary"]},
                },
                "bar": {"color": bar_color},
                "bgcolor": DARK_THEME["bg_card"],
                "borderwidth": 2,
                "bordercolor": DARK_THEME["text_secondary"],
                "steps": [
                    {"range": [0, 50], "color": "rgba(220, 53, 69, 0.2)"},
                    {"range": [50, 75], "color": "rgba(255, 193, 7, 0.2)"},
                    {"range": [75, 100], "color": "rgba(0, 210, 106, 0.2)"},
                ],
                "threshold": {
                    "line": {"color": DARK_THEME["accent"], "width": 4},
                    "thickness": 0.75,
                    "value": score,
                },
            },
        )
    )

    fig.update_layout(
        paper_bgcolor=DARK_THEME["bg_primary"],
        font={"color": DARK_THEME["text_primary"]},
        height=250,
        margin={"t": 50, "b": 20, "l": 30, "r": 30},
    )

    return fig.to_html(full_html=False, include_plotlyjs=False)


def create_deck_winrate_bar(deck_stats: list["DeckStats"]) -> str:
    """Create a horizontal bar chart of deck win rates.

    Args:
        deck_stats: List of DeckStats objects

    Returns:
        HTML string with embedded Plotly chart
    """
    # Sort by win rate descending
    sorted_stats = sorted(deck_stats, key=lambda d: d.win_rate, reverse=True)

    names = [f"{d.commander_name}" for d in sorted_stats]
    win_rates = [d.win_rate * 100 for d in sorted_stats]
    ci_lower = [(d.win_rate - d.win_rate_ci_lower) * 100 for d in sorted_stats]
    ci_upper = [(d.win_rate_ci_upper - d.win_rate) * 100 for d in sorted_stats]
    colors = [FACTION_COLORS.get(d.faction, DARK_THEME["text_secondary"]) for d in sorted_stats]

    fig = go.Figure()

    fig.add_trace(
        go.Bar(
            y=names,
            x=win_rates,
            orientation="h",
            marker_color=colors,
            error_x={"type": "data", "symmetric": False, "array": ci_upper, "arrayminus": ci_lower},
            hovertemplate="<b>%{y}</b><br>Win Rate: %{x:.1f}%<extra></extra>",
        )
    )

    # Add balanced zone (40-60%)
    fig.add_vrect(
        x0=40,
        x1=60,
        fillcolor="rgba(0, 210, 106, 0.1)",
        line_width=0,
        annotation_text="Balanced",
        annotation_position="top left",
        annotation_font_color=DARK_THEME["success"],
    )

    # Add 50% reference line
    fig.add_vline(x=50, line_dash="dash", line_color=DARK_THEME["text_secondary"], line_width=1)

    fig.update_layout(
        title={"text": "Deck Win Rates", "font": {"color": DARK_THEME["text_primary"]}},
        paper_bgcolor=DARK_THEME["bg_primary"],
        plot_bgcolor=DARK_THEME["bg_secondary"],
        font={"color": DARK_THEME["text_primary"]},
        xaxis={
            "title": "Win Rate (%)",
            "range": [0, 100],
            "gridcolor": DARK_THEME["grid"],
            "zeroline": False,
        },
        yaxis={"title": None, "gridcolor": DARK_THEME["grid"], "autorange": "reversed"},
        height=max(300, len(deck_stats) * 40),
        margin={"t": 50, "b": 50, "l": 150, "r": 30},
        showlegend=False,
    )

    return fig.to_html(full_html=False, include_plotlyjs=False)


def create_matchup_heatmap(data: "ValidationData") -> str:
    """Create an interactive matchup heatmap.

    Args:
        data: ValidationData with matchup information

    Returns:
        HTML string with embedded Plotly chart
    """
    matrix = data.get_matchup_matrix()

    # Get commander names for labels
    deck_to_commander = {d.deck_id: d.commander_name for d in data.deck_stats}

    # Sort by faction then by commander name
    deck_to_faction = {d.deck_id: d.faction for d in data.deck_stats}
    faction_order = ["argentum", "symbiote", "obsidion", "neutral"]

    def sort_key(deck_id):
        faction = deck_to_faction.get(deck_id, "neutral")
        faction_idx = faction_order.index(faction) if faction in faction_order else 999
        return (faction_idx, deck_to_commander.get(deck_id, deck_id))

    sorted_decks = sorted(matrix.index, key=sort_key)
    matrix = matrix.loc[sorted_decks, sorted_decks]

    labels = [deck_to_commander.get(d, d) for d in sorted_decks]

    # Create hover text
    hover_text = []
    for i, d1 in enumerate(sorted_decks):
        row = []
        for j, d2 in enumerate(sorted_decks):
            val = matrix.loc[d1, d2]
            if pd.isna(val):
                row.append("")
            else:
                row.append(f"{labels[i]} vs {labels[j]}<br>Win Rate: {val*100:.1f}%")
        hover_text.append(row)

    # Convert NaN to None for proper display
    z_values = matrix.values.tolist()
    for i in range(len(z_values)):
        for j in range(len(z_values[i])):
            if pd.isna(z_values[i][j]):
                z_values[i][j] = None

    fig = go.Figure(
        data=go.Heatmap(
            z=z_values,
            x=labels,
            y=labels,
            colorscale=[
                [0.0, "#dc3545"],  # Red for 0%
                [0.4, "#ffc107"],  # Yellow for 40%
                [0.5, "#ffffff"],  # White for 50%
                [0.6, "#28a745"],  # Green for 60%
                [1.0, "#00d26a"],  # Bright green for 100%
            ],
            zmin=0,
            zmax=1,
            text=hover_text,
            hoverinfo="text",
            colorbar={
                "title": {"text": "Win Rate", "font": {"color": DARK_THEME["text_primary"]}},
                "tickvals": [0, 0.25, 0.5, 0.75, 1.0],
                "ticktext": ["0%", "25%", "50%", "75%", "100%"],
                "tickfont": {"color": DARK_THEME["text_primary"]},
            },
        )
    )

    fig.update_layout(
        title={"text": "Matchup Matrix", "font": {"color": DARK_THEME["text_primary"]}},
        paper_bgcolor=DARK_THEME["bg_primary"],
        plot_bgcolor=DARK_THEME["bg_secondary"],
        font={"color": DARK_THEME["text_primary"]},
        xaxis={"title": "Opponent", "tickangle": 45},
        yaxis={"title": "Deck", "autorange": "reversed"},
        height=max(400, len(labels) * 45),
        margin={"t": 50, "b": 120, "l": 150, "r": 30},
    )

    return fig.to_html(full_html=False, include_plotlyjs=False)


def create_p1_p2_chart(data: "ValidationData") -> str:
    """Create a P1/P2 win rate visualization.

    Args:
        data: ValidationData with P1/P2 diagnostics

    Returns:
        HTML string with embedded Plotly chart
    """
    p1_diag = data.summary.get("p1_p2_diagnostics", {})
    p1_rate = p1_diag.get("overall_p1_win_rate", 0.5) * 100
    p1_ci_lower = p1_diag.get("overall_p1_ci_lower", 0.45) * 100
    p1_ci_upper = p1_diag.get("overall_p1_ci_upper", 0.55) * 100

    fig = go.Figure()

    # P1 bar with error bars
    fig.add_trace(
        go.Bar(
            x=["Player 1 Win Rate"],
            y=[p1_rate],
            marker_color=FACTION_COLORS["argentum"],
            error_y={
                "type": "data",
                "symmetric": False,
                "array": [p1_ci_upper - p1_rate],
                "arrayminus": [p1_rate - p1_ci_lower],
                "color": DARK_THEME["text_primary"],
            },
            hovertemplate=f"P1 Win Rate: {p1_rate:.1f}%<br>95% CI: [{p1_ci_lower:.1f}%, {p1_ci_upper:.1f}%]<extra></extra>",
        )
    )

    # Add 50% reference line
    fig.add_hline(y=50, line_dash="dash", line_color=DARK_THEME["text_secondary"], line_width=2)

    # Add balanced zone
    fig.add_hrect(
        y0=45,
        y1=55,
        fillcolor="rgba(0, 210, 106, 0.1)",
        line_width=0,
    )

    fig.update_layout(
        title={"text": "P1/P2 Balance", "font": {"color": DARK_THEME["text_primary"]}},
        paper_bgcolor=DARK_THEME["bg_primary"],
        plot_bgcolor=DARK_THEME["bg_secondary"],
        font={"color": DARK_THEME["text_primary"]},
        yaxis={
            "title": "Win Rate (%)",
            "range": [30, 70],
            "gridcolor": DARK_THEME["grid"],
        },
        xaxis={"title": None},
        height=250,
        margin={"t": 50, "b": 30, "l": 60, "r": 30},
        showlegend=False,
    )

    return fig.to_html(full_html=False, include_plotlyjs=False)


def create_faction_distribution(deck_stats: list["DeckStats"]) -> str:
    """Create a faction distribution donut chart.

    Args:
        deck_stats: List of DeckStats objects

    Returns:
        HTML string with embedded Plotly chart
    """
    # Count decks by faction and calculate average win rate
    faction_data = {}
    for d in deck_stats:
        if d.faction not in faction_data:
            faction_data[d.faction] = {"count": 0, "total_wr": 0}
        faction_data[d.faction]["count"] += 1
        faction_data[d.faction]["total_wr"] += d.win_rate

    factions = list(faction_data.keys())
    counts = [faction_data[f]["count"] for f in factions]
    avg_wr = [faction_data[f]["total_wr"] / faction_data[f]["count"] * 100 for f in factions]
    colors = [FACTION_COLORS.get(f, DARK_THEME["text_secondary"]) for f in factions]

    fig = go.Figure(
        data=[
            go.Pie(
                labels=[f.title() for f in factions],
                values=counts,
                hole=0.5,
                marker_colors=colors,
                textinfo="label+percent",
                textposition="outside",
                hovertemplate="<b>%{label}</b><br>Decks: %{value}<br>Avg Win Rate: %{customdata:.1f}%<extra></extra>",
                customdata=avg_wr,
            )
        ]
    )

    fig.update_layout(
        title={"text": "Faction Distribution", "font": {"color": DARK_THEME["text_primary"]}},
        paper_bgcolor=DARK_THEME["bg_primary"],
        font={"color": DARK_THEME["text_primary"]},
        height=300,
        margin={"t": 50, "b": 30, "l": 30, "r": 30},
        showlegend=False,
        annotations=[
            {
                "text": f"{len(deck_stats)}<br>Decks",
                "x": 0.5,
                "y": 0.5,
                "font_size": 16,
                "font_color": DARK_THEME["text_primary"],
                "showarrow": False,
            }
        ],
    )

    return fig.to_html(full_html=False, include_plotlyjs=False)


def create_tuning_curves(tuning_data: "TuningData") -> str:
    """Create a multi-axis chart showing fitness, win rate, and sigma over generations.

    Args:
        tuning_data: TuningData object with training curves

    Returns:
        HTML string with embedded Plotly chart
    """

    df = tuning_data.generations_df

    # Create figure with secondary y-axis
    fig = make_subplots(
        rows=2,
        cols=1,
        shared_xaxes=True,
        vertical_spacing=0.08,
        subplot_titles=("Fitness & Win Rate", "Step Size (Sigma)"),
        row_heights=[0.7, 0.3],
    )

    # Fitness trace
    fig.add_trace(
        go.Scatter(
            x=df["generation"],
            y=df["fitness"],
            name="Fitness",
            line={"color": DARK_THEME["success"], "width": 2},
            mode="lines",
            hovertemplate="Gen %{x}<br>Fitness: %{y:.1f}<extra></extra>",
        ),
        row=1,
        col=1,
    )

    # Win rate trace
    fig.add_trace(
        go.Scatter(
            x=df["generation"],
            y=df["win_rate"],
            name="Win Rate",
            line={"color": FACTION_COLORS["argentum"], "width": 2},
            mode="lines",
            hovertemplate="Gen %{x}<br>Win Rate: %{y:.1f}%<extra></extra>",
        ),
        row=1,
        col=1,
    )

    # Sigma trace (lower subplot)
    fig.add_trace(
        go.Scatter(
            x=df["generation"],
            y=df["sigma"],
            name="Sigma",
            line={"color": FACTION_COLORS["obsidion"], "width": 2},
            fill="tozeroy",
            fillcolor="rgba(0, 255, 255, 0.1)",
            mode="lines",
            hovertemplate="Gen %{x}<br>Sigma: %{y:.4f}<extra></extra>",
        ),
        row=2,
        col=1,
    )

    fig.update_layout(
        title={
            "text": f"Training Curves: {tuning_data.tag}",
            "font": {"color": DARK_THEME["text_primary"]},
        },
        paper_bgcolor=DARK_THEME["bg_primary"],
        plot_bgcolor=DARK_THEME["bg_secondary"],
        font={"color": DARK_THEME["text_primary"]},
        height=450,
        margin={"t": 70, "b": 50, "l": 60, "r": 30},
        legend={
            "orientation": "h",
            "yanchor": "bottom",
            "y": 1.02,
            "xanchor": "right",
            "x": 1,
        },
    )

    # Update axes styling
    fig.update_xaxes(
        title_text="Generation",
        gridcolor=DARK_THEME["grid"],
        row=2,
        col=1,
    )
    fig.update_yaxes(
        title_text="Score",
        gridcolor=DARK_THEME["grid"],
        row=1,
        col=1,
    )
    fig.update_yaxes(
        title_text="Sigma",
        gridcolor=DARK_THEME["grid"],
        row=2,
        col=1,
    )

    # Style subplot titles
    for annotation in fig["layout"]["annotations"]:
        annotation["font"] = {"color": DARK_THEME["text_primary"]}

    return fig.to_html(full_html=False, include_plotlyjs=False)


def create_tuning_comparison(tuning_experiments: list["TuningData"]) -> str:
    """Create a comparison chart of multiple tuning experiments.

    Args:
        tuning_experiments: List of TuningData objects to compare

    Returns:
        HTML string with embedded Plotly chart
    """

    fig = go.Figure()

    # Color palette for different experiments
    colors = [
        FACTION_COLORS["argentum"],
        FACTION_COLORS["symbiote"],
        FACTION_COLORS["obsidion"],
        DARK_THEME["accent"],
        DARK_THEME["success"],
        DARK_THEME["warning"],
    ]

    for i, data in enumerate(tuning_experiments):
        color = colors[i % len(colors)]
        df = data.generations_df

        fig.add_trace(
            go.Scatter(
                x=df["generation"],
                y=df["fitness"],
                name=data.tag[:25],  # Truncate long tags
                line={"color": color, "width": 2},
                mode="lines",
                hovertemplate=f"{data.tag}<br>Gen %{{x}}<br>Fitness: %{{y:.1f}}<extra></extra>",
            )
        )

    fig.update_layout(
        title={
            "text": "Experiment Comparison (Fitness)",
            "font": {"color": DARK_THEME["text_primary"]},
        },
        paper_bgcolor=DARK_THEME["bg_primary"],
        plot_bgcolor=DARK_THEME["bg_secondary"],
        font={"color": DARK_THEME["text_primary"]},
        xaxis={
            "title": "Generation",
            "gridcolor": DARK_THEME["grid"],
        },
        yaxis={
            "title": "Fitness",
            "gridcolor": DARK_THEME["grid"],
        },
        height=350,
        margin={"t": 50, "b": 50, "l": 60, "r": 30},
        legend={
            "orientation": "v",
            "yanchor": "top",
            "y": 1,
            "xanchor": "left",
            "x": 1.02,
        },
    )

    return fig.to_html(full_html=False, include_plotlyjs=False)


def create_convergence_status(tuning_experiments: list["TuningData"]) -> str:
    """Create a status summary chart for experiment convergence.

    Args:
        tuning_experiments: List of TuningData objects

    Returns:
        HTML string with embedded Plotly chart
    """

    names = []
    fitness_values = []
    colors = []

    for data in tuning_experiments:
        names.append(data.tag[:20])
        fitness_values.append(data.final_fitness)
        conv_info = data.get_convergence_info()
        if conv_info["converged"]:
            colors.append(DARK_THEME["success"])
        else:
            colors.append(DARK_THEME["warning"])

    fig = go.Figure(
        data=[
            go.Bar(
                x=names,
                y=fitness_values,
                marker_color=colors,
                hovertemplate="<b>%{x}</b><br>Fitness: %{y:.1f}<extra></extra>",
            )
        ]
    )

    fig.update_layout(
        title={
            "text": "Final Fitness by Experiment",
            "font": {"color": DARK_THEME["text_primary"]},
        },
        paper_bgcolor=DARK_THEME["bg_primary"],
        plot_bgcolor=DARK_THEME["bg_secondary"],
        font={"color": DARK_THEME["text_primary"]},
        xaxis={
            "title": None,
            "tickangle": 45,
        },
        yaxis={
            "title": "Fitness",
            "gridcolor": DARK_THEME["grid"],
        },
        height=300,
        margin={"t": 50, "b": 100, "l": 60, "r": 30},
        showlegend=False,
    )

    return fig.to_html(full_html=False, include_plotlyjs=False)
