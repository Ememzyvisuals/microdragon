// microdragon-core/src/cli/terminal/mod.rs
// Terminal Capability Detection System — Windows CMD + cross-platform

pub mod caps;
pub mod platform;
pub mod writer;

pub use caps::{TermCaps, TermLevel};
pub use platform::detect_platform;
pub use writer::TermWriter;

use once_cell::sync::OnceCell;
use std::sync::Arc;

/// Global terminal capabilities — detected once at startup
static TERM_CAPS: OnceCell<Arc<TermCaps>> = OnceCell::new();

/// Initialize terminal capabilities (call once at startup)
pub fn init() -> Arc<TermCaps> {
    TERM_CAPS.get_or_init(|| Arc::new(TermCaps::detect())).clone()
}

/// Get cached terminal capabilities
pub fn caps() -> Arc<TermCaps> {
    TERM_CAPS.get_or_init(|| Arc::new(TermCaps::detect())).clone()
}

/// Is this a rich terminal (animations, colors, cursor)?
pub fn is_rich() -> bool {
    caps().level >= TermLevel::Basic
}

/// Does terminal support ANSI colors?
pub fn supports_color() -> bool {
    caps().ansi_color
}

/// Does terminal support full Unicode?
pub fn supports_unicode() -> bool {
    caps().unicode
}

/// Does terminal support cursor movement?
pub fn supports_cursor() -> bool {
    caps().cursor_movement
}

/// Force simple/plain output mode (pipe, script, minimal terminal)
pub fn force_simple() {
    // Re-init with forced simple level — only works before first init
    // After init, the caps are frozen; callers should check is_pipe()

// Convenient global caps accessor used like: CAPS.unicode, CAPS.ansi_color


/// Lazily-initialised global terminal caps — use via CAPS.field_name
/// e.g.  if terminal::CAPS.ansi_color { ... }
pub static CAPS: once_cell::sync::Lazy<TermCaps> = once_cell::sync::Lazy::new(TermCaps::detect);
