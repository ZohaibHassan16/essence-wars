//! Help screen - comprehensive documentation and keyboard shortcuts

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};

use super::ScreenWidget;
use crate::tui::app::Message;
use crate::tui::events::is_back_key;
use crate::tui::theme::Theme;
use crate::tui::widgets::{KeyHint, StatusBar};

/// Documentation sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocSection {
    Overview,
    Arena,
    Tuning,
    Weights,
    Analysis,
    Shortcuts,
}

impl DocSection {
    fn all() -> &'static [DocSection] {
        &[
            DocSection::Overview,
            DocSection::Arena,
            DocSection::Tuning,
            DocSection::Weights,
            DocSection::Analysis,
            DocSection::Shortcuts,
        ]
    }

    fn title(&self) -> &'static str {
        match self {
            DocSection::Overview => "Overview",
            DocSection::Arena => "Arena",
            DocSection::Tuning => "Tuning",
            DocSection::Weights => "Weights",
            DocSection::Analysis => "Analysis",
            DocSection::Shortcuts => "Keys",
        }
    }

    fn next(&self) -> Self {
        match self {
            DocSection::Overview => DocSection::Arena,
            DocSection::Arena => DocSection::Tuning,
            DocSection::Tuning => DocSection::Weights,
            DocSection::Weights => DocSection::Analysis,
            DocSection::Analysis => DocSection::Shortcuts,
            DocSection::Shortcuts => DocSection::Overview,
        }
    }

    fn prev(&self) -> Self {
        match self {
            DocSection::Overview => DocSection::Shortcuts,
            DocSection::Arena => DocSection::Overview,
            DocSection::Tuning => DocSection::Arena,
            DocSection::Weights => DocSection::Tuning,
            DocSection::Analysis => DocSection::Weights,
            DocSection::Shortcuts => DocSection::Analysis,
        }
    }
}

/// Help screen state
#[derive(Debug, Clone)]
pub struct HelpScreen {
    section: DocSection,
    scroll: u16,
}

impl HelpScreen {
    pub fn new() -> Self {
        Self {
            section: DocSection::Overview,
            scroll: 0,
        }
    }

    fn build_overview(theme: &Theme) -> Vec<Line<'static>> {
        let h1 = Style::default().fg(theme.primary).bold();
        let h2 = Style::default().fg(theme.success).bold();
        let text = Style::default().fg(theme.fg);
        let dim = Style::default().fg(theme.fg_dim);
        let highlight = Style::default().fg(theme.warning);

        vec![
            Line::from(""),
            Line::from(Span::styled("  ESSENCE WARS RESEARCH LAB", h1)),
            Line::from(""),
            Line::from(Span::styled("  Welcome to the Essence Wars TUI! This app helps you:", text)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    ", text),
                Span::styled("1. ", highlight),
                Span::styled("Run bot matches to test strategies (Arena)", text),
            ]),
            Line::from(vec![
                Span::styled("    ", text),
                Span::styled("2. ", highlight),
                Span::styled("Optimize bot weights using AI (Tuning)", text),
            ]),
            Line::from(vec![
                Span::styled("    ", text),
                Span::styled("3. ", highlight),
                Span::styled("Analyze experiment results (Analysis)", text),
            ]),
            Line::from(vec![
                Span::styled("    ", text),
                Span::styled("4. ", highlight),
                Span::styled("Manage and compare weight files (Weights)", text),
            ]),
            Line::from(vec![
                Span::styled("    ", text),
                Span::styled("5. ", highlight),
                Span::styled("Benchmark engine performance (Benchmarks)", text),
            ]),
            Line::from(""),
            Line::from(Span::styled("  QUICK START", h2)),
            Line::from(""),
            Line::from(Span::styled("  Typical workflow:", text)),
            Line::from(""),
            Line::from(Span::styled("    1. Go to Arena, run Greedy vs Random to see baseline", dim)),
            Line::from(Span::styled("    2. Go to Tuning, run multi-opponent optimization", dim)),
            Line::from(Span::styled("    3. After tuning, press A to view Analysis", dim)),
            Line::from(Span::styled("    4. Press W to compare new weights vs default", dim)),
            Line::from(Span::styled("    5. Press P to promote if results are good!", dim)),
            Line::from(""),
            Line::from(Span::styled("  BOT TYPES", h2)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Random    ", highlight),
                Span::styled("Picks actions uniformly at random. Baseline.", dim),
            ]),
            Line::from(vec![
                Span::styled("    Greedy    ", highlight),
                Span::styled("Simulates each action, picks best by heuristic.", dim),
            ]),
            Line::from(vec![
                Span::styled("    MCTS      ", highlight),
                Span::styled("Monte Carlo Tree Search. Strongest but slowest.", dim),
            ]),
            Line::from(""),
            Line::from(Span::styled("  Typical win rates:", text)),
            Line::from(Span::styled("    - Greedy beats Random: ~100%", dim)),
            Line::from(Span::styled("    - MCTS beats Greedy: 60-85% (depends on simulations)", dim)),
            Line::from(Span::styled("    - Tuned Greedy vs Default Greedy: 55-70%", dim)),
            Line::from(""),
            Line::from(Span::styled("  Use ←/→ to switch sections, ↑/↓ to scroll", dim)),
            Line::from(""),
        ]
    }

    fn build_arena(theme: &Theme) -> Vec<Line<'static>> {
        let h1 = Style::default().fg(theme.primary).bold();
        let h2 = Style::default().fg(theme.success).bold();
        let text = Style::default().fg(theme.fg);
        let dim = Style::default().fg(theme.fg_dim);
        let highlight = Style::default().fg(theme.warning);
        let code = Style::default().fg(theme.info);

        vec![
            Line::from(""),
            Line::from(Span::styled("  ARENA MODE", h1)),
            Line::from(""),
            Line::from(Span::styled("  Run matches between bots to compare strategies.", text)),
            Line::from(""),
            Line::from(Span::styled("  CONFIGURATION OPTIONS", h2)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Bot 1 / Bot 2", highlight),
            ]),
            Line::from(Span::styled("    Select which bot types to pit against each other.", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("      random  ", code),
                Span::styled(" - Random action selection (baseline)", dim),
            ]),
            Line::from(vec![
                Span::styled("      greedy  ", code),
                Span::styled(" - Heuristic evaluation (fast, decent)", dim),
            ]),
            Line::from(vec![
                Span::styled("      mcts    ", code),
                Span::styled(" - Tree search (slow, strongest)", dim),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Deck 1 / Deck 2", highlight),
            ]),
            Line::from(Span::styled("    Choose which card decks each player uses.", dim)),
            Line::from(Span::styled("    Different decks have different strategies:", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("      aggressive_assault  ", code),
                Span::styled(" - Fast aggro, Rush creatures", dim),
            ]),
            Line::from(vec![
                Span::styled("      defensive_control   ", code),
                Span::styled(" - Slow control, Guard creatures", dim),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Number of Games", highlight),
            ]),
            Line::from(Span::styled("    How many games to run in the match.", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("      10-50    ", code),
                Span::styled(" - Quick test, high variance", dim),
            ]),
            Line::from(vec![
                Span::styled("      100-500  ", code),
                Span::styled(" - Good balance (recommended)", dim),
            ]),
            Line::from(vec![
                Span::styled("      1000+    ", code),
                Span::styled(" - Statistical significance, slower", dim),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Swap Sides", highlight),
            ]),
            Line::from(Span::styled("    Run half games with players swapped to reduce", dim)),
            Line::from(Span::styled("    first-player advantage bias. Recommended ON.", dim)),
            Line::from(""),
            Line::from(Span::styled("  INTERPRETING RESULTS", h2)),
            Line::from(""),
            Line::from(Span::styled("    Win Rate: Player 1's wins / total games", dim)),
            Line::from(Span::styled("    50% = evenly matched, 60%+ = clear advantage", dim)),
            Line::from(""),
            Line::from(Span::styled("  EXAMPLE EXPERIMENTS", h2)),
            Line::from(""),
            Line::from(Span::styled("    Test your tuned weights:", dim)),
            Line::from(Span::styled("      Bot1: greedy (with tuned weights)", dim)),
            Line::from(Span::styled("      Bot2: greedy (default weights)", dim)),
            Line::from(Span::styled("      Games: 500, Swap: ON", dim)),
            Line::from(""),
        ]
    }

    fn build_tuning(theme: &Theme) -> Vec<Line<'static>> {
        let h1 = Style::default().fg(theme.primary).bold();
        let h2 = Style::default().fg(theme.success).bold();
        let text = Style::default().fg(theme.fg);
        let dim = Style::default().fg(theme.fg_dim);
        let highlight = Style::default().fg(theme.warning);
        let code = Style::default().fg(theme.info);

        vec![
            Line::from(""),
            Line::from(Span::styled("  WEIGHT TUNING (CMA-ES)", h1)),
            Line::from(""),
            Line::from(Span::styled("  Automatically optimize GreedyBot's evaluation weights", text)),
            Line::from(Span::styled("  using CMA-ES (Covariance Matrix Adaptation Evolution Strategy).", text)),
            Line::from(""),
            Line::from(Span::styled("  HOW CMA-ES WORKS", h2)),
            Line::from(""),
            Line::from(Span::styled("    CMA-ES is an evolutionary optimization algorithm:", dim)),
            Line::from(""),
            Line::from(Span::styled("    1. Generate N candidate weight vectors (population)", dim)),
            Line::from(Span::styled("    2. Evaluate each by playing games (fitness)", dim)),
            Line::from(Span::styled("    3. Select the best performers", dim)),
            Line::from(Span::styled("    4. Update distribution toward better solutions", dim)),
            Line::from(Span::styled("    5. Repeat for G generations", dim)),
            Line::from(""),
            Line::from(Span::styled("  TUNING MODES", h2)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    vs-random", highlight),
            ]),
            Line::from(Span::styled("      Optimize to beat RandomBot. Easy baseline.", dim)),
            Line::from(Span::styled("      Use this first to verify tuning works.", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    vs-greedy", highlight),
            ]),
            Line::from(Span::styled("      Optimize to beat default GreedyBot.", dim)),
            Line::from(Span::styled("      Harder than vs-random, tests real improvement.", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    multi-opponent (RECOMMENDED)", highlight),
            ]),
            Line::from(Span::styled("      Optimize against multiple opponents:", dim)),
            Line::from(Span::styled("        - 10% Random (easy baseline)", dim)),
            Line::from(Span::styled("        - 40% Greedy (main challenge)", dim)),
            Line::from(Span::styled("        - 50% MCTS (hard target)", dim)),
            Line::from(Span::styled("      Most robust, prevents overfitting to one opponent.", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    generalist", highlight),
            ]),
            Line::from(Span::styled("      Ultra-robust: ALL deck matchups vs Random/Greedy/MCTS.", dim)),
            Line::from(Span::styled("      Best for universal weights that work with any deck.", dim)),
            Line::from(Span::styled("      ⚠️  Slowest - 30+ min for 100 generations.", dim)),
            Line::from(""),
            Line::from(Span::styled("  PARAMETERS EXPLAINED", h2)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Tag", highlight),
            ]),
            Line::from(Span::styled("      Name for this experiment. Used in folder name:", dim)),
            Line::from(vec![
                Span::styled("      experiments/mcts/", dim),
                Span::styled("2026-01-13_1430_mytag", code),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Generations", highlight),
            ]),
            Line::from(Span::styled("      Number of optimization iterations.", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("        10-20   ", code),
                Span::styled(" - Quick test, may not converge", dim),
            ]),
            Line::from(vec![
                Span::styled("        50-100  ", code),
                Span::styled(" - Good balance (recommended)", dim),
            ]),
            Line::from(vec![
                Span::styled("        200+    ", code),
                Span::styled(" - Thorough, diminishing returns", dim),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Population", highlight),
            ]),
            Line::from(Span::styled("      Candidates evaluated per generation.", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("        0       ", code),
                Span::styled(" - Auto: 4 + 3*ln(24) = ~13 (recommended)", dim),
            ]),
            Line::from(vec![
                Span::styled("        5-10    ", code),
                Span::styled(" - Faster, may miss solutions", dim),
            ]),
            Line::from(vec![
                Span::styled("        20-30   ", code),
                Span::styled(" - More thorough exploration", dim),
            ]),
            Line::from(""),
            Line::from(Span::styled("      Larger population = more exploration but slower.", dim)),
            Line::from(Span::styled("      If results seem unstable, try increasing.", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Games per Eval", highlight),
            ]),
            Line::from(Span::styled("      Games played to evaluate each candidate.", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("        20-50   ", code),
                Span::styled(" - Fast, noisy fitness signal", dim),
            ]),
            Line::from(vec![
                Span::styled("        100     ", code),
                Span::styled(" - Good balance (recommended)", dim),
            ]),
            Line::from(vec![
                Span::styled("        200+    ", code),
                Span::styled(" - More accurate, much slower", dim),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Parallel", highlight),
            ]),
            Line::from(Span::styled("      Use all CPU cores for evaluation. 10-15x faster!", dim)),
            Line::from(Span::styled("      Recommended: ON (unless debugging).", dim)),
            Line::from(""),
            Line::from(Span::styled("  AFTER TUNING", h2)),
            Line::from(""),
            Line::from(Span::styled("    When tuning completes, you can:", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("      A ", code),
                Span::styled("- View Analysis (see detailed stats)", dim),
            ]),
            Line::from(vec![
                Span::styled("      W ", code),
                Span::styled("- Compare Weights (diff vs default)", dim),
            ]),
            Line::from(vec![
                Span::styled("      P ", code),
                Span::styled("- Promote (set as new default)", dim),
            ]),
            Line::from(""),
            Line::from(Span::styled("  TIPS", h2)),
            Line::from(""),
            Line::from(Span::styled("    - Start with vs-random to verify setup", dim)),
            Line::from(Span::styled("    - Use multi-opponent for production tuning", dim)),
            Line::from(Span::styled("    - 50 generations is usually sufficient", dim)),
            Line::from(Span::styled("    - Keep Parallel ON unless debugging", dim)),
            Line::from(Span::styled("    - Compare results in Arena before promoting", dim)),
            Line::from(""),
        ]
    }

    fn build_weights(theme: &Theme) -> Vec<Line<'static>> {
        let h1 = Style::default().fg(theme.primary).bold();
        let h2 = Style::default().fg(theme.success).bold();
        let text = Style::default().fg(theme.fg);
        let dim = Style::default().fg(theme.fg_dim);
        let highlight = Style::default().fg(theme.warning);
        let pos = Style::default().fg(theme.success);
        let neg = Style::default().fg(theme.error);

        vec![
            Line::from(""),
            Line::from(Span::styled("  WEIGHT PARAMETERS", h1)),
            Line::from(""),
            Line::from(Span::styled("  GreedyBot evaluates game states using 24 weighted factors.", text)),
            Line::from(Span::styled("  Higher weight = more important in decisions.", text)),
            Line::from(""),
            Line::from(Span::styled("  LIFE CATEGORY", h2)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    own_life ", highlight),
                Span::styled("(+) ", pos),
                Span::styled("Value of your life total. Higher = more defensive.", dim),
            ]),
            Line::from(vec![
                Span::styled("    enemy_life_damage ", highlight),
                Span::styled("(+) ", pos),
                Span::styled("Value of dealing damage. Higher = more aggressive.", dim),
            ]),
            Line::from(""),
            Line::from(Span::styled("  CREATURE CATEGORY", h2)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    own_creature_attack ", highlight),
                Span::styled("(+) ", pos),
                Span::styled("Value your creatures' attack power.", dim),
            ]),
            Line::from(vec![
                Span::styled("    own_creature_health ", highlight),
                Span::styled("(+) ", pos),
                Span::styled("Value your creatures' survivability.", dim),
            ]),
            Line::from(vec![
                Span::styled("    enemy_creature_attack ", highlight),
                Span::styled("(-) ", neg),
                Span::styled("Penalty for enemy attack (want to remove threats).", dim),
            ]),
            Line::from(vec![
                Span::styled("    enemy_creature_health ", highlight),
                Span::styled("(-) ", neg),
                Span::styled("Penalty for enemy health (hard to kill).", dim),
            ]),
            Line::from(""),
            Line::from(Span::styled("  BOARD CATEGORY", h2)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    creature_count ", highlight),
                Span::styled("(+) ", pos),
                Span::styled("Value having more creatures on board.", dim),
            ]),
            Line::from(vec![
                Span::styled("    board_advantage ", highlight),
                Span::styled("(+) ", pos),
                Span::styled("Value having more creatures than opponent.", dim),
            ]),
            Line::from(""),
            Line::from(Span::styled("  RESOURCE CATEGORY", h2)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    cards_in_hand ", highlight),
                Span::styled("(+) ", pos),
                Span::styled("Value having cards (options/flexibility).", dim),
            ]),
            Line::from(vec![
                Span::styled("    action_points ", highlight),
                Span::styled("(+) ", pos),
                Span::styled("Value unspent action points.", dim),
            ]),
            Line::from(""),
            Line::from(Span::styled("  KEYWORD CATEGORY", h2)),
            Line::from(""),
            Line::from(Span::styled("    Value multipliers for creatures with keywords:", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    guard ", highlight),
                Span::styled("- Protects other creatures from attack", dim),
            ]),
            Line::from(vec![
                Span::styled("    lethal ", highlight),
                Span::styled("- Kills any creature it damages", dim),
            ]),
            Line::from(vec![
                Span::styled("    lifesteal ", highlight),
                Span::styled("- Heals you when dealing damage", dim),
            ]),
            Line::from(vec![
                Span::styled("    rush ", highlight),
                Span::styled("- Can attack immediately when played", dim),
            ]),
            Line::from(vec![
                Span::styled("    ranged ", highlight),
                Span::styled("- Doesn't take counter-damage", dim),
            ]),
            Line::from(vec![
                Span::styled("    piercing ", highlight),
                Span::styled("- Excess damage hits enemy life", dim),
            ]),
            Line::from(vec![
                Span::styled("    shield ", highlight),
                Span::styled("- Blocks first damage instance", dim),
            ]),
            Line::from(vec![
                Span::styled("    quick ", highlight),
                Span::styled("- Attacks first in combat", dim),
            ]),
            Line::from(vec![
                Span::styled("    ephemeral ", highlight),
                Span::styled("- Dies at end of turn (negative value)", dim),
            ]),
            Line::from(vec![
                Span::styled("    regenerate ", highlight),
                Span::styled("- Heals 2 HP at start of turn", dim),
            ]),
            Line::from(vec![
                Span::styled("    stealth ", highlight),
                Span::styled("- Cannot be targeted by enemy", dim),
            ]),
            Line::from(vec![
                Span::styled("    charge ", highlight),
                Span::styled("- +2 attack when attacking", dim),
            ]),
            Line::from(""),
            Line::from(Span::styled("  TERMINAL CATEGORY", h2)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    win_bonus ", highlight),
                Span::styled("(+) ", pos),
                Span::styled("Bonus score for winning the game.", dim),
            ]),
            Line::from(vec![
                Span::styled("    lose_penalty ", highlight),
                Span::styled("(-) ", neg),
                Span::styled("Penalty for losing the game.", dim),
            ]),
            Line::from(""),
            Line::from(Span::styled("  INTERPRETING WEIGHT DIFFS", h2)),
            Line::from(""),
            Line::from(Span::styled("    When comparing weights:", dim)),
            Line::from(vec![
                Span::styled("      Green (+) ", pos),
                Span::styled("= tuned version values this MORE", dim),
            ]),
            Line::from(vec![
                Span::styled("      Red (-)   ", neg),
                Span::styled("= tuned version values this LESS", dim),
            ]),
            Line::from(""),
            Line::from(Span::styled("    Large differences (>0.5) indicate significant", dim)),
            Line::from(Span::styled("    strategy changes. Small differences (<0.1) are noise.", dim)),
            Line::from(""),
        ]
    }

    fn build_analysis(theme: &Theme) -> Vec<Line<'static>> {
        let h1 = Style::default().fg(theme.primary).bold();
        let h2 = Style::default().fg(theme.success).bold();
        let text = Style::default().fg(theme.fg);
        let dim = Style::default().fg(theme.fg_dim);
        let highlight = Style::default().fg(theme.warning);
        let code = Style::default().fg(theme.info);

        vec![
            Line::from(""),
            Line::from(Span::styled("  ANALYSIS & RESULTS", h1)),
            Line::from(""),
            Line::from(Span::styled("  View and interpret tuning experiment results.", text)),
            Line::from(""),
            Line::from(Span::styled("  EXPERIMENT FILES", h2)),
            Line::from(""),
            Line::from(Span::styled("    Each experiment creates a folder:", dim)),
            Line::from(vec![
                Span::styled("      experiments/mcts/", dim),
                Span::styled("2026-01-13_1430_mytag/", code),
            ]),
            Line::from(""),
            Line::from(Span::styled("    Containing:", dim)),
            Line::from(vec![
                Span::styled("      weights.toml  ", code),
                Span::styled("- Best weights found", dim),
            ]),
            Line::from(vec![
                Span::styled("      summary.txt   ", code),
                Span::styled("- Quick stats overview", dim),
            ]),
            Line::from(vec![
                Span::styled("      stats.csv     ", code),
                Span::styled("- Per-generation metrics", dim),
            ]),
            Line::from(vec![
                Span::styled("      train.log     ", code),
                Span::styled("- Full training log", dim),
            ]),
            Line::from(vec![
                Span::styled("      version.toml  ", code),
                Span::styled("- Engine version info", dim),
            ]),
            Line::from(""),
            Line::from(Span::styled("  KEY METRICS", h2)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Fitness", highlight),
            ]),
            Line::from(Span::styled("      The optimization score. Higher = better.", dim)),
            Line::from(Span::styled("      Computed from win rate against opponents.", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Win Rate", highlight),
            ]),
            Line::from(Span::styled("      Percentage of games won during evaluation.", dim)),
            Line::from(Span::styled("      > 50% means beating opponents on average.", dim)),
            Line::from(""),
            Line::from(vec![
                Span::styled("    Sigma", highlight),
            ]),
            Line::from(Span::styled("      CMA-ES step size. Shows exploration amount.", dim)),
            Line::from(Span::styled("      Decreases as algorithm converges.", dim)),
            Line::from(Span::styled("      Very small sigma = converged (may stop early).", dim)),
            Line::from(""),
            Line::from(Span::styled("  WHAT GOOD RESULTS LOOK LIKE", h2)),
            Line::from(""),
            Line::from(Span::styled("    Successful tuning typically shows:", dim)),
            Line::from(""),
            Line::from(Span::styled("    - Fitness trending upward over generations", dim)),
            Line::from(Span::styled("    - Win rate improving from ~50% to 60-80%", dim)),
            Line::from(Span::styled("    - Sigma decreasing (converging)", dim)),
            Line::from(Span::styled("    - Final win rate significantly above baseline", dim)),
            Line::from(""),
            Line::from(Span::styled("  WARNING SIGNS", h2)),
            Line::from(""),
            Line::from(Span::styled("    - Fitness stuck or fluctuating wildly", dim)),
            Line::from(Span::styled("    - Win rate around 50% (not learning)", dim)),
            Line::from(Span::styled("    - Very fast convergence (local optimum)", dim)),
            Line::from(""),
            Line::from(Span::styled("    If you see these, try:", dim)),
            Line::from(Span::styled("    - More generations", dim)),
            Line::from(Span::styled("    - Larger population", dim)),
            Line::from(Span::styled("    - Different tuning mode", dim)),
            Line::from(""),
        ]
    }

    fn build_shortcuts(theme: &Theme) -> Vec<Line<'static>> {
        let h1 = Style::default().fg(theme.primary).bold();
        let h2 = Style::default().fg(theme.success).bold();
        let key = Style::default().fg(theme.fg);
        let desc = Style::default().fg(theme.fg_dim);

        vec![
            Line::from(""),
            Line::from(Span::styled("  KEYBOARD SHORTCUTS", h1)),
            Line::from(""),
            Line::from(Span::styled("  GLOBAL", h2)),
            Line::from(""),
            Line::from(vec![Span::styled("    Q           ", key), Span::styled("Quit application", desc)]),
            Line::from(vec![Span::styled("    Esc         ", key), Span::styled("Go back / Cancel", desc)]),
            Line::from(vec![Span::styled("    ?           ", key), Span::styled("Show this help", desc)]),
            Line::from(vec![Span::styled("    ↑↓          ", key), Span::styled("Navigate / Scroll", desc)]),
            Line::from(vec![Span::styled("    Enter       ", key), Span::styled("Select / Confirm", desc)]),
            Line::from(vec![Span::styled("    Tab         ", key), Span::styled("Next field", desc)]),
            Line::from(vec![Span::styled("    Shift+Tab   ", key), Span::styled("Previous field", desc)]),
            Line::from(""),
            Line::from(Span::styled("  HOME", h2)),
            Line::from(""),
            Line::from(vec![Span::styled("    1-6         ", key), Span::styled("Quick jump to menu item", desc)]),
            Line::from(""),
            Line::from(Span::styled("  ARENA", h2)),
            Line::from(""),
            Line::from(vec![Span::styled("    Space       ", key), Span::styled("Toggle checkbox", desc)]),
            Line::from(vec![Span::styled("    Enter       ", key), Span::styled("Start match", desc)]),
            Line::from(""),
            Line::from(Span::styled("  TUNING", h2)),
            Line::from(""),
            Line::from(vec![Span::styled("    Space       ", key), Span::styled("Pause / Resume", desc)]),
            Line::from(vec![Span::styled("    Ctrl+C      ", key), Span::styled("Stop tuning", desc)]),
            Line::from(""),
            Line::from(Span::styled("  TUNING (COMPLETED)", h2)),
            Line::from(""),
            Line::from(vec![Span::styled("    A           ", key), Span::styled("View Analysis", desc)]),
            Line::from(vec![Span::styled("    W           ", key), Span::styled("Compare Weights", desc)]),
            Line::from(vec![Span::styled("    P           ", key), Span::styled("Promote to default", desc)]),
            Line::from(vec![Span::styled("    R           ", key), Span::styled("New tuning run", desc)]),
            Line::from(""),
            Line::from(Span::styled("  ANALYSIS", h2)),
            Line::from(""),
            Line::from(vec![Span::styled("    Enter       ", key), Span::styled("View experiment details", desc)]),
            Line::from(vec![Span::styled("    PgUp/PgDn   ", key), Span::styled("Page through stats", desc)]),
            Line::from(""),
            Line::from(Span::styled("  WEIGHTS", h2)),
            Line::from(""),
            Line::from(vec![Span::styled("    Enter       ", key), Span::styled("View weight details", desc)]),
            Line::from(vec![Span::styled("    C           ", key), Span::styled("Compare (select two)", desc)]),
            Line::from(vec![Span::styled("    P           ", key), Span::styled("Promote to default", desc)]),
            Line::from(""),
            Line::from(Span::styled("  BENCHMARKS", h2)),
            Line::from(""),
            Line::from(vec![Span::styled("    R           ", key), Span::styled("Run benchmarks", desc)]),
            Line::from(""),
            Line::from(Span::styled("  THIS HELP", h2)),
            Line::from(""),
            Line::from(vec![Span::styled("    ←→          ", key), Span::styled("Switch sections", desc)]),
            Line::from(vec![Span::styled("    ↑↓          ", key), Span::styled("Scroll content", desc)]),
            Line::from(vec![Span::styled("    PgUp/PgDn   ", key), Span::styled("Page scroll", desc)]),
            Line::from(""),
        ]
    }

    fn get_content(&self, theme: &Theme) -> Vec<Line<'static>> {
        match self.section {
            DocSection::Overview => Self::build_overview(theme),
            DocSection::Arena => Self::build_arena(theme),
            DocSection::Tuning => Self::build_tuning(theme),
            DocSection::Weights => Self::build_weights(theme),
            DocSection::Analysis => Self::build_analysis(theme),
            DocSection::Shortcuts => Self::build_shortcuts(theme),
        }
    }
}

impl Default for HelpScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenWidget for HelpScreen {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Tabs
                Constraint::Min(10),    // Content
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Section tabs
        let tab_titles: Vec<&str> = DocSection::all().iter().map(|s| s.title()).collect();
        let selected = DocSection::all().iter().position(|s| *s == self.section).unwrap_or(0);

        let tabs = Tabs::new(tab_titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Documentation ")
                    .style(Style::default().bg(theme.bg_alt)),
            )
            .select(selected)
            .style(Style::default().fg(theme.fg_dim))
            .highlight_style(Style::default().fg(theme.primary).bold());
        frame.render_widget(tabs, chunks[0]);

        // Content
        let content_lines = self.get_content(theme);
        let content = Paragraph::new(content_lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            );
        frame.render_widget(content, chunks[1]);

        // Footer
        let hints = vec![
            KeyHint::new("←→", "Section"),
            KeyHint::new("↑↓", "Scroll"),
            KeyHint::new("PgUp/Dn", "Page"),
            KeyHint::new("Esc", "Back"),
        ];
        let status_bar = StatusBar::new(&hints, theme);
        frame.render_widget(status_bar, chunks[2]);
    }

    fn handle_key(&mut self, key: &KeyEvent) -> Option<Message> {
        if is_back_key(key) {
            return Some(Message::GoBack);
        }

        match key.code {
            // Section navigation
            KeyCode::Left | KeyCode::Char('h') => {
                self.section = self.section.prev();
                self.scroll = 0;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.section = self.section.next();
                self.scroll = 0;
            }
            // Scroll
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll += 1;
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(15);
            }
            KeyCode::PageDown => {
                self.scroll += 15;
            }
            KeyCode::Home => {
                self.scroll = 0;
            }
            _ => {}
        }

        None
    }
}
