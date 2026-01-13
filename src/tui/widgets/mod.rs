//! Reusable UI widgets
//!
//! Common components used across multiple screens.

mod checkbox;
mod menu;
mod progress;
mod select;
mod status_bar;
mod text_input;

pub use checkbox::Checkbox;
pub use menu::{Menu, MenuItem};
pub use progress::{ProgressBar, ProgressDisplay};
pub use select::Select;
pub use status_bar::{KeyHint, StatusBar};
pub use text_input::TextInput;

// TODO: Implement in Phase 5-6
// mod dialog;
// mod log_viewer;
// mod table;

// pub use dialog::Dialog;
// pub use log_viewer::LogViewer;
// pub use table::DataTable;
