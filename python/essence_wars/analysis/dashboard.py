#!/usr/bin/env python3
"""
Interactive HTML dashboard generator for MCTS training analysis.
Loads data from CSV files dynamically for efficiency and flexibility.
"""

import logging
from datetime import datetime
from pathlib import Path

import pandas as pd
import plotly.graph_objects as go
from jinja2 import Template

logger = logging.getLogger(__name__)


class MCTSDashboard:
    """Generate lightweight HTML dashboard that loads data from CSV files."""

    def __init__(self, df: pd.DataFrame, summary_df: pd.DataFrame):
        """
        Initialize dashboard generator.

        Args:
            df: Full generation-by-generation data
            summary_df: Summary statistics per experiment
        """
        self.df = df
        self.summary_df = summary_df

    def generate_html(
        self, output_path: Path, title: str = "MCTS Training Analysis Dashboard"
    ) -> None:
        """
        Generate HTML dashboard that loads data from CSV files.

        Args:
            output_path: Path to save HTML file
            title: Dashboard title
        """
        # Generate summary statistics
        summary_stats = self._generate_summary_stats()

        # HTML template with CSV loading using XMLHttpRequest for file:// compatibility
        template_str = '''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }}</title>
    <script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/PapaParse/5.4.1/papaparse.min.js"></script>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 1600px;
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
        .header h1 { font-size: 2.5em; margin-bottom: 10px; text-shadow: 2px 2px 4px rgba(0,0,0,0.3); }
        .header p { font-size: 1.1em; opacity: 0.9; }
        .controls {
            padding: 30px 40px;
            background: #f8f9fa;
            border-bottom: 2px solid #e9ecef;
            display: flex;
            gap: 20px;
            flex-wrap: wrap;
            align-items: center;
        }
        .control-group { display: flex; flex-direction: column; gap: 5px; }
        .control-group label { font-size: 0.85em; font-weight: 600; color: #495057; text-transform: uppercase; }
        select, input { padding: 8px 12px; border: 2px solid #dee2e6; border-radius: 6px; font-size: 14px; background: white; }
        select:focus, input:focus { outline: none; border-color: #667eea; }
        button {
            padding: 10px 20px;
            background: #667eea;
            color: white;
            border: none;
            border-radius: 6px;
            font-weight: 600;
            cursor: pointer;
            transition: background 0.3s;
        }
        button:hover { background: #5568d3; }
        .summary { padding: 40px; background: #f8f9fa; border-bottom: 3px solid #e9ecef; }
        .summary h2 { color: #495057; margin-bottom: 20px; font-size: 1.8em; }
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
        .stat-card h3 { color: #6c757d; font-size: 0.9em; text-transform: uppercase; margin-bottom: 10px; }
        .stat-card .value { color: #212529; font-size: 2em; font-weight: bold; }
        .plots { padding: 40px; }
        .plot-container {
            margin-bottom: 50px;
            background: #f8f9fa;
            padding: 20px;
            border-radius: 10px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.05);
        }
        .plot-title { font-size: 1.3em; font-weight: 600; color: #212529; margin-bottom: 15px; }
        .loading { text-align: center; padding: 40px; color: #6c757d; }
        .spinner {
            border: 4px solid #f3f3f3;
            border-top: 4px solid #667eea;
            border-radius: 50%;
            width: 40px;
            height: 40px;
            animation: spin 1s linear infinite;
            margin: 20px auto;
        }
        @keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
        .footer { background: #212529; color: white; padding: 30px; text-align: center; }
        .footer p { opacity: 0.7; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; background: white; border-radius: 10px; overflow: hidden; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #e9ecef; }
        th { background: #667eea; color: white; font-weight: 600; }
        tr:hover { background: #f8f9fa; }
        .experiment-badge { display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 0.85em; font-weight: 600; }
        .badge-generalist { background: #e3f2fd; color: #1976d2; }
        .badge-specialist { background: #f3e5f5; color: #7b1fa2; }
        .badge-faction { background: #fff3e0; color: #e65100; }
        .error { background: #f8d7da; color: #721c24; padding: 20px; margin: 20px; border-radius: 8px; border-left: 4px solid #f5c6cb; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{{ title }}</h1>
            <p>Generated: {{ timestamp }}</p>
            <p>{{ num_experiments }} experiments • Data loaded from CSV</p>
        </div>

        <div class="controls">
            <div class="control-group">
                <label for="modeFilter">Training Mode</label>
                <select id="modeFilter">
                    <option value="all">All Modes</option>
                    <option value="generalist">Generalist</option>
                    <option value="faction-specialist">Faction Specialist</option>
                    <option value="specialist">Specialist</option>
                    <option value="multi-opponent">Multi-Opponent</option>
                </select>
            </div>
            <div class="control-group">
                <label for="experimentFilter">Experiment</label>
                <select id="experimentFilter">
                    <option value="all">All Experiments</option>
                </select>
            </div>
            <div class="control-group">
                <label for="minGenerations">Min Generations</label>
                <input type="number" id="minGenerations" value="0" min="0" step="10">
            </div>
            <button onclick="applyFilters()">Apply Filters</button>
            <button onclick="resetFilters()">Reset</button>
            <button onclick="exportVisibleData()">Export Filtered Data</button>
        </div>

        <div class="summary">
            <h2>📊 Summary Statistics</h2>
            <div class="stats-grid">
                <div class="stat-card">
                    <h3>Total Experiments</h3>
                    <div class="value" id="stat-total">{{ summary_stats.total_experiments }}</div>
                </div>
                <div class="stat-card">
                    <h3>Best Fitness</h3>
                    <div class="value" id="stat-best-fitness">{{ summary_stats.best_fitness }}</div>
                </div>
                <div class="stat-card">
                    <h3>Average Fitness</h3>
                    <div class="value" id="stat-avg-fitness">{{ summary_stats.avg_fitness }}</div>
                </div>
                <div class="stat-card">
                    <h3>Best Win Rate</h3>
                    <div class="value" id="stat-best-winrate">{{ summary_stats.best_winrate }}%</div>
                </div>
                <div class="stat-card">
                    <h3>Total Training Time</h3>
                    <div class="value" id="stat-total-time">{{ summary_stats.total_hours }}h</div>
                </div>
            </div>

            <h3 style="margin-top: 40px; margin-bottom: 20px;">🏆 Top 5 Experiments</h3>
            <table id="top-experiments-table">
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
                <tbody id="top-experiments-body">
                </tbody>
            </table>
        </div>

        <div class="plots">
            <div class="loading" id="loading">
                <div class="spinner"></div>
                <p>Loading experiment data from CSV...</p>
            </div>
            
            <div id="plots-container" style="display: none;">
                <div class="plot-container">
                    <div class="plot-title">Fitness Evolution Across All Experiments</div>
                    <div id="plot-fitness" style="height: 600px;"></div>
                </div>
                
                <div class="plot-container">
                    <div class="plot-title">Win Rate Evolution (vs Opponents)</div>
                    <div id="plot-winrate" style="height: 600px;"></div>
                </div>
                
                <div class="plot-container">
                    <div class="plot-title">CMA-ES Step Size Adaptation (Sigma)</div>
                    <div id="plot-sigma" style="height: 600px;"></div>
                </div>
                
                <div class="plot-container">
                    <div class="plot-title">Final Performance Comparison</div>
                    <div id="plot-performance" style="height: 500px;"></div>
                </div>
                
                <div class="plot-container">
                    <div class="plot-title">Training Efficiency (Fitness per Hour)</div>
                    <div id="plot-efficiency" style="height: 500px;"></div>
                </div>
            </div>
        </div>

        <div class="footer">
            <p>Essence Wars - MCTS Training Analysis</p>
            <p>AI Card Game Engine - Rust + Python</p>
            <p style="margin-top: 10px; font-size: 0.9em;">Dashboard loads data dynamically from CSV files</p>
        </div>
    </div>

    <script>
        let fullData = [];
        let summaryData = [];
        let filteredData = [];

        // Load CSV using XMLHttpRequest (works with file:// protocol)
        function loadCSV(filename) {
            return new Promise((resolve, reject) => {
                const xhr = new XMLHttpRequest();
                xhr.open('GET', filename, true);
                xhr.onload = function() {
                    if (xhr.status === 200) {
                        resolve(xhr.responseText);
                    } else {
                        reject(new Error(`Failed to load ${filename}: ${xhr.status}`));
                    }
                };
                xhr.onerror = () => reject(new Error(`Network error loading ${filename}`));
                xhr.send();
            });
        }

        async function loadData() {
            try {
                console.log('Loading CSV files...');
                const [fullCsv, summaryCsv] = await Promise.all([
                    loadCSV('aggregated_data.csv'),
                    loadCSV('summary.csv')
                ]);
                console.log(`Loaded CSVs: ${fullCsv.length} bytes, ${summaryCsv.length} bytes`);

                fullData = Papa.parse(fullCsv, { header: true, dynamicTyping: true }).data;
                summaryData = Papa.parse(summaryCsv, { header: true, dynamicTyping: true }).data;
                console.log(`Parsed: ${fullData.length} rows, ${summaryData.length} summary rows`);
                
                fullData = fullData.filter(row => row.experiment_id);
                summaryData = summaryData.filter(row => row.experiment_id);
                console.log(`Filtered: ${fullData.length} data points, ${summaryData.length} experiments`);
                
                if (fullData.length === 0) {
                    throw new Error('No valid data found in CSV files');
                }
                
                populateExperimentFilter();
                filteredData = fullData;
                console.log('Rendering plots...');
                renderPlots();
                updateTopExperimentsTable();
                
                document.getElementById('loading').style.display = 'none';
                document.getElementById('plots-container').style.display = 'block';
                
                console.log(`✓ Dashboard loaded successfully: ${fullData.length} data points from ${summaryData.length} experiments`);
            } catch (error) {
                console.error('Error loading data:', error);
                document.getElementById('loading').innerHTML = 
                    '<div class="error"><strong>Error loading data</strong><br>' + 
                    'Make sure the CSV files (aggregated_data.csv and summary.csv) are in the same directory as this HTML file.<br>' +
                    'Error details: ' + error.message + '<br><br>' +
                    'Check browser console (F12) for more details.</div>';
            }
        }

        function populateExperimentFilter() {
            const select = document.getElementById('experimentFilter');
            const experiments = [...new Set(summaryData.map(row => row.tag))].sort();
            experiments.forEach(exp => {
                const option = document.createElement('option');
                option.value = exp;
                option.textContent = exp;
                select.appendChild(option);
            });
        }

        function applyFilters() {
            const modeFilter = document.getElementById('modeFilter').value;
            const expFilter = document.getElementById('experimentFilter').value;
            const minGens = parseInt(document.getElementById('minGenerations').value) || 0;
            
            let filteredSummary = summaryData;
            if (modeFilter !== 'all') filteredSummary = filteredSummary.filter(row => row.mode === modeFilter);
            if (expFilter !== 'all') filteredSummary = filteredSummary.filter(row => row.tag === expFilter);
            if (minGens > 0) filteredSummary = filteredSummary.filter(row => row.num_generations >= minGens);
            
            const matchingExpIds = new Set(filteredSummary.map(row => row.experiment_id));
            filteredData = fullData.filter(row => matchingExpIds.has(row.experiment_id));
            
            renderPlots();
            updateTopExperimentsTable();
            updateStats(filteredSummary);
        }

        function resetFilters() {
            document.getElementById('modeFilter').value = 'all';
            document.getElementById('experimentFilter').value = 'all';
            document.getElementById('minGenerations').value = '0';
            applyFilters();
        }

        function updateStats(summaryData) {
            const count = summaryData.length;
            const bestFitness = Math.max(...summaryData.map(row => row.final_fitness));
            const avgFitness = summaryData.reduce((sum, row) => sum + row.final_fitness, 0) / count;
            const bestWinrate = Math.max(...summaryData.map(row => row.final_winrate));
            const totalTime = summaryData.reduce((sum, row) => sum + row.total_time_min, 0) / 60;
            
            document.getElementById('stat-total').textContent = count;
            document.getElementById('stat-best-fitness').textContent = bestFitness.toFixed(2);
            document.getElementById('stat-avg-fitness').textContent = avgFitness.toFixed(2);
            document.getElementById('stat-best-winrate').textContent = bestWinrate.toFixed(1) + '%';
            document.getElementById('stat-total-time').textContent = totalTime.toFixed(1) + 'h';
        }

        function updateTopExperimentsTable() {
            const tbody = document.getElementById('top-experiments-body');
            const top5 = summaryData
                .filter(row => filteredData.some(d => d.experiment_id === row.experiment_id))
                .sort((a, b) => b.final_fitness - a.final_fitness)
                .slice(0, 5);
            
            tbody.innerHTML = top5.map(row => `
                <tr>
                    <td>${row.tag}</td>
                    <td><span class="experiment-badge badge-${row.mode.split('-')[0]}">${row.mode}</span></td>
                    <td>${row.final_fitness.toFixed(2)}</td>
                    <td>${row.final_winrate.toFixed(1)}%</td>
                    <td>${row.num_generations}</td>
                    <td>${row.total_time_min.toFixed(1)} min</td>
                </tr>
            `).join('');
        }

        function renderPlots() {
            renderFitnessPlot();
            renderWinratePlot();
            renderSigmaPlot();
            renderPerformanceBar();
            renderEfficiencyBar();
        }

        function renderFitnessPlot() {
            const experiments = [...new Set(filteredData.map(row => row.experiment_id))];
            const traces = experiments.map(expId => {
                const expData = filteredData.filter(row => row.experiment_id === expId);
                const tag = expData[0].tag;
                const mode = expData[0].mode;
                return {
                    x: expData.map(row => row.generation),
                    y: expData.map(row => row.best_fitness),
                    mode: 'lines',
                    name: `${tag} (${mode})`,
                    hovertemplate: '<b>%{fullData.name}</b><br>Gen: %{x}<br>Fitness: %{y:.2f}<extra></extra>'
                };
            });
            
            const layout = {
                xaxis: { title: 'Generation' },
                yaxis: { title: 'Best Fitness' },
                hovermode: 'x unified',
                template: 'plotly_white',
                height: 600
            };
            Plotly.newPlot('plot-fitness', traces, layout, { responsive: true });
        }

        function renderWinratePlot() {
            const experiments = [...new Set(filteredData.map(row => row.experiment_id))];
            const traces = experiments.map(expId => {
                const expData = filteredData.filter(row => row.experiment_id === expId);
                return {
                    x: expData.map(row => row.generation),
                    y: expData.map(row => row.best_winrate),
                    mode: 'lines',
                    name: expData[0].tag,
                    hovertemplate: '<b>%{fullData.name}</b><br>Gen: %{x}<br>Win Rate: %{y:.1f}%<extra></extra>'
                };
            });
            
            const layout = {
                xaxis: { title: 'Generation' },
                yaxis: { title: 'Win Rate (%)' },
                hovermode: 'x unified',
                template: 'plotly_white',
                height: 600,
                shapes: [{ type: 'line', x0: 0, x1: 1, xref: 'x domain', y0: 50, y1: 50, yref: 'y',
                          line: { color: 'red', dash: 'dash' }, opacity: 0.5 }],
                annotations: [{ text: '50% Baseline', x: 1, xanchor: 'right', xref: 'x domain',
                               y: 50, yanchor: 'bottom', yref: 'y', showarrow: false }]
            };
            Plotly.newPlot('plot-winrate', traces, layout, { responsive: true });
        }

        function renderSigmaPlot() {
            const experiments = [...new Set(filteredData.map(row => row.experiment_id))];
            const traces = experiments.map(expId => {
                const expData = filteredData.filter(row => row.experiment_id === expId);
                return {
                    x: expData.map(row => row.generation),
                    y: expData.map(row => row.sigma),
                    mode: 'lines',
                    name: expData[0].tag,
                    hovertemplate: '<b>%{fullData.name}</b><br>Gen: %{x}<br>Sigma: %{y:.4f}<extra></extra>'
                };
            });
            
            const layout = {
                xaxis: { title: 'Generation' },
                yaxis: { title: 'Sigma (Exploration Parameter)' },
                hovermode: 'x unified',
                template: 'plotly_white',
                height: 600
            };
            Plotly.newPlot('plot-sigma', traces, layout, { responsive: true });
        }

        function renderPerformanceBar() {
            const experiments = [...new Set(filteredData.map(row => row.experiment_id))];
            const summaryFiltered = summaryData.filter(row => experiments.includes(row.experiment_id));
            summaryFiltered.sort((a, b) => b.final_fitness - a.final_fitness);
            
            const trace = {
                x: summaryFiltered.map(row => row.tag),
                y: summaryFiltered.map(row => row.final_fitness),
                type: 'bar',
                text: summaryFiltered.map(row => row.final_fitness.toFixed(2)),
                textposition: 'auto',
                marker: { color: summaryFiltered.map(row => row.final_fitness), colorscale: 'Viridis',
                         showscale: true, colorbar: { title: 'Fitness' } },
                hovertemplate: '<b>%{x}</b><br>Fitness: %{y:.2f}<br>Win Rate: %{customdata:.1f}%<extra></extra>',
                customdata: summaryFiltered.map(row => row.final_winrate)
            };
            
            const layout = {
                xaxis: { title: 'Experiment', tickangle: -45 },
                yaxis: { title: 'Final Fitness' },
                template: 'plotly_white',
                height: 500
            };
            Plotly.newPlot('plot-performance', [trace], layout, { responsive: true });
        }

        function renderEfficiencyBar() {
            const experiments = [...new Set(filteredData.map(row => row.experiment_id))];
            const summaryFiltered = summaryData.filter(row => experiments.includes(row.experiment_id));
            const withEfficiency = summaryFiltered.map(row => ({
                ...row,
                efficiency: row.final_fitness / (row.total_time_min / 60)
            }));
            withEfficiency.sort((a, b) => b.efficiency - a.efficiency);
            
            const trace = {
                x: withEfficiency.map(row => row.tag),
                y: withEfficiency.map(row => row.efficiency),
                type: 'bar',
                text: withEfficiency.map(row => row.efficiency.toFixed(1)),
                textposition: 'auto',
                marker: { color: 'coral' },
                hovertemplate: '<b>%{x}</b><br>Efficiency: %{y:.1f} fitness/hour<extra></extra>'
            };
            
            const layout = {
                xaxis: { title: 'Experiment', tickangle: -45 },
                yaxis: { title: 'Fitness / Hour' },
                template: 'plotly_white',
                height: 500
            };
            Plotly.newPlot('plot-efficiency', [trace], layout, { responsive: true });
        }

        function exportVisibleData() {
            const csv = Papa.unparse(filteredData);
            const blob = new Blob([csv], { type: 'text/csv' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `filtered_data_${new Date().toISOString().split('T')[0]}.csv`;
            a.click();
            URL.revokeObjectURL(url);
        }

        window.addEventListener('DOMContentLoaded', loadData);
    </script>
</body>
</html>'''

        # Render template
        template = Template(template_str)
        html_content = template.render(
            title=title,
            timestamp=datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
            num_experiments=len(self.summary_df),
            summary_stats=summary_stats
        )

        # Write to file
        output_path.write_text(html_content)
        file_size_kb = len(html_content) / 1024
        logger.info(f"Dashboard saved to: {output_path} (~{file_size_kb:.1f}KB)")

    def _generate_summary_stats(self) -> dict:
        """Generate summary statistics for template."""
        return {
            "total_experiments": len(self.summary_df),
            "best_fitness": f"{self.summary_df['final_fitness'].max():.2f}",
            "avg_fitness": f"{self.summary_df['final_fitness'].mean():.2f}",
            "best_winrate": f"{self.summary_df['final_winrate'].max():.1f}",
            "total_hours": f"{self.summary_df['total_time_min'].sum() / 60:.1f}"
        }

    # Legacy methods kept for backwards compatibility but not used
    def create_all_plots(self):
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
