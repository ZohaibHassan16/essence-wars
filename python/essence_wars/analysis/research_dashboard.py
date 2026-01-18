#!/usr/bin/env python3
"""
Research Dashboard Generator for Essence Wars

Generates an interactive HTML dashboard from validation results,
showing faction matchups, deck performance, P1/P2 analysis, and more.

Usage:
    python research_dashboard.py results.json --output dashboard.html
    python research_dashboard.py --latest  # Uses latest validation run
"""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from string import Template
from typing import Any


def get_dashboard_template() -> str:
    """Return the dashboard HTML template."""
    return '''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Essence Wars Research Dashboard</title>
    <script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
    <style>
        :root {
            --bg-primary: #1a1a2e;
            --bg-secondary: #16213e;
            --bg-card: #0f3460;
            --text-primary: #eee;
            --text-secondary: #aaa;
            --accent-blue: #4da6ff;
            --accent-green: #4dff88;
            --accent-red: #ff6b6b;
            --accent-yellow: #ffd93d;
            --argentum: #7cb9e8;
            --symbiote: #77dd77;
            --obsidion: #9370db;
        }

        * { box-sizing: border-box; margin: 0; padding: 0; }

        body {
            font-family: 'Segoe UI', system-ui, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
        }

        .container {
            max-width: 1600px;
            margin: 0 auto;
            padding: 20px;
        }

        header {
            background: linear-gradient(135deg, var(--bg-secondary), var(--bg-card));
            padding: 30px;
            border-radius: 12px;
            margin-bottom: 30px;
            border: 1px solid rgba(77, 166, 255, 0.3);
        }

        header h1 {
            font-size: 2.5em;
            margin-bottom: 10px;
            background: linear-gradient(90deg, var(--accent-blue), var(--accent-green));
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }

        .meta-info {
            display: flex;
            gap: 30px;
            flex-wrap: wrap;
            color: var(--text-secondary);
            font-size: 0.9em;
        }

        .meta-info span {
            display: flex;
            align-items: center;
            gap: 8px;
        }

        .dashboard-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
            gap: 20px;
        }

        .card {
            background: var(--bg-secondary);
            border-radius: 12px;
            padding: 20px;
            border: 1px solid rgba(255,255,255,0.1);
        }

        .card-full { grid-column: 1 / -1; }

        .card h2 {
            font-size: 1.3em;
            margin-bottom: 15px;
            padding-bottom: 10px;
            border-bottom: 2px solid var(--accent-blue);
            display: flex;
            align-items: center;
            gap: 10px;
        }

        .card h2::before {
            content: '';
            width: 4px;
            height: 20px;
            background: var(--accent-blue);
            border-radius: 2px;
        }

        .chart-container { min-height: 400px; }

        .summary-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-bottom: 20px;
        }

        .summary-card {
            background: var(--bg-card);
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }

        .summary-card .value {
            font-size: 2.5em;
            font-weight: bold;
            margin-bottom: 5px;
        }

        .summary-card .label {
            color: var(--text-secondary);
            font-size: 0.9em;
        }

        .balanced { color: var(--accent-green); }
        .warning { color: var(--accent-yellow); }
        .imbalanced { color: var(--accent-red); }

        table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 15px;
        }

        th, td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid rgba(255,255,255,0.1);
        }

        th { background: var(--bg-card); font-weight: 600; }
        tr:hover { background: rgba(77, 166, 255, 0.1); }
        .win-rate { font-weight: bold; }

        .tabs {
            display: flex;
            gap: 10px;
            margin-bottom: 20px;
            flex-wrap: wrap;
        }

        .tab {
            padding: 10px 20px;
            background: var(--bg-card);
            border: none;
            border-radius: 8px;
            color: var(--text-primary);
            cursor: pointer;
            transition: all 0.2s;
        }

        .tab:hover, .tab.active { background: var(--accent-blue); }
        .tab-content { display: none; }
        .tab-content.active { display: block; }

        footer {
            text-align: center;
            padding: 30px;
            color: var(--text-secondary);
            font-size: 0.9em;
        }

        footer a { color: var(--accent-blue); text-decoration: none; }

        @media (max-width: 768px) {
            .dashboard-grid { grid-template-columns: 1fr; }
            header h1 { font-size: 1.8em; }
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Essence Wars Research Dashboard</h1>
            <div class="meta-info">
                <span>Generated: $generation_time</span>
                <span>Games Analyzed: $total_games</span>
                <span>Matchups: $num_matchups</span>
                <span>Engine Version: $engine_version</span>
            </div>
        </header>

        <div class="summary-grid">
            <div class="summary-card">
                <div class="value $overall_status">$overall_status_text</div>
                <div class="label">Overall Balance</div>
            </div>
            <div class="summary-card">
                <div class="value">$faction_delta%</div>
                <div class="label">Max Faction Delta</div>
            </div>
            <div class="summary-card">
                <div class="value">$p1_win_rate%</div>
                <div class="label">P1 Win Rate</div>
            </div>
            <div class="summary-card">
                <div class="value">$avg_game_length</div>
                <div class="label">Avg Game Length</div>
            </div>
        </div>

        <div class="dashboard-grid">
            <div class="card card-full">
                <h2>Faction Matchup Matrix</h2>
                <p style="color: var(--text-secondary); margin-bottom: 15px;">
                    Win rates for Row Faction vs Column Faction. Values > 50% favor the row faction.
                </p>
                <div id="heatmap-container" class="chart-container"></div>
            </div>

            <div class="card">
                <h2>Faction Win Rates</h2>
                <div id="faction-bars-container" class="chart-container"></div>
            </div>

            <div class="card">
                <h2>Deck Performance Ranking</h2>
                <div id="deck-ranking-container" class="chart-container"></div>
            </div>

            <div class="card">
                <h2>P1/P2 Advantage Analysis</h2>
                <div id="p1p2-container" class="chart-container"></div>
            </div>

            <div class="card">
                <h2>Game Length Distribution</h2>
                <div id="game-length-container" class="chart-container"></div>
            </div>

            <div class="card card-full">
                <h2>Combat Efficiency by Faction</h2>
                <div id="combat-efficiency-container" class="chart-container"></div>
            </div>

            <div class="card card-full">
                <h2>Detailed Matchup Analysis</h2>
                <div class="tabs">
                    $faction_tabs
                </div>
                <div id="matchup-details">
                    $matchup_tables
                </div>
            </div>
        </div>

        <footer>
            <p>
                <strong>Essence Wars</strong> - A deterministic card game engine for ML/AI research
            </p>
        </footer>
    </div>

    <script>
        const dashboardData = $dashboard_data_json;

        const colors = {
            argentum: '#7cb9e8',
            symbiote: '#77dd77',
            obsidion: '#9370db',
            neutral: '#888888'
        };

        const plotlyLayout = {
            paper_bgcolor: 'rgba(0,0,0,0)',
            plot_bgcolor: 'rgba(22,33,62,0.8)',
            font: { color: '#eee', family: 'Segoe UI, system-ui, sans-serif' },
            margin: { t: 40, r: 20, b: 60, l: 60 }
        };

        const plotlyConfig = { responsive: true, displayModeBar: true };

        function renderHeatmap() {
            const matrix = dashboardData.matchup_matrix;
            const factions = ['argentum', 'symbiote', 'obsidion'];
            const labels = ['Argentum', 'Symbiote', 'Obsidion'];

            const z = factions.map(f1 => factions.map(f2 => matrix[f1]?.[f2] ?? 50));

            const annotations = [];
            for (let i = 0; i < 3; i++) {
                for (let j = 0; j < 3; j++) {
                    const val = z[i][j];
                    annotations.push({
                        x: labels[j], y: labels[i],
                        text: val.toFixed(1) + '%',
                        showarrow: false,
                        font: { color: val > 55 || val < 45 ? '#fff' : '#333', size: 16 }
                    });
                }
            }

            Plotly.newPlot('heatmap-container', [{
                type: 'heatmap', z: z, x: labels, y: labels,
                colorscale: [[0, '#ff6b6b'], [0.45, '#ff6b6b'], [0.45, '#ffd93d'],
                            [0.55, '#ffd93d'], [0.55, '#4dff88'], [1, '#4dff88']],
                zmin: 30, zmax: 70, showscale: true,
                colorbar: { title: 'Win Rate %', ticksuffix: '%' }
            }], { ...plotlyLayout, annotations, xaxis: { title: 'Opponent' }, yaxis: { title: 'Your Faction', autorange: 'reversed' }}, plotlyConfig);
        }

        function renderFactionBars() {
            const factions = Object.keys(dashboardData.faction_win_rates);
            const winRates = Object.values(dashboardData.faction_win_rates);
            const barColors = factions.map(f => colors[f.toLowerCase()] || '#888');

            Plotly.newPlot('faction-bars-container', [{
                type: 'bar', x: factions.map(f => f.charAt(0).toUpperCase() + f.slice(1)),
                y: winRates, marker: { color: barColors },
                text: winRates.map(v => v.toFixed(1) + '%'), textposition: 'outside'
            }], {
                ...plotlyLayout, yaxis: { title: 'Win Rate %', range: [0, 70], gridcolor: 'rgba(255,255,255,0.1)' },
                shapes: [{ type: 'line', x0: -0.5, x1: 2.5, y0: 50, y1: 50, line: { color: '#ffd93d', width: 2, dash: 'dash' }},
                        { type: 'rect', x0: -0.5, x1: 2.5, y0: 45, y1: 55, fillcolor: 'rgba(77, 255, 136, 0.1)', line: { width: 0 }}]
            }, plotlyConfig);
        }

        function renderDeckRanking() {
            const decks = dashboardData.deck_rankings;
            const sorted = [...decks].sort((a, b) => b.win_rate - a.win_rate);
            const barColors = sorted.map(d => colors[d.faction.toLowerCase()] || '#888');

            Plotly.newPlot('deck-ranking-container', [{
                type: 'bar', y: sorted.map(d => d.deck_id.replace(/_/g, ' ')),
                x: sorted.map(d => d.win_rate), orientation: 'h',
                marker: { color: barColors },
                text: sorted.map(d => d.win_rate.toFixed(1) + '%'), textposition: 'outside'
            }], {
                ...plotlyLayout, xaxis: { title: 'Win Rate %', range: [0, 75], gridcolor: 'rgba(255,255,255,0.1)' },
                yaxis: { autorange: 'reversed' }, height: Math.max(400, sorted.length * 35),
                shapes: [{ type: 'line', x0: 50, x1: 50, y0: -0.5, y1: sorted.length - 0.5, line: { color: '#ffd93d', width: 2, dash: 'dash' }}]
            }, plotlyConfig);
        }

        function renderP1P2Analysis() {
            const p1Data = dashboardData.p1_p2_analysis;
            Plotly.newPlot('p1p2-container', [{
                type: 'indicator', mode: 'gauge+number+delta',
                value: p1Data.overall_p1_win_rate, delta: { reference: 50, valueformat: '.1f' },
                gauge: {
                    axis: { range: [35, 65], ticksuffix: '%' }, bar: { color: '#4da6ff' },
                    bgcolor: 'rgba(22,33,62,0.8)', borderwidth: 2, bordercolor: 'rgba(255,255,255,0.3)',
                    steps: [{ range: [35, 45], color: 'rgba(255, 107, 107, 0.3)' },
                            { range: [45, 55], color: 'rgba(77, 255, 136, 0.2)' },
                            { range: [55, 65], color: 'rgba(255, 107, 107, 0.3)' }],
                    threshold: { line: { color: '#ffd93d', width: 4 }, thickness: 0.75, value: 50 }
                },
                title: { text: 'Player 1 Win Rate', font: { size: 16 } },
                number: { suffix: '%', font: { size: 36 } }
            }], { ...plotlyLayout, height: 350 }, plotlyConfig);
        }

        function renderGameLength() {
            const lengths = dashboardData.game_lengths;
            Plotly.newPlot('game-length-container', [{
                type: 'histogram', x: lengths, marker: { color: '#4da6ff' }, nbinsx: 20
            }], {
                ...plotlyLayout,
                xaxis: { title: 'Game Length (Turns)', gridcolor: 'rgba(255,255,255,0.1)' },
                yaxis: { title: 'Number of Games', gridcolor: 'rgba(255,255,255,0.1)' }, bargap: 0.05
            }, plotlyConfig);
        }

        function renderCombatEfficiency() {
            const factions = ['argentum', 'symbiote', 'obsidion'];
            const labels = ['Argentum', 'Symbiote', 'Obsidion'];
            const tradeRatios = factions.map(f => dashboardData.combat_stats[f]?.trade_ratio ?? 1);
            const faceDamage = factions.map(f => dashboardData.combat_stats[f]?.face_damage ?? 0);

            Plotly.newPlot('combat-efficiency-container', [
                { type: 'bar', name: 'Trade Ratio', x: labels, y: tradeRatios, marker: { color: '#4da6ff' }},
                { type: 'bar', name: 'Avg Face Damage', x: labels, y: faceDamage, marker: { color: '#4dff88' }, yaxis: 'y2' }
            ], {
                ...plotlyLayout, barmode: 'group',
                yaxis: { title: 'Trade Ratio', gridcolor: 'rgba(255,255,255,0.1)' },
                yaxis2: { title: 'Face Damage', overlaying: 'y', side: 'right', showgrid: false },
                legend: { orientation: 'h', y: -0.15 }
            }, plotlyConfig);
        }

        function switchTab(faction) {
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            document.querySelector('.tab[data-faction="' + faction + '"]').classList.add('active');
            document.getElementById('table-' + faction).classList.add('active');
        }

        document.addEventListener('DOMContentLoaded', function() {
            renderHeatmap();
            renderFactionBars();
            renderDeckRanking();
            renderP1P2Analysis();
            renderGameLength();
            renderCombatEfficiency();
            document.querySelectorAll('.tab').forEach(tab => {
                tab.addEventListener('click', () => switchTab(tab.dataset.faction));
            });
        });
    </script>
</body>
</html>
'''


def load_validation_results(path: Path) -> dict[str, Any]:
    """Load validation results from JSON file."""
    with open(path) as f:
        return json.load(f)


def find_latest_validation() -> Path | None:
    """Find the most recent validation results file."""
    validation_dir = Path("experiments/validation")
    if not validation_dir.exists():
        return None
    results_files = list(validation_dir.glob("*/results.json"))
    if not results_files:
        return None
    return max(results_files, key=lambda p: p.stat().st_mtime)


def compute_matchup_matrix(matchups: list[dict]) -> dict[str, dict[str, float]]:
    """Compute faction vs faction win rate matrix."""
    matrix: dict[str, dict[str, float]] = {}
    counts: dict[str, dict[str, tuple[int, int]]] = {}

    for matchup in matchups:
        f1 = matchup["faction1"]
        f2 = matchup["faction2"]

        for f in [f1, f2]:
            if f not in matrix:
                matrix[f] = {}
                counts[f] = {}

        f1_wins = matchup.get("faction1_total_wins", 0)
        f2_wins = matchup.get("faction2_total_wins", 0)
        total = matchup.get("total_games", f1_wins + f2_wins)

        if f2 not in counts[f1]:
            counts[f1][f2] = (0, 0)
        w, g = counts[f1][f2]
        counts[f1][f2] = (w + f1_wins, g + total)

        if f1 not in counts[f2]:
            counts[f2][f1] = (0, 0)
        w, g = counts[f2][f1]
        counts[f2][f1] = (w + f2_wins, g + total)

    for f1 in counts:
        for f2 in counts[f1]:
            wins, games = counts[f1][f2]
            matrix[f1][f2] = (wins / games * 100) if games > 0 else 50.0

    return matrix


def compute_faction_win_rates(matchups: list[dict]) -> dict[str, float]:
    """Compute overall win rate for each faction."""
    faction_stats: dict[str, tuple[int, int]] = {}

    for matchup in matchups:
        f1 = matchup["faction1"]
        f2 = matchup["faction2"]
        f1_wins = matchup.get("faction1_total_wins", 0)
        f2_wins = matchup.get("faction2_total_wins", 0)
        total = matchup.get("total_games", f1_wins + f2_wins)

        if f1 not in faction_stats:
            faction_stats[f1] = (0, 0)
        w, g = faction_stats[f1]
        faction_stats[f1] = (w + f1_wins, g + total)

        if f2 not in faction_stats:
            faction_stats[f2] = (0, 0)
        w, g = faction_stats[f2]
        faction_stats[f2] = (w + f2_wins, g + total)

    return {f: (wins / games * 100) if games > 0 else 50.0 for f, (wins, games) in faction_stats.items()}


def compute_deck_rankings(matchups: list[dict]) -> list[dict]:
    """Compute win rate for each deck."""
    deck_stats: dict[str, dict[str, Any]] = {}

    for matchup in matchups:
        d1, d2 = matchup["deck1_id"], matchup["deck2_id"]
        f1, f2 = matchup["faction1"], matchup["faction2"]
        f1_wins = matchup.get("faction1_total_wins", 0)
        f2_wins = matchup.get("faction2_total_wins", 0)
        total = matchup.get("total_games", f1_wins + f2_wins)

        if d1 not in deck_stats:
            deck_stats[d1] = {"wins": 0, "games": 0, "faction": f1}
        deck_stats[d1]["wins"] += f1_wins
        deck_stats[d1]["games"] += total

        if d2 not in deck_stats:
            deck_stats[d2] = {"wins": 0, "games": 0, "faction": f2}
        deck_stats[d2]["wins"] += f2_wins
        deck_stats[d2]["games"] += total

    return [{"deck_id": k, "faction": v["faction"], "win_rate": (v["wins"]/v["games"]*100) if v["games"] > 0 else 50, "games": v["games"]} for k, v in deck_stats.items()]


def compute_p1_p2_analysis(matchups: list[dict]) -> dict[str, Any]:
    """Analyze P1 vs P2 advantage."""
    total_p1_wins = 0
    total_games = 0

    for matchup in matchups:
        f1_p1_wins = matchup.get("f1_as_p1_wins", 0)
        f1_p1_games = matchup.get("f1_as_p1_games", 0)
        f1_p2_wins = matchup.get("f1_as_p2_wins", 0)
        f1_p2_games = matchup.get("f1_as_p2_games", 0)

        p1_wins = f1_p1_wins + (f1_p2_games - f1_p2_wins)
        games = f1_p1_games + f1_p2_games
        total_p1_wins += p1_wins
        total_games += games

    return {"overall_p1_win_rate": (total_p1_wins / total_games * 100) if total_games > 0 else 50.0, "total_games": total_games}


def extract_game_lengths(matchups: list[dict]) -> list[float]:
    """Extract game length distribution."""
    lengths = []
    for matchup in matchups:
        avg = matchup.get("avg_turns", 15)
        diag = matchup.get("diagnostics", {})
        p10 = diag.get("game_length_p10", avg - 3)
        p50 = diag.get("game_length_p50", avg)
        p90 = diag.get("game_length_p90", avg + 5)

        games = matchup.get("total_games", 100)
        for _ in range(games // 10):
            lengths.append(p10)
        for _ in range(games // 2):
            lengths.append(p50)
        for _ in range(games // 10):
            lengths.append(p90)
        for _ in range(games - (games // 10 * 2 + games // 2)):
            lengths.append(avg)

    return lengths[:1000]


def compute_combat_stats(matchups: list[dict]) -> dict[str, dict[str, float]]:
    """Compute combat efficiency stats by faction."""
    faction_combat: dict[str, dict[str, list[float]]] = {}

    for matchup in matchups:
        f1, f2 = matchup["faction1"], matchup["faction2"]
        diag = matchup.get("diagnostics", {})

        for f, prefix in [(f1, "p1"), (f2, "p2")]:
            if f not in faction_combat:
                faction_combat[f] = {"trade_ratio": [], "face_damage": [], "board_advantage": []}
            tr = diag.get(f"{prefix}_trade_ratio", 1.0)
            fd = diag.get(f"{prefix}_face_damage_avg", 15.0)
            if tr:
                faction_combat[f]["trade_ratio"].append(tr)
            if fd:
                faction_combat[f]["face_damage"].append(fd)

    return {f: {"trade_ratio": sum(s["trade_ratio"])/len(s["trade_ratio"]) if s["trade_ratio"] else 1.0, "face_damage": sum(s["face_damage"])/len(s["face_damage"]) if s["face_damage"] else 15.0} for f, s in faction_combat.items()}


def generate_matchup_tables(matchups: list[dict]) -> tuple[str, str]:
    """Generate HTML tabs and tables for matchup details."""
    factions = ["argentum", "symbiote", "obsidion"]
    faction_matchups: dict[str, list[dict]] = {f: [] for f in factions}
    for matchup in matchups:
        f1 = matchup["faction1"]
        if f1 in faction_matchups:
            faction_matchups[f1].append(matchup)

    tabs = []
    for i, f in enumerate(factions):
        active = "active" if i == 0 else ""
        tabs.append(f'<button class="tab {active}" data-faction="{f}">{f.title()}</button>')

    tables = []
    for i, f in enumerate(factions):
        active = "active" if i == 0 else ""
        rows = []
        for m in faction_matchups[f]:
            wr = m.get("faction1_win_rate", 50)
            status = "balanced" if 45 <= wr <= 55 else ("warning" if 40 <= wr <= 60 else "imbalanced")
            rows.append(f'<tr><td>{m["deck1_id"]}</td><td>{m["deck2_id"]}</td><td class="win-rate {status}">{wr:.1f}%</td><td>{m.get("total_games", 0)}</td><td>{m.get("avg_turns", 0):.1f}</td></tr>')
        tables.append(f'<div id="table-{f}" class="tab-content {active}"><table><thead><tr><th>{f.title()} Deck</th><th>Opponent</th><th>Win Rate</th><th>Games</th><th>Avg Turns</th></tr></thead><tbody>{"".join(rows)}</tbody></table></div>')

    return "\n".join(tabs), "\n".join(tables)


def generate_dashboard(results: dict[str, Any], output_path: Path) -> None:
    """Generate the HTML dashboard."""
    matchups = results.get("matchups", [])
    summary = results.get("summary", {})

    matchup_matrix = compute_matchup_matrix(matchups)
    faction_win_rates = compute_faction_win_rates(matchups)
    deck_rankings = compute_deck_rankings(matchups)
    p1_p2_analysis = compute_p1_p2_analysis(matchups)
    game_lengths = extract_game_lengths(matchups)
    combat_stats = compute_combat_stats(matchups)
    faction_tabs, matchup_tables = generate_matchup_tables(matchups)

    dashboard_data = {
        "matchup_matrix": matchup_matrix,
        "faction_win_rates": faction_win_rates,
        "deck_rankings": deck_rankings,
        "p1_p2_analysis": p1_p2_analysis,
        "game_lengths": game_lengths,
        "combat_stats": combat_stats
    }

    total_games = sum(m.get("total_games", 0) for m in matchups)
    avg_game_length = sum(m.get("avg_turns", 15) * m.get("total_games", 1) for m in matchups) / max(total_games, 1)

    faction_rates = list(faction_win_rates.values())
    faction_delta = max(faction_rates) - min(faction_rates) if faction_rates else 0

    overall_status = summary.get("overall_status", "balanced")
    status_map = {"balanced": "Balanced", "warning": "Warning", "imbalanced": "Imbalanced"}

    version_info = results.get("version", {})

    template = Template(get_dashboard_template())
    html = template.safe_substitute(
        generation_time=datetime.now().strftime("%Y-%m-%d %H:%M"),
        total_games=f"{total_games:,}",
        num_matchups=len(matchups),
        engine_version=version_info.get("version", "unknown"),
        overall_status=overall_status,
        overall_status_text=status_map.get(overall_status, "Unknown"),
        faction_delta=f"{faction_delta:.1f}",
        p1_win_rate=f"{p1_p2_analysis['overall_p1_win_rate']:.1f}",
        avg_game_length=f"{avg_game_length:.1f}",
        faction_tabs=faction_tabs,
        matchup_tables=matchup_tables,
        dashboard_data_json=json.dumps(dashboard_data)
    )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(html)
    print(f"Dashboard generated: {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Generate research dashboard")
    parser.add_argument("input", nargs="?", help="Path to results.json")
    parser.add_argument("--latest", action="store_true", help="Use latest validation")
    parser.add_argument("--output", "-o", type=Path, default=Path("research_dashboard.html"))
    args = parser.parse_args()

    if args.latest:
        input_path = find_latest_validation()
        if not input_path:
            print("No validation results found")
            sys.exit(1)
        print(f"Using latest validation: {input_path}")
    elif args.input:
        input_path = Path(args.input)
    else:
        parser.print_help()
        sys.exit(1)

    if not input_path.exists():
        print(f"File not found: {input_path}")
        sys.exit(1)

    results = load_validation_results(input_path)
    generate_dashboard(results, args.output)


if __name__ == "__main__":
    main()
