//! Shared application state types

use std::path::PathBuf;

/// Information about a deck
#[derive(Debug, Clone)]
pub struct DeckInfo {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
}

/// Information about a weight file
#[derive(Debug, Clone)]
pub struct WeightInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_default: bool,
}

/// Information about an experiment
#[derive(Debug, Clone)]
pub struct ExperimentInfo {
    pub id: String,
    pub path: PathBuf,
    pub mode: String,
    pub timestamp: String,
    pub best_fitness: Option<f64>,
    pub best_win_rate: Option<f64>,
}

/// Information about a benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkInfo {
    pub id: String,
    pub path: PathBuf,
    pub timestamp: String,
}

/// Shared application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Available decks
    pub decks: Vec<DeckInfo>,

    /// Available weight files
    pub weights: Vec<WeightInfo>,

    /// Recent experiments
    pub experiments: Vec<ExperimentInfo>,

    /// Recent benchmark runs
    pub benchmarks: Vec<BenchmarkInfo>,

    /// Current working directory
    pub cwd: PathBuf,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            decks: Vec::new(),
            weights: Vec::new(),
            experiments: Vec::new(),
            benchmarks: Vec::new(),
            cwd: std::env::current_dir().unwrap_or_default(),
        }
    }
}

impl AppState {
    /// Create new state and scan for available resources
    pub fn new() -> Self {
        let mut state = Self::default();
        state.refresh();
        state
    }

    /// Refresh all resource lists
    pub fn refresh(&mut self) {
        self.scan_decks();
        self.scan_weights();
        self.scan_experiments();
        self.scan_benchmarks();
    }

    fn scan_decks(&mut self) {
        self.decks.clear();
        let deck_dir = self.cwd.join("data/decks");
        if let Ok(entries) = std::fs::read_dir(&deck_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "toml") {
                    if let Some(stem) = path.file_stem() {
                        self.decks.push(DeckInfo {
                            id: stem.to_string_lossy().to_string(),
                            name: stem.to_string_lossy().to_string(),
                            path,
                        });
                    }
                }
            }
        }
        self.decks.sort_by(|a, b| a.id.cmp(&b.id));
    }

    fn scan_weights(&mut self) {
        self.weights.clear();
        let weight_dir = self.cwd.join("data/weights");
        if let Ok(entries) = std::fs::read_dir(&weight_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "toml") {
                    if let Some(stem) = path.file_stem() {
                        let name = stem.to_string_lossy().to_string();
                        self.weights.push(WeightInfo {
                            is_default: name == "default",
                            name,
                            path,
                        });
                    }
                }
            }
        }
        self.weights.sort_by(|a, b| {
            // Default first, then alphabetical
            match (a.is_default, b.is_default) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
    }

    fn scan_experiments(&mut self) {
        self.experiments.clear();
        let exp_dir = self.cwd.join("experiments/mcts");
        if let Ok(entries) = std::fs::read_dir(&exp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        let id = name.to_string_lossy().to_string();
                        // Parse timestamp from ID (format: YYYY-MM-DD_HHMM_tag)
                        let timestamp = id.split('_').take(2).collect::<Vec<_>>().join("_");

                        // Try to read summary for fitness/win rate
                        let mut best_fitness = None;
                        let mut best_win_rate = None;
                        let mut mode = String::from("unknown");

                        let summary_path = path.join("summary.txt");
                        if let Ok(content) = std::fs::read_to_string(&summary_path) {
                            for line in content.lines() {
                                if let Some(val) = line.strip_prefix("Mode: ") {
                                    mode = val.trim().to_string();
                                }
                                if let Some(val) = line.strip_prefix("Best Fitness: ") {
                                    best_fitness = val.trim().parse().ok();
                                }
                                if let Some(val) = line.strip_prefix("Best Win Rate: ") {
                                    best_win_rate = val.trim().trim_end_matches('%').parse().ok();
                                }
                            }
                        }

                        self.experiments.push(ExperimentInfo {
                            id,
                            path,
                            mode,
                            timestamp,
                            best_fitness,
                            best_win_rate,
                        });
                    }
                }
            }
        }
        // Sort by timestamp descending (most recent first)
        self.experiments.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }

    fn scan_benchmarks(&mut self) {
        self.benchmarks.clear();
        // Look for benchmark_results_* directories in cwd
        if let Ok(entries) = std::fs::read_dir(&self.cwd) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with("benchmark_results_") {
                            let timestamp = name_str
                                .strip_prefix("benchmark_results_")
                                .unwrap_or("")
                                .to_string();
                            self.benchmarks.push(BenchmarkInfo {
                                id: name_str.to_string(),
                                path,
                                timestamp,
                            });
                        }
                    }
                }
            }
        }
        // Sort by timestamp descending
        self.benchmarks.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }
}
