"""HTML report generator for Essence Wars.

Generates unified HTML reports with tabs for different data views.
"""

from __future__ import annotations

from datetime import datetime
from pathlib import Path

from .loaders.validation import ValidationData, find_latest_validation, load_validation_data
from .loaders.tuning import load_multiple_tuning_experiments
from .loaders.elo import load_elo_data, elo_file_exists
from .tabs.overview import OverviewTab
from .tabs.validation import ValidationTab
from .tabs.tuning import TuningTab
from .tabs.elo import EloTab

# Default output directory
DEFAULT_OUTPUT_DIR = Path("experiments/reports")


class ReportGenerator:
    """Generator for unified HTML reports."""

    def __init__(
        self,
        output_dir: Path | str | None = None,
        theme: str = "dark",
    ):
        """Initialize the report generator.

        Args:
            output_dir: Directory to write reports to (default: experiments/reports)
            theme: Color theme ("dark" or "light")
        """
        self.output_dir = Path(output_dir) if output_dir else DEFAULT_OUTPUT_DIR
        self.theme = theme

        # Find template directory
        self.template_dir = Path(__file__).parent.parent.parent.parent / "templates" / "report"
        if not self.template_dir.exists():
            # Fallback to embedded templates
            self.template_dir = None

    def generate_validation_report(
        self,
        run_id: str | None = None,
        validation_dir: Path | str = "experiments/validation",
        tuning_dir: Path | str = "experiments/mcts",
        elo_file: Path | str | None = None,
        tabs: list[str] | None = None,
    ) -> Path:
        """Generate a report for a single validation run.

        Args:
            run_id: Validation run ID ("latest" for most recent)
            validation_dir: Base directory for validation runs
            tuning_dir: Base directory for tuning experiments
            elo_file: Path to ELO ratings file (default: data/ratings/deck_elo.json)
            tabs: List of tabs to include (default: all)

        Returns:
            Path to the generated report
        """
        # Load validation data
        data = load_validation_data(run_id, Path(validation_dir))

        # Load tuning experiments (up to 10 most recent)
        tuning_experiments = load_multiple_tuning_experiments(
            base_dir=Path(tuning_dir),
            limit=10,
        )

        # Load ELO data if available
        elo_data = None
        if elo_file_exists(elo_file):
            try:
                elo_data = load_elo_data(elo_file)
            except Exception:
                pass  # Skip ELO tab if loading fails

        # Determine which tabs to include
        all_tabs = ["overview", "validation"]
        if tuning_experiments:
            all_tabs.append("tuning")
        if elo_data and elo_data.ratings:
            all_tabs.append("elo")
        tabs = tabs or all_tabs
        tabs = [t for t in tabs if t in all_tabs]

        # Create output directory
        report_dir = self.output_dir / data.run_id
        report_dir.mkdir(parents=True, exist_ok=True)

        # Generate tab content
        tab_content = {}
        if "overview" in tabs:
            tab_content["overview"] = OverviewTab(data).render()
        if "validation" in tabs:
            tab_content["validation"] = ValidationTab(data).render()
        if "tuning" in tabs and tuning_experiments:
            tab_content["tuning"] = TuningTab(tuning_experiments).render()
        if "elo" in tabs and elo_data:
            tab_content["elo"] = EloTab(elo_data).render()

        # Generate HTML
        html = self._render_html(
            title=f"Validation Report - {data.run_id}",
            tab_content=tab_content,
            tabs=tabs,
            data=data,
        )

        # Write files
        output_path = report_dir / "index.html"
        output_path.write_text(html)

        # Write assets
        self._write_assets(report_dir)

        return output_path

    def _render_html(
        self,
        title: str,
        tab_content: dict[str, str],
        tabs: list[str],
        data: ValidationData,
    ) -> str:
        """Render the complete HTML document."""
        # Tab labels
        tab_labels = {
            "overview": "Overview",
            "validation": "Validation",
            "tuning": "Tuning",
            "elo": "ELO Ratings",
        }

        # Build tabs HTML
        tabs_html = []
        for i, tab_id in enumerate(tabs):
            active_class = " active" if i == 0 else ""
            tabs_html.append(
                f'<button class="tab-btn{active_class}" data-tab="{tab_id}">'
                f'{tab_labels.get(tab_id, tab_id.title())}</button>'
            )

        # Build tab panels HTML
        panels_html = []
        for i, tab_id in enumerate(tabs):
            active_class = " active" if i == 0 else ""
            content = tab_content.get(tab_id, "")
            panels_html.append(f'<div class="tab-panel{active_class}" id="{tab_id}-panel">{content}</div>')

        return f"""<!DOCTYPE html>
<html lang="en" data-theme="{self.theme}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <link rel="stylesheet" href="assets/styles.css">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css">
    <script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
</head>
<body>
    <header class="report-header">
        <div class="header-content">
            <h1>Essence Wars</h1>
            <span class="report-type">Balance Report</span>
        </div>
        <div class="header-meta">
            <span class="run-id">{data.run_id}</span>
            <span class="timestamp">{data.timestamp.strftime('%Y-%m-%d %H:%M')}</span>
        </div>
    </header>

    <nav class="tabs">
        {"".join(tabs_html)}
    </nav>

    <main class="content">
        {"".join(panels_html)}
    </main>

    <footer class="report-footer">
        <p>Generated by Essence Wars Report Generator &bull; {datetime.now().strftime('%Y-%m-%d %H:%M')}</p>
        <p>Version {data.version} ({data.git_hash})</p>
    </footer>

    <script src="assets/report.js"></script>
</body>
</html>
"""

    def _write_assets(self, report_dir: Path) -> None:
        """Write CSS and JS assets to the report directory."""
        assets_dir = report_dir / "assets"
        assets_dir.mkdir(exist_ok=True)

        # Write CSS
        css_path = assets_dir / "styles.css"
        css_path.write_text(self._get_css())

        # Write JS
        js_path = assets_dir / "report.js"
        js_path.write_text(self._get_js())

    def _get_css(self) -> str:
        """Get the CSS styles for the report."""
        return """/* Essence Wars Report Styles */

:root {
    /* Dark theme (default) */
    --bg-primary: #1a1a2e;
    --bg-secondary: #16213e;
    --bg-card: #0f3460;
    --bg-hover: #1f4287;
    --text-primary: #eaeaea;
    --text-secondary: #a0a0a0;
    --border-color: #2a2a4e;
    --accent: #e94560;
    --success: #00d26a;
    --warning: #ffc107;
    --danger: #dc3545;

    /* Faction colors */
    --argentum: #D4AF37;
    --symbiote: #7FFF00;
    --obsidion: #00FFFF;
    --neutral: #B87333;
}

[data-theme="light"] {
    --bg-primary: #f5f5f5;
    --bg-secondary: #ffffff;
    --bg-card: #ffffff;
    --bg-hover: #e8e8e8;
    --text-primary: #1a1a2e;
    --text-secondary: #666666;
    --border-color: #dddddd;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    line-height: 1.6;
    min-height: 100vh;
}

/* Header */
.report-header {
    background: var(--bg-secondary);
    padding: 1.5rem 2rem;
    border-bottom: 2px solid var(--accent);
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.header-content h1 {
    font-size: 1.5rem;
    color: var(--accent);
    font-weight: 700;
}

.report-type {
    color: var(--text-secondary);
    font-size: 0.9rem;
    margin-left: 1rem;
}

.header-meta {
    text-align: right;
    font-size: 0.9rem;
    color: var(--text-secondary);
}

.header-meta .run-id {
    display: block;
    font-family: monospace;
    color: var(--text-primary);
}

/* Tabs */
.tabs {
    background: var(--bg-secondary);
    padding: 0 2rem;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    gap: 0.5rem;
}

.tab-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: 1rem 1.5rem;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 500;
    transition: all 0.2s ease;
    border-bottom: 3px solid transparent;
}

.tab-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
}

.tab-btn.active {
    color: var(--accent);
    border-bottom-color: var(--accent);
}

/* Content */
.content {
    padding: 2rem;
    max-width: 1400px;
    margin: 0 auto;
}

.tab-panel {
    display: none;
}

.tab-panel.active {
    display: block;
}

/* Metrics Grid */
.metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
}

.metric-card {
    background: var(--bg-card);
    border-radius: 8px;
    padding: 1.5rem;
    display: flex;
    align-items: center;
    gap: 1rem;
    border: 1px solid var(--border-color);
}

.metric-card.wide {
    grid-column: span 2;
    justify-content: center;
}

.metric-icon {
    font-size: 2rem;
    color: var(--accent);
}

.metric-value {
    font-size: 1.75rem;
    font-weight: 700;
}

.metric-title {
    color: var(--text-secondary);
    font-size: 0.85rem;
}

.metric-card.status-success .metric-value { color: var(--success); }
.metric-card.status-warning .metric-value { color: var(--warning); }
.metric-card.status-balanced .metric-value { color: var(--success); }

/* Charts */
.charts-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
    margin-bottom: 2rem;
}

.chart-card, .chart-section {
    background: var(--bg-card);
    border-radius: 8px;
    padding: 1rem;
    border: 1px solid var(--border-color);
}

.chart-section h3 {
    margin-bottom: 1rem;
    color: var(--text-primary);
}

.chart-container {
    min-height: 300px;
}

/* Warnings */
.warnings-section {
    background: rgba(255, 193, 7, 0.1);
    border: 1px solid var(--warning);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 2rem;
}

.warnings-section h3, .warnings-section h4 {
    color: var(--warning);
    margin-bottom: 1rem;
}

.warning-item {
    padding: 0.5rem 0;
    border-bottom: 1px solid rgba(255, 193, 7, 0.2);
}

.outlier-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 0.5rem;
}

.outlier-item {
    background: var(--bg-card);
    padding: 0.5rem 1rem;
    border-radius: 4px;
    display: flex;
    align-items: center;
    gap: 0.75rem;
}

.outlier-item.high { border-left: 3px solid var(--danger); }
.outlier-item.low { border-left: 3px solid var(--warning); }

/* Tables */
.table-section {
    background: var(--bg-card);
    border-radius: 8px;
    padding: 1.5rem;
    border: 1px solid var(--border-color);
    margin-bottom: 2rem;
}

.table-section h3 {
    margin-bottom: 1rem;
}

.table-wrapper {
    overflow-x: auto;
}

.deck-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
}

.deck-table th, .deck-table td {
    padding: 0.75rem 1rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
}

.deck-table th {
    background: var(--bg-secondary);
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    font-size: 0.75rem;
    letter-spacing: 0.5px;
}

.deck-table tr:hover {
    background: var(--bg-hover);
}

.deck-cell .commander-name {
    font-weight: 600;
    display: block;
}

.deck-cell .deck-id {
    color: var(--text-secondary);
    font-size: 0.8rem;
}

/* Faction badges */
.faction-badge {
    display: inline-block;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
}

.faction-argentum { background: rgba(212, 175, 55, 0.2); color: var(--argentum); }
.faction-symbiote { background: rgba(127, 255, 0, 0.2); color: var(--symbiote); }
.faction-obsidion { background: rgba(0, 255, 255, 0.2); color: var(--obsidion); }
.faction-neutral { background: rgba(184, 115, 51, 0.2); color: var(--neutral); }

/* Indicators */
.indicator {
    font-weight: bold;
    margin-right: 0.5rem;
}

.indicator.high { color: var(--danger); }
.indicator.low { color: var(--warning); }
.indicator.balanced { color: var(--success); }

/* Matchup cells */
.matchup-cell {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
}

.matchup-good { color: var(--success); }
.matchup-bad { color: var(--danger); }
.matchup-rate {
    color: var(--text-secondary);
    font-size: 0.8rem;
}

/* Run info */
.run-info {
    background: var(--bg-card);
    border-radius: 8px;
    padding: 1.5rem;
    border: 1px solid var(--border-color);
}

.run-info h3 {
    margin-bottom: 1rem;
    color: var(--text-primary);
}

.run-info p {
    margin-bottom: 0.5rem;
    color: var(--text-secondary);
}

.run-info strong {
    color: var(--text-primary);
}

/* Footer */
.report-footer {
    background: var(--bg-secondary);
    padding: 1.5rem 2rem;
    text-align: center;
    color: var(--text-secondary);
    font-size: 0.85rem;
    border-top: 1px solid var(--border-color);
    margin-top: 2rem;
}

/* Persistent outliers section */
.persistent-outliers {
    background: rgba(233, 69, 96, 0.1);
    border: 1px solid var(--accent);
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 2rem;
}

.persistent-outliers h3 {
    color: var(--accent);
    margin-bottom: 0.5rem;
}

.persistent-outliers p {
    color: var(--text-secondary);
    margin-bottom: 1rem;
}

.persistent-outliers ul {
    list-style: none;
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
}

.persistent-outliers li {
    background: var(--bg-card);
    padding: 0.5rem 1rem;
    border-radius: 4px;
    border-left: 3px solid var(--accent);
}

/* Status colors for table cells */
.status-success { color: var(--success); font-weight: 600; }
.status-warning { color: var(--warning); font-weight: 600; }
.status-danger { color: var(--danger); font-weight: 600; }

/* Links in tables */
.deck-table a {
    color: var(--accent);
    text-decoration: none;
}

.deck-table a:hover {
    text-decoration: underline;
}

/* Tuning tab styles */
.experiment-selector {
    background: var(--bg-card);
    padding: 1rem 1.5rem;
    border-radius: 8px;
    margin-bottom: 1.5rem;
    border: 1px solid var(--border-color);
}

.experiment-selector label {
    margin-right: 0.5rem;
    color: var(--text-secondary);
}

.experiment-selector select {
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    padding: 0.5rem 1rem;
    border-radius: 4px;
    font-size: 0.9rem;
    cursor: pointer;
}

.experiment-selector select:focus {
    outline: none;
    border-color: var(--accent);
}

.experiment-panel {
    margin-bottom: 2rem;
}

.experiment-header {
    margin-bottom: 1rem;
}

.experiment-header h3 {
    margin-bottom: 0.5rem;
    color: var(--text-primary);
}

.experiment-meta {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    color: var(--text-secondary);
    font-size: 0.9rem;
}

.mode-badge {
    background: var(--accent);
    color: white;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
}

.version {
    font-family: monospace;
    color: var(--text-secondary);
}

.chart-card.full-width {
    grid-column: span 2;
}

.convergence-analysis {
    margin: 1rem 0;
}

.convergence-status {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    font-size: 0.9rem;
}

.convergence-status.converged {
    background: rgba(0, 210, 106, 0.15);
    color: var(--success);
    border: 1px solid var(--success);
}

.convergence-status.in-progress {
    background: rgba(255, 193, 7, 0.15);
    color: var(--warning);
    border: 1px solid var(--warning);
}

.experiment-results {
    background: var(--bg-card);
    padding: 1.5rem;
    border-radius: 8px;
    border: 1px solid var(--border-color);
    margin-top: 1rem;
}

.experiment-results h4 {
    margin-bottom: 1rem;
    color: var(--text-primary);
}

.results-grid {
    display: flex;
    gap: 2rem;
    flex-wrap: wrap;
}

.result-item {
    display: flex;
    flex-direction: column;
}

.result-item .label {
    color: var(--text-secondary);
    font-size: 0.85rem;
}

.result-item .value {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary);
}

.comparison-section {
    margin-top: 2rem;
    padding-top: 2rem;
    border-top: 1px solid var(--border-color);
}

.comparison-section h3 {
    margin-bottom: 1.5rem;
    color: var(--text-primary);
}

.experiments-table-container {
    margin-top: 1.5rem;
}

.data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
}

.data-table th, .data-table td {
    padding: 0.75rem 1rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
}

.data-table th {
    background: var(--bg-secondary);
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    font-size: 0.75rem;
    letter-spacing: 0.5px;
    cursor: pointer;
}

.data-table th:hover {
    color: var(--text-primary);
}

.data-table tr:hover {
    background: var(--bg-hover);
}

.tag-cell {
    font-family: monospace;
    font-size: 0.85rem;
}

.no-data {
    text-align: center;
    padding: 2rem;
    color: var(--text-secondary);
}

/* ELO tab styles */
.chart-description {
    color: var(--text-secondary);
    font-size: 0.9rem;
    margin-bottom: 1rem;
}

.trend-up { color: var(--success); font-weight: bold; }
.trend-down { color: var(--danger); font-weight: bold; }
.trend-stable { color: var(--text-secondary); }

.form-win { color: var(--success); font-weight: 600; margin: 0 1px; }
.form-loss { color: var(--danger); font-weight: 600; margin: 0 1px; }
.form-draw { color: var(--warning); font-weight: 600; margin: 0 1px; }

.history-section {
    margin-top: 2rem;
}

.history-section h3 {
    margin-bottom: 1rem;
    color: var(--text-primary);
}

.history-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1rem;
}

.history-card {
    background: var(--bg-card);
    border-radius: 8px;
    padding: 1rem;
    border: 1px solid var(--border-color);
}

.history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border-color);
}

.history-header .deck-name {
    font-weight: 600;
    color: var(--text-primary);
}

.history-header .current-rating {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--accent);
}

.history-list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
}

.history-item {
    display: flex;
    justify-content: space-between;
    padding: 0.35rem 0.5rem;
    border-radius: 4px;
    font-size: 0.85rem;
}

.history-item.history-win {
    background: rgba(0, 210, 106, 0.1);
}

.history-item.history-loss {
    background: rgba(220, 53, 69, 0.1);
}

.history-item.history-draw {
    background: rgba(255, 193, 7, 0.1);
}

.history-item .opponent {
    color: var(--text-secondary);
}

.history-item .delta {
    font-weight: 600;
}

.history-win .delta { color: var(--success); }
.history-loss .delta { color: var(--danger); }
.history-draw .delta { color: var(--warning); }

/* Responsive */
@media (max-width: 768px) {
    .report-header {
        flex-direction: column;
        gap: 1rem;
        text-align: center;
    }

    .header-meta {
        text-align: center;
    }

    .tabs {
        overflow-x: auto;
        padding: 0 1rem;
    }

    .tab-btn {
        padding: 0.75rem 1rem;
        white-space: nowrap;
    }

    .content {
        padding: 1rem;
    }

    .metric-card.wide {
        grid-column: span 1;
    }
}
"""

    def _get_js(self) -> str:
        """Get the JavaScript for the report."""
        return """// Essence Wars Report JavaScript

document.addEventListener('DOMContentLoaded', function() {
    // Tab switching
    const tabBtns = document.querySelectorAll('.tab-btn');
    const tabPanels = document.querySelectorAll('.tab-panel');

    tabBtns.forEach(btn => {
        btn.addEventListener('click', function() {
            const tabId = this.dataset.tab;

            // Update active states
            tabBtns.forEach(b => b.classList.remove('active'));
            tabPanels.forEach(p => p.classList.remove('active'));

            this.classList.add('active');
            document.getElementById(tabId + '-panel').classList.add('active');

            // Trigger Plotly resize for charts that might be in the newly visible tab
            window.dispatchEvent(new Event('resize'));
        });
    });

    // Make tables sortable
    const tables = document.querySelectorAll('.deck-table');
    tables.forEach(table => {
        const headers = table.querySelectorAll('th');
        headers.forEach((header, index) => {
            header.style.cursor = 'pointer';
            header.addEventListener('click', () => sortTable(table, index));
        });
    });
});

// Experiment selector for tuning tab
function showExperiment(index) {
    // Hide all experiment panels
    document.querySelectorAll('.experiment-panel').forEach(panel => {
        panel.style.display = 'none';
    });
    // Show selected panel
    const panel = document.getElementById('exp-' + index);
    if (panel) {
        panel.style.display = 'block';
        // Trigger Plotly resize for charts
        window.dispatchEvent(new Event('resize'));
    }
}

function sortTable(table, columnIndex) {
    const tbody = table.querySelector('tbody');
    const rows = Array.from(tbody.querySelectorAll('tr'));

    // Determine sort direction
    const currentDir = table.dataset.sortDir === 'asc' ? 'desc' : 'asc';
    table.dataset.sortDir = currentDir;
    table.dataset.sortCol = columnIndex;

    rows.sort((a, b) => {
        const aVal = a.cells[columnIndex].textContent.trim();
        const bVal = b.cells[columnIndex].textContent.trim();

        // Try numeric comparison first
        const aNum = parseFloat(aVal.replace(/[^0-9.-]/g, ''));
        const bNum = parseFloat(bVal.replace(/[^0-9.-]/g, ''));

        if (!isNaN(aNum) && !isNaN(bNum)) {
            return currentDir === 'asc' ? aNum - bNum : bNum - aNum;
        }

        // Fall back to string comparison
        return currentDir === 'asc'
            ? aVal.localeCompare(bVal)
            : bVal.localeCompare(aVal);
    });

    // Re-append sorted rows
    rows.forEach(row => tbody.appendChild(row));
}
"""

    def generate_all_reports(
        self,
        validation_dir: Path | str = "experiments/validation",
        tuning_dir: Path | str = "experiments/mcts",
        elo_file: Path | str | None = None,
        since: datetime | None = None,
        limit: int | None = None,
        force: bool = False,
        tabs: list[str] | None = None,
    ) -> list[Path]:
        """Generate reports for all validation runs.

        Args:
            validation_dir: Base directory for validation runs
            tuning_dir: Base directory for tuning experiments
            elo_file: Path to ELO ratings file
            since: Only generate for runs after this date
            limit: Only generate for the N most recent runs
            force: Regenerate even if report already exists
            tabs: List of tabs to include (default: all)

        Returns:
            List of paths to generated reports
        """
        validation_dir = Path(validation_dir)
        generated = []

        # Find all validation runs
        runs = self._find_validation_runs(validation_dir, since, limit)

        for run_dir in runs:
            run_id = run_dir.name
            report_path = self.output_dir / run_id / "index.html"

            # Skip if report exists and not forcing
            if report_path.exists() and not force:
                continue

            try:
                path = self.generate_validation_report(
                    run_id=run_id,
                    validation_dir=validation_dir,
                    tuning_dir=tuning_dir,
                    elo_file=elo_file,
                    tabs=tabs,
                )
                generated.append(path)
            except Exception as e:
                print(f"Warning: Failed to generate report for {run_id}: {e}")

        return generated

    def generate_aggregated_dashboard(
        self,
        validation_dir: Path | str = "experiments/validation",
        tuning_dir: Path | str = "experiments/mcts",
        limit: int | None = 20,
    ) -> Path:
        """Generate an aggregated dashboard across all runs.

        Args:
            validation_dir: Base directory for validation runs
            tuning_dir: Base directory for tuning experiments
            limit: Maximum number of runs to include (default: 20 most recent)

        Returns:
            Path to the generated dashboard
        """
        validation_dir = Path(validation_dir)
        tuning_dir = Path(tuning_dir)

        # Load validation runs data
        runs = self._find_validation_runs(validation_dir, limit=limit)
        validation_data = []
        for run_dir in runs:
            try:
                data = ValidationData.from_json(run_dir / "results.json")
                validation_data.append(data)
            except Exception:
                continue

        # Load tuning experiments data
        tuning_data = self._load_tuning_summaries(tuning_dir, limit=limit)

        # Generate the aggregated HTML
        html = self._render_aggregated_html(validation_data, tuning_data)

        # Write files
        self.output_dir.mkdir(parents=True, exist_ok=True)
        output_path = self.output_dir / "index.html"
        output_path.write_text(html)

        # Write assets
        self._write_assets(self.output_dir)

        return output_path

    def _find_validation_runs(
        self,
        validation_dir: Path,
        since: datetime | None = None,
        limit: int | None = None,
    ) -> list[Path]:
        """Find validation run directories sorted by date (most recent first)."""
        if not validation_dir.exists():
            return []

        candidates = []
        for d in validation_dir.iterdir():
            if not d.is_dir():
                continue
            if not (d / "results.json").exists():
                continue

            # Try to parse timestamp from directory name
            name = d.name
            if len(name) >= 15 and name[4] == "-" and name[7] == "-" and name[10] == "_":
                try:
                    ts = datetime.strptime(name[:15], "%Y-%m-%d_%H%M")
                    if since and ts < since:
                        continue
                    candidates.append((ts, d))
                except ValueError:
                    # Include runs with non-standard names
                    candidates.append((datetime.min, d))
            else:
                # Include runs with non-standard names (e.g., "phase9-quick")
                candidates.append((datetime.min, d))

        # Sort by timestamp descending (most recent first)
        candidates.sort(key=lambda x: x[0], reverse=True)

        if limit:
            candidates = candidates[:limit]

        return [d for _, d in candidates]

    def _load_tuning_summaries(
        self, tuning_dir: Path, limit: int | None = None
    ) -> list[dict]:
        """Load summary data from tuning experiments."""
        if not tuning_dir.exists():
            return []

        summaries = []
        candidates = []

        for d in tuning_dir.iterdir():
            if not d.is_dir():
                continue
            stats_file = d / "stats.csv"
            if not stats_file.exists():
                continue

            # Parse timestamp from directory name
            name = d.name
            if len(name) >= 15:
                try:
                    ts = datetime.strptime(name[:15], "%Y-%m-%d_%H%M")
                    candidates.append((ts, d))
                except ValueError:
                    candidates.append((datetime.min, d))
            else:
                candidates.append((datetime.min, d))

        # Sort by timestamp descending
        candidates.sort(key=lambda x: x[0], reverse=True)

        if limit:
            candidates = candidates[:limit]

        for ts, d in candidates:
            try:
                summary = self._parse_tuning_summary(d)
                if summary:
                    summaries.append(summary)
            except Exception:
                continue

        return summaries

    def _parse_tuning_summary(self, exp_dir: Path) -> dict | None:
        """Parse summary data from a tuning experiment directory."""
        stats_file = exp_dir / "stats.csv"
        if not stats_file.exists():
            return None

        # Read the last line of stats.csv for final values
        lines = stats_file.read_text().strip().split("\n")
        if len(lines) < 2:
            return None

        header = lines[0].split(",")
        last_row = lines[-1].split(",")

        try:
            data = dict(zip(header, last_row))
            return {
                "experiment_id": exp_dir.name,
                "tag": exp_dir.name.split("_", 2)[-1] if "_" in exp_dir.name else exp_dir.name,
                "generations": int(data.get("generation", 0)) + 1,
                "final_fitness": float(data.get("fitness", 0)),
                "final_win_rate": float(data.get("win_rate", 0)),
                "total_time_mins": float(data.get("cumulative_minutes", 0)),
            }
        except (ValueError, KeyError):
            return None

    def _render_aggregated_html(
        self, validation_data: list[ValidationData], tuning_data: list[dict]
    ) -> str:
        """Render the aggregated dashboard HTML."""
        # Calculate health scores and timeline data
        health_timeline = []
        for data in validation_data:
            health_timeline.append({
                "run_id": data.run_id,
                "timestamp": data.timestamp.strftime("%Y-%m-%d %H:%M"),
                "health_score": data.get_health_score(),
                "total_games": data.total_games,
                "p1_win_rate": data.p1_win_rate,
                "outlier_count": len(data.get_outlier_decks()),
            })

        # Find persistent outliers (appearing in 3+ runs)
        outlier_counts: dict[str, int] = {}
        for data in validation_data:
            for deck in data.get_outlier_decks():
                outlier_counts[deck.commander_name] = outlier_counts.get(deck.commander_name, 0) + 1

        persistent_outliers = [
            (name, count) for name, count in outlier_counts.items() if count >= 3
        ]
        persistent_outliers.sort(key=lambda x: x[1], reverse=True)

        # Build validation runs table
        validation_rows = []
        for entry in health_timeline:
            status_class = "success" if entry["health_score"] >= 75 else "warning" if entry["health_score"] >= 50 else "danger"
            validation_rows.append(f"""
                <tr>
                    <td><a href="{entry['run_id']}/index.html">{entry['run_id']}</a></td>
                    <td>{entry['timestamp']}</td>
                    <td class="status-{status_class}">{entry['health_score']:.0f}</td>
                    <td>{entry['total_games']}</td>
                    <td>{entry['p1_win_rate'] * 100:.1f}%</td>
                    <td>{entry['outlier_count']}</td>
                </tr>
            """)

        # Build tuning experiments table
        tuning_rows = []
        for exp in tuning_data:
            tuning_rows.append(f"""
                <tr>
                    <td>{exp['experiment_id']}</td>
                    <td>{exp['tag']}</td>
                    <td>{exp['generations']}</td>
                    <td>{exp['final_fitness']:.1f}</td>
                    <td>{exp['final_win_rate']:.1f}%</td>
                    <td>{exp['total_time_mins']:.1f}m</td>
                </tr>
            """)

        # Build persistent outliers section
        outliers_html = ""
        if persistent_outliers:
            outliers_html = '<div class="persistent-outliers"><h3>Persistent Outliers</h3><p>Decks appearing as outliers in 3+ validation runs:</p><ul>'
            for name, count in persistent_outliers:
                outliers_html += f"<li><strong>{name}</strong> ({count} runs)</li>"
            outliers_html += "</ul></div>"

        # Build health timeline chart
        chart_html = self._build_health_timeline_chart(health_timeline)

        return f"""<!DOCTYPE html>
<html lang="en" data-theme="{self.theme}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Essence Wars - Reports Dashboard</title>
    <link rel="stylesheet" href="assets/styles.css">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css">
    <script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
</head>
<body>
    <header class="report-header">
        <div class="header-content">
            <h1>Essence Wars</h1>
            <span class="report-type">Reports Dashboard</span>
        </div>
        <div class="header-meta">
            <span class="timestamp">Generated: {datetime.now().strftime('%Y-%m-%d %H:%M')}</span>
        </div>
    </header>

    <main class="content">
        <div class="metrics-grid">
            <div class="metric-card">
                <div class="metric-icon"><i class="fa-solid fa-clipboard-check"></i></div>
                <div class="metric-content">
                    <div class="metric-value">{len(validation_data)}</div>
                    <div class="metric-title">Validation Runs</div>
                </div>
            </div>
            <div class="metric-card">
                <div class="metric-icon"><i class="fa-solid fa-sliders"></i></div>
                <div class="metric-content">
                    <div class="metric-value">{len(tuning_data)}</div>
                    <div class="metric-title">Tuning Experiments</div>
                </div>
            </div>
            <div class="metric-card">
                <div class="metric-icon"><i class="fa-solid fa-triangle-exclamation"></i></div>
                <div class="metric-content">
                    <div class="metric-value">{len(persistent_outliers)}</div>
                    <div class="metric-title">Persistent Outliers</div>
                </div>
            </div>
        </div>

        <div class="chart-section">
            <h3>Balance Health Over Time</h3>
            <div class="chart-container">
                {chart_html}
            </div>
        </div>

        {outliers_html}

        <div class="table-section">
            <h3>Validation Runs</h3>
            <div class="table-wrapper">
                <table class="deck-table">
                    <thead>
                        <tr>
                            <th>Run ID</th>
                            <th>Timestamp</th>
                            <th>Health</th>
                            <th>Games</th>
                            <th>P1 Win Rate</th>
                            <th>Outliers</th>
                        </tr>
                    </thead>
                    <tbody>
                        {"".join(validation_rows) if validation_rows else "<tr><td colspan='6'>No validation runs found</td></tr>"}
                    </tbody>
                </table>
            </div>
        </div>

        <div class="table-section">
            <h3>Tuning Experiments</h3>
            <div class="table-wrapper">
                <table class="deck-table">
                    <thead>
                        <tr>
                            <th>Experiment ID</th>
                            <th>Tag</th>
                            <th>Generations</th>
                            <th>Final Fitness</th>
                            <th>Win Rate</th>
                            <th>Duration</th>
                        </tr>
                    </thead>
                    <tbody>
                        {"".join(tuning_rows) if tuning_rows else "<tr><td colspan='6'>No tuning experiments found</td></tr>"}
                    </tbody>
                </table>
            </div>
        </div>
    </main>

    <footer class="report-footer">
        <p>Generated by Essence Wars Report Generator &bull; {datetime.now().strftime('%Y-%m-%d %H:%M')}</p>
    </footer>

    <script src="assets/report.js"></script>
</body>
</html>
"""

    def _build_health_timeline_chart(self, health_timeline: list[dict]) -> str:
        """Build a Plotly line chart for health score timeline."""
        if not health_timeline:
            return "<p>No data available</p>"

        import plotly.graph_objects as go

        # Reverse to show chronological order
        timeline = list(reversed(health_timeline))

        fig = go.Figure()

        fig.add_trace(
            go.Scatter(
                x=[e["timestamp"] for e in timeline],
                y=[e["health_score"] for e in timeline],
                mode="lines+markers",
                name="Health Score",
                line={"color": "#e94560", "width": 2},
                marker={"size": 8},
                hovertemplate="<b>%{x}</b><br>Health: %{y:.0f}<extra></extra>",
            )
        )

        # Add threshold lines
        fig.add_hline(y=75, line_dash="dash", line_color="#00d26a", line_width=1)
        fig.add_hline(y=50, line_dash="dash", line_color="#ffc107", line_width=1)

        fig.update_layout(
            paper_bgcolor="#1a1a2e",
            plot_bgcolor="#16213e",
            font={"color": "#eaeaea"},
            xaxis={
                "title": None,
                "gridcolor": "#2a2a4e",
                "tickangle": 45,
            },
            yaxis={
                "title": "Health Score",
                "range": [0, 100],
                "gridcolor": "#2a2a4e",
            },
            height=300,
            margin={"t": 20, "b": 80, "l": 60, "r": 30},
            showlegend=False,
        )

        return fig.to_html(full_html=False, include_plotlyjs=False)
