//! Background task management
//!
//! Handles long-running operations like arena matches, tuning, and benchmarks.

mod arena_task;

pub use arena_task::{
    spawn_arena_task, ArenaConfig, ArenaProgress, ArenaResult, ArenaTaskHandle, BotType,
};

// TODO: Implement in Phase 4-5
// mod benchmark_task;
// mod tuning_task;
// mod analysis_task;

// pub use benchmark_task::BenchmarkTask;
// pub use tuning_task::TuningTask;
// pub use analysis_task::AnalysisTask;
