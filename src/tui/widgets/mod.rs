//! Reusable UI widgets
//!
//! Common components used across multiple screens.

mod checkbox;
mod menu;
mod number_input;
mod progress;
mod select;
mod status_bar;
mod text_input;
mod toast;

pub use checkbox::Checkbox;
pub use menu::{Menu, MenuItem};
pub use number_input::NumberInput;
pub use progress::{ProgressBar, ProgressDisplay};
pub use select::Select;
pub use status_bar::{KeyHint, StatusBar};
pub use text_input::TextInput;
pub use toast::{Toast, ToastContainer, ToastLevel};

/// Dropdown is an alias for Select
pub type Dropdown = Select;
