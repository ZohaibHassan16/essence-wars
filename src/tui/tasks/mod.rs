//! Background task management
//!
//! Handles long-running operations like arena matches, tuning, and benchmarks.

mod arena_task;
mod tuning_task;

pub use arena_task::{
    spawn_arena_task, ArenaConfig, ArenaProgress, ArenaResult, ArenaTaskHandle, BotType,
};

pub use tuning_task::{
    spawn_tuning_task, GenerationStats, TuningCommand, TuningConfig, TuningModeConfig,
    TuningProgress, TuningResult, TuningTaskHandle,
};

// TODO: Implement in Phase 5
// mod benchmark_task;
// mod analysis_task;

// pub use benchmark_task::BenchmarkTask;
// pub use analysis_task::AnalysisTask;
