//! Experiment directory setup and management.
//!
//! Provides utilities for creating and managing experiment directories
//! with consistent structure for tuning runs.

use std::fs::{self, File};
use std::io::{self, Write as _};
use std::path::PathBuf;

use crate::bots::{BotWeights, GreedyWeights, WeightSet};
use crate::version::VersionInfo;

/// Configuration for an experiment.
#[derive(Clone, Debug)]
pub struct ExperimentConfig {
    /// Base directory for experiments (e.g., "experiments").
    pub base_dir: PathBuf,
    /// Subdirectory category (e.g., "mcts").
    pub category: String,
    /// Descriptive tag for this experiment.
    pub tag: String,
}

impl ExperimentConfig {
    /// Create a new experiment config.
    pub fn new(base_dir: impl Into<PathBuf>, category: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            base_dir: base_dir.into(),
            category: category.into(),
            tag: tag.into(),
        }
    }
}

/// Represents a created experiment directory.
pub struct ExperimentDir {
    /// Root experiment directory path.
    pub root: PathBuf,
    /// Plots subdirectory path.
    pub plots_dir: PathBuf,
    /// Experiment ID (timestamp_tag).
    pub id: String,
}

impl ExperimentDir {
    /// Create a new experiment directory with timestamped ID.
    ///
    /// Creates the directory structure:
    /// ```text
    /// {base_dir}/{category}/{timestamp}_{tag}/
    /// └── plots/
    /// ```
    pub fn create(config: &ExperimentConfig) -> io::Result<Self> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H%M").to_string();
        let id = format!("{}_{}", timestamp, config.tag);
        let root = config.base_dir.join(&config.category).join(&id);

        fs::create_dir_all(&root)?;

        let plots_dir = root.join("plots");
        fs::create_dir_all(&plots_dir)?;

        Ok(Self {
            root,
            plots_dir,
            id,
        })
    }

    /// Save version info for reproducibility.
    pub fn save_version(&self) -> io::Result<()> {
        let version_info = VersionInfo::current();
        let version_path = self.root.join("version.toml");
        let toml_str = toml::to_string_pretty(&version_info)
            .map_err(io::Error::other)?;
        fs::write(version_path, toml_str)
    }

    /// Save weights to the experiment directory.
    pub fn save_weights(&self, weights: &GreedyWeights, name: &str) -> io::Result<PathBuf> {
        let weights_path = self.root.join("weights.toml");
        let bot_weights = BotWeights {
            name: name.to_string(),
            version: 1,
            default: WeightSet {
                greedy: weights.clone(),
            },
            deck_specific: std::collections::HashMap::new(),
        };

        bot_weights
            .save(&weights_path)
            .map_err(io::Error::other)?;

        Ok(weights_path)
    }

    /// Save summary metadata for the experiment.
    pub fn save_summary(
        &self,
        mode: &str,
        best_fitness: f64,
        best_win_rate: f64,
        total_time_secs: f64,
        generations: u32,
    ) -> io::Result<()> {
        let summary_path = self.root.join("summary.txt");
        let mut summary_file = File::create(summary_path)?;
        writeln!(summary_file, "Experiment: {}", self.id)?;
        writeln!(summary_file, "Mode: {}", mode)?;
        writeln!(summary_file, "Best Fitness: {:.2}", best_fitness)?;
        writeln!(summary_file, "Best Win Rate: {:.1}%", best_win_rate * 100.0)?;
        writeln!(summary_file, "Total Time: {:.1}s", total_time_secs)?;
        writeln!(summary_file, "Generations: {}", generations)?;
        Ok(())
    }
}

/// Auto-deploy weights to the canonical location in data/weights/.
///
/// Determines the deploy path based on mode and faction:
/// - generalist -> data/weights/generalist.toml
/// - faction-specialist -> data/weights/specialists/{faction}.toml
/// - specialist -> data/weights/specialists/{faction}.toml (inferred from deck name)
pub fn deploy_weights(
    weights: &GreedyWeights,
    name: &str,
    mode: &str,
    faction: Option<&str>,
    deck: Option<&str>,
) -> io::Result<Option<PathBuf>> {
    let deploy_path = match mode {
        "generalist" | "agent-generalist" => Some(PathBuf::from("data/weights/generalist.toml")),
        "faction-specialist" => {
            if let Some(faction_str) = faction {
                let specialists_dir = PathBuf::from("data/weights/specialists");
                fs::create_dir_all(&specialists_dir)?;
                Some(specialists_dir.join(format!("{}.toml", faction_str.to_lowercase())))
            } else {
                None
            }
        }
        "specialist" => {
            if let Some(deck_name) = deck {
                let specialists_dir = PathBuf::from("data/weights/specialists");
                fs::create_dir_all(&specialists_dir)?;
                // Extract faction from deck name (e.g., "argentum_control" -> "argentum")
                let faction = deck_name.split('_').next().unwrap_or(deck_name);
                Some(specialists_dir.join(format!("{}.toml", faction.to_lowercase())))
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some(ref path) = deploy_path {
        let bot_weights = BotWeights {
            name: name.to_string(),
            version: 1,
            default: WeightSet {
                greedy: weights.clone(),
            },
            deck_specific: std::collections::HashMap::new(),
        };

        bot_weights
            .save(path)
            .map_err(io::Error::other)?;
    }

    Ok(deploy_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_experiment_config_new() {
        let config = ExperimentConfig::new("experiments", "mcts", "test_run");
        assert_eq!(config.base_dir, PathBuf::from("experiments"));
        assert_eq!(config.category, "mcts");
        assert_eq!(config.tag, "test_run");
    }

    #[test]
    fn test_experiment_dir_create() {
        let temp_dir = env::temp_dir().join("cardgame_test_exp");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up from previous runs

        let config = ExperimentConfig::new(&temp_dir, "mcts", "unit_test");
        let exp_dir = ExperimentDir::create(&config).unwrap();

        assert!(exp_dir.root.exists());
        assert!(exp_dir.plots_dir.exists());
        assert!(exp_dir.id.contains("unit_test"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_experiment_dir_save_version() {
        let temp_dir = env::temp_dir().join("cardgame_test_version");
        let _ = fs::remove_dir_all(&temp_dir);

        let config = ExperimentConfig::new(&temp_dir, "mcts", "version_test");
        let exp_dir = ExperimentDir::create(&config).unwrap();

        exp_dir.save_version().unwrap();

        let version_path = exp_dir.root.join("version.toml");
        assert!(version_path.exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
