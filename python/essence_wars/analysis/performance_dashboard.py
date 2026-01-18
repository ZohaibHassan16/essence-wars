#!/usr/bin/env python3
"""
Generate performance benchmark dashboard from Criterion results.
"""

import argparse
import json
from datetime import datetime
from pathlib import Path
from string import Template


def load_criterion_benchmarks(criterion_dir: Path) -> dict:
    """Load benchmark results from Criterion JSON files."""
    benchmarks = {}

    benchmark_dirs = [
        ("state_tensor", "State Tensor", "ns"),
        ("legal_actions", "Legal Actions", "ns"),
        ("engine_fork", "Engine Fork", "ns"),
        ("random_game", "Random Game", "µs"),
        ("greedy_game", "Greedy Game", "µs"),
    ]

    for dir_name, display_name, unit in benchmark_dirs:
        estimates_path = criterion_dir / dir_name / "new" / "estimates.json"
        if estimates_path.exists():
            with open(estimates_path) as f:
                data = json.load(f)
                # Convert to appropriate unit
                value = data["mean"]["point_estimate"]
                ci_lower = data["mean"]["confidence_interval"]["lower_bound"]
                ci_upper = data["mean"]["confidence_interval"]["upper_bound"]

                if unit == "µs" and value < 1000:
                    # Already in ns, convert to µs
                    pass

                benchmarks[dir_name] = {
                    "name": display_name,
                    "value": value,
                    "ci_lower": ci_lower,
                    "ci_upper": ci_upper,
                    "unit": unit,
                }

    # Check for MCTS simulations
    mcts_path = criterion_dir / "mcts_simulations" / "50" / "new" / "estimates.json"
    if mcts_path.exists():
        with open(mcts_path) as f:
            data = json.load(f)
            benchmarks["mcts_50"] = {
                "name": "MCTS 50 sims",
                "value": data["mean"]["point_estimate"],
                "ci_lower": data["mean"]["confidence_interval"]["lower_bound"],
                "ci_upper": data["mean"]["confidence_interval"]["upper_bound"],
                "unit": "µs",
            }

    return benchmarks


def compute_throughput(benchmarks: dict) -> dict:
    """Compute throughput metrics from latency benchmarks."""
    throughput = {}

    if "random_game" in benchmarks:
        # Game time in µs -> games per second
        game_time_us = benchmarks["random_game"]["value"]
        throughput["random_games_per_sec"] = int(1_000_000 / game_time_us)

    if "greedy_game" in benchmarks:
        game_time_us = benchmarks["greedy_game"]["value"]
        throughput["greedy_games_per_sec"] = int(1_000_000 / game_time_us)

    if "state_tensor" in benchmarks:
        # Tensor time in ns -> tensors per second
        tensor_time_ns = benchmarks["state_tensor"]["value"]
        throughput["tensors_per_sec"] = int(1_000_000_000 / tensor_time_ns)

    if "engine_fork" in benchmarks:
        fork_time_ns = benchmarks["engine_fork"]["value"]
        throughput["forks_per_sec"] = int(1_000_000_000 / fork_time_ns)

    return throughput


def generate_html(benchmarks: dict, throughput: dict, output_path: Path):
    """Generate performance dashboard HTML."""

    template = Template('''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Essence Wars - Performance Dashboard</title>
    <script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
    <style>
        :root {
            --bg-primary: #0d1117;
            --bg-secondary: #161b22;
            --bg-card: #21262d;
            --border: #30363d;
            --text-primary: #e6edf3;
            --text-secondary: #8b949e;
            --accent-green: #3fb950;
            --accent-blue: #58a6ff;
            --accent-orange: #d29922;
            --accent-purple: #a371f7;
        }

        * { box-sizing: border-box; margin: 0; padding: 0; }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
        }

        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
        }

        header {
            background: linear-gradient(135deg, var(--bg-secondary), var(--bg-card));
            padding: 30px;
            border-radius: 12px;
            margin-bottom: 30px;
            border: 1px solid var(--border);
        }

        header h1 {
            font-size: 2em;
            margin-bottom: 8px;
            color: var(--accent-orange);
        }

        header .meta {
            color: var(--text-secondary);
            font-size: 0.9em;
        }

        .throughput-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }

        .throughput-card {
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 24px;
            text-align: center;
        }

        .throughput-card .value {
            font-size: 2.5em;
            font-weight: bold;
            color: var(--accent-green);
        }

        .throughput-card .unit {
            font-size: 0.9em;
            color: var(--text-secondary);
        }

        .throughput-card .label {
            margin-top: 8px;
            color: var(--text-secondary);
            font-size: 0.9em;
        }

        .chart-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
            gap: 20px;
        }

        .chart-card {
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 20px;
        }

        .chart-card h2 {
            font-size: 1.2em;
            margin-bottom: 15px;
            color: var(--text-primary);
        }

        .benchmark-table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
        }

        .benchmark-table th, .benchmark-table td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid var(--border);
        }

        .benchmark-table th {
            color: var(--text-secondary);
            font-weight: normal;
        }

        .benchmark-table td.value {
            font-family: 'SFMono-Regular', Consolas, monospace;
            color: var(--accent-blue);
        }

        .back-link {
            display: inline-block;
            margin-bottom: 20px;
            color: var(--accent-blue);
            text-decoration: none;
        }

        .back-link:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <div class="container">
        <a href="../index.html" class="back-link">&larr; Back to Portal</a>

        <header>
            <h1>Performance Benchmarks</h1>
            <p class="meta">Generated: $timestamp | Engine: Essence Wars v0.5.0</p>
        </header>

        <div class="throughput-grid">
            <div class="throughput-card">
                <div class="value">$random_throughput</div>
                <div class="unit">games/sec</div>
                <div class="label">Random Bot Games</div>
            </div>
            <div class="throughput-card">
                <div class="value">$greedy_throughput</div>
                <div class="unit">games/sec</div>
                <div class="label">Greedy Bot Games</div>
            </div>
            <div class="throughput-card">
                <div class="value">$tensor_throughput</div>
                <div class="unit">tensors/sec</div>
                <div class="label">State Tensors</div>
            </div>
            <div class="throughput-card">
                <div class="value">$fork_throughput</div>
                <div class="unit">forks/sec</div>
                <div class="label">Engine Clones</div>
            </div>
        </div>

        <div class="chart-grid">
            <div class="chart-card">
                <h2>Latency Comparison</h2>
                <div id="latency-chart" style="height: 350px;"></div>
            </div>

            <div class="chart-card">
                <h2>Throughput Comparison</h2>
                <div id="throughput-chart" style="height: 350px;"></div>
            </div>
        </div>

        <div class="chart-card" style="margin-top: 20px;">
            <h2>Detailed Benchmark Results</h2>
            <table class="benchmark-table">
                <thead>
                    <tr>
                        <th>Benchmark</th>
                        <th>Mean</th>
                        <th>95% CI</th>
                        <th>Throughput</th>
                    </tr>
                </thead>
                <tbody>
                    $benchmark_rows
                </tbody>
            </table>
        </div>
    </div>

    <script>
        // Latency chart
        const latencyData = $latency_data;

        Plotly.newPlot('latency-chart', [{
            x: latencyData.names,
            y: latencyData.values,
            type: 'bar',
            marker: {
                color: ['#58a6ff', '#58a6ff', '#58a6ff', '#3fb950', '#3fb950', '#a371f7'],
            },
            error_y: {
                type: 'data',
                symmetric: false,
                array: latencyData.errors_upper,
                arrayminus: latencyData.errors_lower,
                color: '#8b949e'
            }
        }], {
            paper_bgcolor: 'rgba(0,0,0,0)',
            plot_bgcolor: 'rgba(0,0,0,0)',
            font: { color: '#e6edf3' },
            yaxis: {
                title: 'Latency (log scale)',
                type: 'log',
                gridcolor: '#30363d'
            },
            xaxis: { gridcolor: '#30363d' },
            margin: { t: 20, b: 80 }
        }, { responsive: true });

        // Throughput chart
        const throughputData = $throughput_data;

        Plotly.newPlot('throughput-chart', [{
            x: throughputData.names,
            y: throughputData.values,
            type: 'bar',
            marker: {
                color: ['#3fb950', '#3fb950', '#58a6ff', '#58a6ff'],
            }
        }], {
            paper_bgcolor: 'rgba(0,0,0,0)',
            plot_bgcolor: 'rgba(0,0,0,0)',
            font: { color: '#e6edf3' },
            yaxis: {
                title: 'Operations/sec (log scale)',
                type: 'log',
                gridcolor: '#30363d'
            },
            xaxis: { gridcolor: '#30363d' },
            margin: { t: 20, b: 80 }
        }, { responsive: true });
    </script>
</body>
</html>
''')

    # Prepare latency data for chart
    latency_names = []
    latency_values = []
    latency_errors_lower = []
    latency_errors_upper = []

    for key, bench in benchmarks.items():
        latency_names.append(bench["name"])
        latency_values.append(bench["value"])
        latency_errors_lower.append(bench["value"] - bench["ci_lower"])
        latency_errors_upper.append(bench["ci_upper"] - bench["value"])

    latency_data = {
        "names": latency_names,
        "values": latency_values,
        "errors_lower": latency_errors_lower,
        "errors_upper": latency_errors_upper,
    }

    # Prepare throughput data for chart
    throughput_data = {
        "names": ["Random Games", "Greedy Games", "State Tensors", "Engine Forks"],
        "values": [
            throughput.get("random_games_per_sec", 0),
            throughput.get("greedy_games_per_sec", 0),
            throughput.get("tensors_per_sec", 0),
            throughput.get("forks_per_sec", 0),
        ]
    }

    # Format throughput for display
    def format_throughput(val):
        if val >= 1_000_000:
            return f"{val / 1_000_000:.1f}M"
        elif val >= 1_000:
            return f"{val / 1_000:.0f}K"
        else:
            return str(val)

    # Generate benchmark rows
    rows = []
    for key, bench in benchmarks.items():
        tp = ""
        if key == "random_game":
            tp = f"{format_throughput(throughput.get('random_games_per_sec', 0))}/sec"
        elif key == "greedy_game":
            tp = f"{format_throughput(throughput.get('greedy_games_per_sec', 0))}/sec"
        elif key == "state_tensor":
            tp = f"{format_throughput(throughput.get('tensors_per_sec', 0))}/sec"
        elif key == "engine_fork":
            tp = f"{format_throughput(throughput.get('forks_per_sec', 0))}/sec"

        rows.append(f'''<tr>
            <td>{bench["name"]}</td>
            <td class="value">{bench["value"]:.2f} {bench["unit"]}</td>
            <td class="value">{bench["ci_lower"]:.2f} - {bench["ci_upper"]:.2f}</td>
            <td class="value">{tp}</td>
        </tr>''')

    html = template.substitute(
        timestamp=datetime.now().strftime("%Y-%m-%d %H:%M"),
        random_throughput=format_throughput(throughput.get("random_games_per_sec", 0)),
        greedy_throughput=format_throughput(throughput.get("greedy_games_per_sec", 0)),
        tensor_throughput=format_throughput(throughput.get("tensors_per_sec", 0)),
        fork_throughput=format_throughput(throughput.get("forks_per_sec", 0)),
        latency_data=json.dumps(latency_data),
        throughput_data=json.dumps(throughput_data),
        benchmark_rows="\n".join(rows),
    )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(html)
    print(f"Performance dashboard generated: {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Generate performance dashboard")
    parser.add_argument("--criterion-dir", type=Path, default=Path("target/criterion"),
                        help="Path to Criterion benchmark results")
    parser.add_argument("--output", "-o", type=Path, default=Path("docs/dashboard/performance.html"),
                        help="Output HTML file")
    args = parser.parse_args()

    if not args.criterion_dir.exists():
        print(f"Error: Criterion directory not found: {args.criterion_dir}")
        print("Run 'cargo bench' first to generate benchmark data.")
        return 1

    benchmarks = load_criterion_benchmarks(args.criterion_dir)
    if not benchmarks:
        print("No benchmark data found.")
        return 1

    throughput = compute_throughput(benchmarks)
    generate_html(benchmarks, throughput, args.output)
    return 0


if __name__ == "__main__":
    exit(main())
