// microdragon-core/src/cli/animation/spinner.rs
// Animated spinner — gracefully degrades to static text on CMD

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::io::{self, Write};
use tokio::task::JoinHandle;
use crossterm::{execute, cursor, style::{Color, Print, ResetColor, SetForegroundColor}, terminal::{Clear, ClearType}};

use crate::cli::terminal;
use crate::cli::terminal::writer::TermColor;

/// Spinner frame styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpinnerStyle {
    /// ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏  — braille dots (Unicode terminals)
    Braille,
    /// ◐◓◑◒  — arcs
    Arc,
    /// ▁▂▃▄▅▆▇█▇▆▅▄▃▂▁  — grow/shrink
    Grow,
    /// — \ | /  — classic ASCII (CMD safe)
    Classic,
    /// ● ○ ●  — simple pulse (CMD safe)
    Pulse,
    /// [    ] [=   ] [==  ] [=== ] [====]  — fill bar
    Fill,
}

impl SpinnerStyle {
    fn frames(&self) -> &'static [&'static str] {
        match self {
            SpinnerStyle::Braille => &["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"],
            SpinnerStyle::Arc     => &["◐","◓","◑","◒"],
            SpinnerStyle::Grow    => &["▁","▂","▃","▄","▅","▆","▇","█","▇","▆","▅","▄","▃","▂"],
            SpinnerStyle::Classic => &["-","\\","|","/"],
            SpinnerStyle::Pulse   => &["●"," ●","  ●"," ●","●"],
            SpinnerStyle::Fill    => &[
                "[    ]","[=   ]","[==  ]","[=== ]","[====]","[=== ]","[==  ]","[=   ]",
            ],
        }
    }

    fn color(&self) -> TermColor {
        match self {
            SpinnerStyle::Braille | SpinnerStyle::Arc | SpinnerStyle::Grow => TermColor::Cyan,
            SpinnerStyle::Classic => TermColor::Yellow,
            SpinnerStyle::Pulse   => TermColor::Magenta,
            SpinnerStyle::Fill    => TermColor::Green,
        }
    }

    /// Best style for this terminal
    pub fn auto() -> Self {
        if terminal::supports_unicode() && terminal::is_rich() {
            SpinnerStyle::Braille
        } else {
            SpinnerStyle::Classic
        }
    }
}

struct SpinnerState {
    message: String,
    running: bool,
    frame: usize,
    start: Instant,
}

/// Animated terminal spinner with automatic CMD fallback
pub struct Spinner {
    state: Arc<Mutex<SpinnerState>>,
    handle: Option<JoinHandle<()>>,
    style: SpinnerStyle,
    is_animated: bool,
}

impl Spinner {
    /// Create and immediately start a spinner
    pub fn new(message: impl Into<String>) -> Self {
        Self::with_style(message, SpinnerStyle::auto())
    }

    pub fn with_style(message: impl Into<String>, style: SpinnerStyle) -> Self {
        let msg = message.into();
        let is_animated = super::animations_enabled();

        // Non-animated: just print the message
        if !is_animated {
            println!("... {}", msg);
            return Self {
                state: Arc::new(Mutex::new(SpinnerState {
                    message: msg,
                    running: false,
                    frame: 0,
                    start: Instant::now(),
                })),
                handle: None,
                style,
                is_animated: false,
            };
        }

        let state = Arc::new(Mutex::new(SpinnerState {
            message: msg,
            running: true,
            frame: 0,
            start: Instant::now(),
        }));

        // Hide cursor while spinning
        let _ = execute!(io::stdout(), cursor::Hide);

        let state_clone = state.clone();
        let style_clone = style;

        let handle = tokio::spawn(async move {
            let frames = style_clone.frames();
            let color = style_clone.color().to_crossterm();

            loop {
                {
                    let mut s = state_clone.lock().unwrap();
                    if !s.running { break; }

                    let frame = frames[s.frame % frames.len()];
                    let elapsed = s.start.elapsed().as_secs();
                    let timer = if elapsed > 0 { format!(" ({}s)", elapsed) } else { String::new() };
                    let line = format!(" {} {}{}", frame, s.message, timer);

                    let _ = execute!(
                        io::stdout(),
                        cursor::MoveToColumn(0),
                        Clear(ClearType::CurrentLine),
                        SetForegroundColor(color),
                        Print(frame),
                        ResetColor,
                        Print(format!(" {}{}", s.message, timer)),
                    );
                    let _ = io::stdout().flush();

                    s.frame += 1;
                }
                tokio::time::sleep(Duration::from_millis(80)).await;
            }
        });

        Self { state, handle: Some(handle), style, is_animated: true }
    }

    /// Update the spinner message mid-spin
    pub fn set_message(&self, msg: impl Into<String>) {
        if let Ok(mut s) = self.state.lock() {
            s.message = msg.into();
        }
    }

    /// Finish with a success message
    pub fn finish_success(&mut self, msg: impl Into<String>) {
        self.stop();
        let prefix = if terminal::supports_unicode() { "✓" } else { "[OK]" };
        if terminal::supports_color() {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::Green),
                Print(format!("{} {}\n", prefix, msg.into())),
                ResetColor,
            );
        } else {
            println!("{} {}", prefix, msg.into());
        }
    }

    /// Finish with an error message
    pub fn finish_error(&mut self, msg: impl Into<String>) {
        self.stop();
        let prefix = if terminal::supports_unicode() { "✗" } else { "[ERR]" };
        if terminal::supports_color() {
            let _ = execute!(
                io::stdout(),
                SetForegroundColor(Color::Red),
                Print(format!("{} {}\n", prefix, msg.into())),
                ResetColor,
            );
        } else {
            println!("{} {}", prefix, msg.into());
        }
    }

    /// Finish with a neutral message
    pub fn finish(&mut self, msg: impl Into<String>) {
        self.stop();
        println!("{}", msg.into());
    }

    fn stop(&mut self) {
        if let Ok(mut s) = self.state.lock() {
            s.running = false;
        }
        if self.is_animated {
            // Clear the spinner line
            let _ = execute!(
                io::stdout(),
                cursor::MoveToColumn(0),
                Clear(ClearType::CurrentLine),
                cursor::Show,
            );
        }
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}
