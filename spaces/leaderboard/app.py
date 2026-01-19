"""Essence Wars Agent Leaderboard - HuggingFace Space.

Interactive leaderboard for comparing trained agents on the Essence Wars
card game benchmark.
"""

import json
from pathlib import Path

import gradio as gr
import pandas as pd

# Load leaderboard data
LEADERBOARD_PATH = Path(__file__).parent / "leaderboard.json"


def load_leaderboard() -> dict:
    """Load leaderboard data from JSON."""
    with open(LEADERBOARD_PATH) as f:
        return json.load(f)


def format_percentage(value: float | None) -> str:
    """Format a float as percentage."""
    if value is None:
        return "-"
    return f"{value * 100:.1f}%"


def get_model_link(agent: dict) -> str:
    """Get HuggingFace model link if available."""
    url = agent.get("submission", {}).get("model_url")
    if url:
        return f"[Download]({url})"
    return "-"


def create_dataframe(data: dict, model_type: str = "All", show_baselines: bool = True) -> pd.DataFrame:
    """Create pandas DataFrame from leaderboard data."""
    agents = data["agents"]

    # Filter by model type
    if model_type != "All":
        agents = [a for a in agents if a.get("model_type", "").lower() == model_type.lower()]

    # Filter baselines
    if not show_baselines:
        agents = [a for a in agents if not a.get("is_baseline", False)]

    # Sort by Elo rating
    agents = sorted(agents, key=lambda x: x.get("elo_rating", 0), reverse=True)

    rows = []
    for rank, agent in enumerate(agents, 1):
        metrics = agent.get("metrics", {})
        rows.append({
            "Rank": rank,
            "Agent": agent["name"],
            "Author": agent.get("author", "-"),
            "Type": agent.get("model_type", "-").upper(),
            "Elo": agent.get("elo_rating", "-"),
            "vs Random": format_percentage(metrics.get("win_rate_vs_random")),
            "vs Greedy": format_percentage(metrics.get("win_rate_vs_greedy")),
            "vs MCTS-50": format_percentage(metrics.get("win_rate_vs_mcts50")),
            "vs MCTS-100": format_percentage(metrics.get("win_rate_vs_mcts100")),
            "Games": agent.get("total_games", "-"),
            "Model": get_model_link(agent),
        })

    return pd.DataFrame(rows)


def get_agent_details(agent_name: str, data: dict) -> str:
    """Get detailed info for a specific agent."""
    for agent in data["agents"]:
        if agent["name"] == agent_name:
            details = f"""## {agent['name']}

**Author:** {agent.get('author', 'Unknown')}
**Type:** {agent.get('model_type', 'Unknown').upper()}
**Elo Rating:** {agent.get('elo_rating', 'N/A')}
**Total Games:** {agent.get('total_games', 'N/A')}

### Description
{agent.get('description', 'No description available.')}

### Performance
| Opponent | Win Rate |
|----------|----------|
| Random | {format_percentage(agent.get('metrics', {}).get('win_rate_vs_random'))} |
| Greedy | {format_percentage(agent.get('metrics', {}).get('win_rate_vs_greedy'))} |
| MCTS-50 | {format_percentage(agent.get('metrics', {}).get('win_rate_vs_mcts50'))} |
| MCTS-100 | {format_percentage(agent.get('metrics', {}).get('win_rate_vs_mcts100'))} |

### Tags
{', '.join(agent.get('tags', ['none']))}
"""
            submission = agent.get("submission", {})
            if submission.get("model_url"):
                details += f"\n### Download\n[HuggingFace Model]({submission['model_url']})"

            return details

    return "Agent not found."


def create_app():
    """Create the Gradio interface."""
    data = load_leaderboard()

    # Get unique model types for filter
    model_types = ["All"] + sorted(set(
        a.get("model_type", "unknown")
        for a in data["agents"]
        if not a.get("is_baseline", False)
    ))

    # Get agent names for details dropdown
    agent_names = [a["name"] for a in sorted(
        data["agents"],
        key=lambda x: x.get("elo_rating", 0),
        reverse=True
    )]

    with gr.Blocks(title="Essence Wars Leaderboard", theme=gr.themes.Soft()) as app:
        gr.Markdown("""
# Essence Wars Agent Leaderboard

Compare AI agents trained on [Essence Wars](https://github.com/christianWissmann85/essence-wars),
a deterministic card game designed for reinforcement learning research.

**Baselines:** RandomBot (Elo 1000), GreedyBot (Elo 1300), MCTS-50 (Elo 1450), MCTS-100 (Elo 1500)
        """)

        with gr.Row():
            model_filter = gr.Dropdown(
                choices=model_types,
                value="All",
                label="Filter by Model Type",
            )
            show_baselines = gr.Checkbox(
                value=True,
                label="Show Baselines",
            )

        # Main leaderboard table
        leaderboard_df = gr.Dataframe(
            value=create_dataframe(data),
            label="Leaderboard",
            interactive=False,
            wrap=True,
        )

        # Update table when filters change
        def update_table(model_type, show_base):
            return create_dataframe(load_leaderboard(), model_type, show_base)

        model_filter.change(
            update_table,
            inputs=[model_filter, show_baselines],
            outputs=[leaderboard_df],
        )
        show_baselines.change(
            update_table,
            inputs=[model_filter, show_baselines],
            outputs=[leaderboard_df],
        )

        gr.Markdown("---")

        # Agent details section
        gr.Markdown("## Agent Details")
        with gr.Row():
            agent_selector = gr.Dropdown(
                choices=agent_names,
                value=agent_names[0] if agent_names else None,
                label="Select Agent",
            )

        agent_details = gr.Markdown(
            value=get_agent_details(agent_names[0], data) if agent_names else "No agents found."
        )

        def update_details(agent_name):
            return get_agent_details(agent_name, load_leaderboard())

        agent_selector.change(
            update_details,
            inputs=[agent_selector],
            outputs=[agent_details],
        )

        gr.Markdown("""
---

## How to Submit Your Agent

1. Train an agent using PPO, AlphaZero, or your own architecture
2. Run the submission script:
   ```bash
   python python/scripts/submit_agent.py \\
       --checkpoint your_model.pt \\
       --name "Your Agent Name" \\
       --repo-id yourusername/essence-wars-agent
   ```
3. Or open a GitHub issue with the `evaluate-agent` label

See the [submission guide](https://github.com/christianWissmann85/essence-wars/blob/master/docs/SUBMIT_AGENT.md) for details.

---

*Last updated: """ + data["metadata"]["last_updated"][:10] + "*"
        )

    return app


if __name__ == "__main__":
    app = create_app()
    app.launch()
