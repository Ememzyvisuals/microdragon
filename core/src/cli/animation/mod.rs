// microdragon-core/src/cli/animation/mod.rs
// MICRODRAGON Animation System
// Spinners, progress bars, typing effect, status transitions
// All with graceful fallback for Windows CMD / dumb terminals

pub mod spinner;
pub mod progress;
pub mod typewriter;
pub mod status;

pub use spinner::{Spinner, SpinnerStyle};
pub use progress::{ProgressBar, ProgressStyle};
pub use typewriter::Typewriter;
pub use status::{StatusLine, StatusState};

use crate::cli::terminal;

/// Check if animations should run (terminal capable + not piped)
pub fn animations_enabled() -> bool {
    terminal::is_rich() && !terminal::caps().is_pipe
}
