#!/usr/bin/env python3
"""Generate leaderboard markdown and HTML from JSON data.

Usage:
    python generate_leaderboard.py                    # Generate docs/leaderboard.md
    python generate_leaderboard.py --format html     # Generate docs/leaderboard.html
    python generate_leaderboard.py --output custom.md # Custom output path
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def load_leaderboard(path: Path) -> dict:
    """Load leaderboard data from JSON file."""
    with open(path) as f:
        return json.load(f)


def generate_markdown(data: dict) -> str:
    """Generate markdown leaderboard from data."""
    metadata = data["metadata"]
    agents = data["agents"]

    # Sort by Elo rating (descending)
    sorted_agents = sorted(agents, key=lambda x: x["elo_rating"], reverse=True)

    # Separate baselines and submitted agents
    baselines = [a for a in sorted_agents if a.get("is_baseline", False)]
    submitted = [a for a in sorted_agents if not a.get("is_baseline", False)]

    lines = []
    lines.append("# Essence Wars Agent Leaderboard")
    lines.append("")
    lines.append(f"> Last updated: {metadata['last_updated'][:10]}")
    lines.append(f"> Benchmark version: {metadata['evaluation_config']['benchmark_version']}")
    lines.append("")
    lines.append("## Rankings")
    lines.append("")
    lines.append("| Rank | Agent | Author | Type | Elo | vs Greedy | vs Random |")
    lines.append("|------|-------|--------|------|-----|-----------|-----------|")

    rank = 1
    for agent in submitted:
        metrics = agent.get("metrics", {})
        vs_greedy = metrics.get("win_rate_vs_greedy")
        vs_random = metrics.get("win_rate_vs_random")

        vs_greedy_str = f"{vs_greedy:.0%}" if vs_greedy is not None else "-"
        vs_random_str = f"{vs_random:.0%}" if vs_random is not None else "-"

        # Add link if HuggingFace URL exists
        submission = agent.get("submission", {})
        model_url = submission.get("model_url")
        name = agent["name"]
        if model_url:
            name = f"[{name}]({model_url})"

        lines.append(
            f"| {rank} | {name} | {agent['author']} | {agent['model_type'].upper()} | "
            f"{agent['elo_rating']:.0f} | {vs_greedy_str} | {vs_random_str} |"
        )
        rank += 1

    lines.append("")
    lines.append("## Baselines")
    lines.append("")
    lines.append("| Agent | Type | Elo | vs Greedy | Description |")
    lines.append("|-------|------|-----|-----------|-------------|")

    for agent in baselines:
        metrics = agent.get("metrics", {})
        vs_greedy = metrics.get("win_rate_vs_greedy")
        vs_greedy_str = f"{vs_greedy:.0%}" if vs_greedy is not None else "-"

        lines.append(
            f"| {agent['name']} | {agent['model_type'].upper()} | "
            f"{agent['elo_rating']:.0f} | {vs_greedy_str} | {agent.get('description', '')} |"
        )

    lines.append("")
    lines.append("## How to Submit")
    lines.append("")
    lines.append("1. Train your agent using `essence-wars` package")
    lines.append("2. Run submission script:")
    lines.append("   ```bash")
    lines.append("   python scripts/submit_agent.py \\")
    lines.append('       --checkpoint your_model.pt \\')
    lines.append('       --name "My Agent" \\')
    lines.append('       --repo-id yourusername/essence-wars-agent')
    lines.append("   ```")
    lines.append("3. Your agent will be evaluated and added to the leaderboard")
    lines.append("")
    lines.append("See [SUBMIT_AGENT.md](./SUBMIT_AGENT.md) for detailed instructions.")
    lines.append("")
    lines.append("## Evaluation Methodology")
    lines.append("")
    config = metadata["evaluation_config"]
    lines.append(f"- **Games per opponent**: {config['games_per_opponent']}")
    lines.append(f"- **Game mode**: {config['game_mode']}")
    lines.append(f"- **Baselines**: {', '.join(config['baselines'])}")
    lines.append("- **Elo calculation**: Standard Elo with K=32")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("*Generated automatically from [leaderboard.json](../data/leaderboard/leaderboard.json)*")

    return "\n".join(lines)


def generate_html(data: dict) -> str:
    """Generate HTML leaderboard from data."""
    metadata = data["metadata"]
    agents = data["agents"]

    # Sort by Elo rating (descending)
    sorted_agents = sorted(agents, key=lambda x: x["elo_rating"], reverse=True)
    submitted = [a for a in sorted_agents if not a.get("is_baseline", False)]

    rows = []
    rank = 1
    for agent in submitted:
        metrics = agent.get("metrics", {})
        vs_greedy = metrics.get("win_rate_vs_greedy")
        vs_random = metrics.get("win_rate_vs_random")

        submission = agent.get("submission", {})
        model_url = submission.get("model_url", "#")

        rows.append(f"""
        <tr>
            <td>{rank}</td>
            <td><a href="{model_url}">{agent['name']}</a></td>
            <td>{agent['author']}</td>
            <td>{agent['model_type'].upper()}</td>
            <td><strong>{agent['elo_rating']:.0f}</strong></td>
            <td>{vs_greedy:.0%}</td>
            <td>{vs_random:.0%}</td>
        </tr>""")
        rank += 1

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Essence Wars Leaderboard</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        h1 {{
            color: #333;
            border-bottom: 2px solid #007bff;
            padding-bottom: 10px;
        }}
        .meta {{
            color: #666;
            font-size: 0.9em;
            margin-bottom: 20px;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            background: white;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
        }}
        th, td {{
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        th {{
            background: #007bff;
            color: white;
        }}
        tr:hover {{
            background: #f5f5f5;
        }}
        a {{
            color: #007bff;
            text-decoration: none;
        }}
        a:hover {{
            text-decoration: underline;
        }}
        .rank-1 {{ background: #ffd700; }}
        .rank-2 {{ background: #c0c0c0; }}
        .rank-3 {{ background: #cd7f32; }}
    </style>
</head>
<body>
    <h1>Essence Wars Agent Leaderboard</h1>
    <div class="meta">
        Last updated: {metadata['last_updated'][:10]} |
        Benchmark v{metadata['evaluation_config']['benchmark_version']} |
        <a href="https://github.com/christianWissmann85/essence-wars">GitHub</a>
    </div>

    <table>
        <thead>
            <tr>
                <th>Rank</th>
                <th>Agent</th>
                <th>Author</th>
                <th>Type</th>
                <th>Elo</th>
                <th>vs Greedy</th>
                <th>vs Random</th>
            </tr>
        </thead>
        <tbody>
            {"".join(rows)}
        </tbody>
    </table>

    <h2>Submit Your Agent</h2>
    <p>
        Train an agent using the <code>essence-wars</code> package and submit it to the leaderboard.
        See the <a href="https://github.com/christianWissmann85/essence-wars/blob/main/docs/SUBMIT_AGENT.md">submission guide</a>.
    </p>

    <footer style="margin-top: 40px; color: #666; font-size: 0.8em;">
        Generated from <a href="https://github.com/christianWissmann85/essence-wars">Essence Wars</a>
    </footer>
</body>
</html>"""

    return html


def main():
    parser = argparse.ArgumentParser(description="Generate leaderboard from JSON")
    parser.add_argument(
        "--input",
        type=Path,
        default=Path("data/leaderboard/leaderboard.json"),
        help="Path to leaderboard JSON",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Output path (default: docs/leaderboard.md or .html)",
    )
    parser.add_argument(
        "--format",
        choices=["markdown", "html"],
        default="markdown",
        help="Output format",
    )
    args = parser.parse_args()

    # Load data
    data = load_leaderboard(args.input)

    # Generate output
    if args.format == "markdown":
        content = generate_markdown(data)
        default_output = Path("docs/leaderboard.md")
    else:
        content = generate_html(data)
        default_output = Path("docs/leaderboard.html")

    output_path = args.output or default_output
    output_path.parent.mkdir(parents=True, exist_ok=True)

    with open(output_path, "w") as f:
        f.write(content)

    print(f"Generated {args.format} leaderboard: {output_path}")


if __name__ == "__main__":
    main()
