//! Configuration persistence for the TUI
//!
//! Saves user preferences to ~/.config/essence-wars/lab.toml

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// User configuration for the TUI
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TuiConfig {
    /// Arena screen preferences
    #[serde(default)]
    pub arena: ArenaConfig,

    /// Tuning screen preferences
    #[serde(default)]
    pub tuning: TuningConfig,
}

/// Arena configuration preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaConfig {
    /// Last selected bot 1 type
    #[serde(default = "default_bot_type")]
    pub bot1_type: String,

    /// Last selected bot 2 type
    #[serde(default = "default_bot_type")]
    pub bot2_type: String,

    /// Last selected deck 1
    #[serde(default)]
    pub deck1: String,

    /// Last selected deck 2
    #[serde(default)]
    pub deck2: String,

    /// Last game count
    #[serde(default = "default_game_count")]
    pub games: u32,
}

/// Tuning configuration preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningConfig {
    /// Last tuning mode
    #[serde(default = "default_tuning_mode")]
    pub mode: String,

    /// Last generation count
    #[serde(default = "default_generations")]
    pub generations: u32,

    /// Last games per evaluation
    #[serde(default = "default_eval_games")]
    pub games_per_eval: u32,

    /// Use parallel evaluation
    #[serde(default = "default_true")]
    pub parallel: bool,
}

fn default_bot_type() -> String {
    "greedy".to_string()
}

fn default_game_count() -> u32 {
    100
}

fn default_tuning_mode() -> String {
    "multi-opponent".to_string()
}

fn default_generations() -> u32 {
    50
}

fn default_eval_games() -> u32 {
    100
}

fn default_true() -> bool {
    true
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            bot1_type: default_bot_type(),
            bot2_type: "random".to_string(),
            deck1: String::new(),
            deck2: String::new(),
            games: default_game_count(),
        }
    }
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            mode: default_tuning_mode(),
            generations: default_generations(),
            games_per_eval: default_eval_games(),
            parallel: true,
        }
    }
}

impl TuiConfig {
    /// Get the config file path
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("essence-wars")
            .join("lab.toml")
    }

    /// Load config from file, or return default if not found
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    toml::from_str(&content).unwrap_or_default()
                }
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, content)
            .map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }
}
