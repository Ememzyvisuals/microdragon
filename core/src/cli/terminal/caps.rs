// microdragon-core/src/cli/terminal/caps.rs
// Terminal capability detection — CMD-safe, cross-platform

use std::env;
use std::io::IsTerminal;

/// Terminal capability levels from most to least capable
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TermLevel {
    /// Pipe, redirect, or no terminal at all — plain text only
    None,
    /// Old Windows CMD, dumb terminal — basic text, no ANSI
    Simple,
    /// Basic ANSI color support (most Linux, macOS, Windows 10+)
    Basic,
    /// Full ANSI + cursor movement (modern terminals)
    Rich,
    /// 256-color + RGB + all features (iTerm2, Windows Terminal, modern)
    Full,
}

/// Complete terminal capability profile
#[derive(Debug, Clone)]
pub struct TermCaps {
    pub level: TermLevel,
    pub ansi_color: bool,
    pub cursor_movement: bool,
    pub unicode: bool,
    pub width: u16,
    pub height: u16,
    pub is_pipe: bool,
    pub is_windows_cmd: bool,
    pub is_windows_terminal: bool,
    pub is_powershell: bool,
    pub platform: TermPlatform,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TermPlatform {
    WindowsCmd,
    WindowsTerminal,
    PowerShell,
    LinuxBash,
    MacOsZsh,
    MacOsBash,
    Unknown,
}

impl TermCaps {
    /// Auto-detect all terminal capabilities
    pub fn detect() -> Self {
        let is_pipe = !std::io::stdout().is_terminal();
        let (width, height) = Self::detect_size();
        let platform = Self::detect_platform();

        let is_windows_cmd = matches!(platform, TermPlatform::WindowsCmd);
        let is_windows_terminal = matches!(platform, TermPlatform::WindowsTerminal);
        let is_powershell = matches!(platform, TermPlatform::PowerShell);

        // ANSI support logic
        let ansi_color = if is_pipe {
            // Check NO_COLOR env var (https://no-color.org)
            env::var("NO_COLOR").is_err()
                && env::var("FORCE_COLOR").is_ok()
        } else {
            Self::detect_ansi_support(&platform)
        };

        // Cursor movement — only in non-pipe, ANSI-capable terminals
        let cursor_movement = ansi_color && !is_pipe;

        // Unicode support
        let unicode = Self::detect_unicode(&platform);

        // Determine overall level
        let level = if is_pipe {
            TermLevel::None
        } else if is_windows_cmd && !Self::is_win10_vt_enabled() {
            TermLevel::Simple
        } else if ansi_color && cursor_movement {
            if Self::detect_256color() {
                TermLevel::Full
            } else {
                TermLevel::Rich
            }
        } else if ansi_color {
            TermLevel::Basic
        } else {
            TermLevel::Simple
        };

        Self {
            level,
            ansi_color,
            cursor_movement,
            unicode,
            width,
            height,
            is_pipe,
            is_windows_cmd,
            is_windows_terminal,
            is_powershell,
            platform,
        }
    }

    /// Convenience: is this a fully-featured terminal?
    pub fn is_rich(&self) -> bool {
        self.level >= TermLevel::Rich
    }

    /// Convenience: at least basic color support?
    pub fn is_colored(&self) -> bool {
        self.level >= TermLevel::Basic
    }

    /// Convenience: plain mode (no color, no animation)
    pub fn is_plain(&self) -> bool {
        self.level <= TermLevel::Simple
    }

    // ─── Private detection helpers ────────────────────────────────────────────

    fn detect_platform() -> TermPlatform {
        #[cfg(windows)]
        {
            // Check TERM_PROGRAM or WT_SESSION for Windows Terminal
            if env::var("WT_SESSION").is_ok() || env::var("WT_PROFILE_ID").is_ok() {
                return TermPlatform::WindowsTerminal;
            }
            // PSModulePath is set by PowerShell
            if env::var("PSModulePath").is_ok() {
                return TermPlatform::PowerShell;
            }
            // ComSpec points to cmd.exe on Windows CMD
            if let Ok(comspec) = env::var("ComSpec") {
                if comspec.to_lowercase().contains("cmd.exe") {
                    return TermPlatform::WindowsCmd;
                }
            }
            return TermPlatform::WindowsCmd;
        }

        #[cfg(not(windows))]
        {
            let shell = env::var("SHELL").unwrap_or_default();
            let term_program = env::var("TERM_PROGRAM").unwrap_or_default();

            if term_program.contains("iTerm") || term_program.contains("Apple_Terminal") {
                if shell.contains("zsh") {
                    return TermPlatform::MacOsZsh;
                }
                return TermPlatform::MacOsBash;
            }

            if shell.contains("zsh") {
                return TermPlatform::MacOsZsh;
            }
            if shell.contains("bash") {
                return TermPlatform::LinuxBash;
            }

            TermPlatform::Unknown
        }
    }

    fn detect_ansi_support(platform: &TermPlatform) -> bool {
        // Explicit override via env
        if env::var("NO_COLOR").is_ok() {
            return false;
        }
        if env::var("FORCE_COLOR").is_ok() || env::var("CLICOLOR_FORCE").is_ok() {
            return true;
        }

        match platform {
            TermPlatform::WindowsCmd => {
                // Windows 10 1511+ supports VT sequences if enabled
                Self::is_win10_vt_enabled()
            }
            TermPlatform::WindowsTerminal | TermPlatform::PowerShell => true,
            TermPlatform::LinuxBash | TermPlatform::MacOsZsh | TermPlatform::MacOsBash => {
                // Check TERM env variable
                let term = env::var("TERM").unwrap_or_default();
                term != "dumb" && !term.is_empty()
            }
            TermPlatform::Unknown => {
                let term = env::var("TERM").unwrap_or_default();
                term.contains("color") || term.contains("xterm") || term.contains("256")
            }
        }
    }

    fn detect_unicode(platform: &TermPlatform) -> bool {
        #[cfg(windows)]
        {
            // Windows Terminal and modern PowerShell support Unicode
            matches!(
                platform,
                TermPlatform::WindowsTerminal | TermPlatform::PowerShell
            )
        }
        #[cfg(not(windows))]
        {
            // Check LANG/LC_ALL for UTF-8
            let lang = env::var("LANG")
                .or_else(|_| env::var("LC_ALL"))
                .unwrap_or_default()
                .to_lowercase();
            lang.contains("utf-8") || lang.contains("utf8") || lang.is_empty()
        }
    }

    fn detect_size() -> (u16, u16) {
        // Try crossterm first, fall back to env vars, then defaults
        if let Ok((w, h)) = crossterm::terminal::size() {
            return (w, h);
        }
        // Fallback: check env
        let cols = env::var("COLUMNS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(80);
        let rows = env::var("LINES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);
        (cols, rows)
    }

    fn detect_256color() -> bool {
        let term = env::var("TERM").unwrap_or_default();
        let colorterm = env::var("COLORTERM").unwrap_or_default();
        term.contains("256") || colorterm == "truecolor" || colorterm == "24bit"
    }

    #[cfg(windows)]
    fn is_win10_vt_enabled() -> bool {
        // On Windows 10+ we can attempt to enable VT processing
        // This checks if it succeeded
        use std::os::windows::io::AsRawHandle;
        unsafe {
            let handle = winapi::um::processenv::GetStdHandle(
                winapi::um::winbase::STD_OUTPUT_HANDLE,
            );
            let mut mode: u32 = 0;
            if winapi::um::consoleapi::GetConsoleMode(handle, &mut mode) != 0 {
                // ENABLE_VIRTUAL_TERMINAL_PROCESSING = 0x0004
                if winapi::um::consoleapi::SetConsoleMode(
                    handle,
                    mode | 0x0004,
                ) != 0 {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(not(windows))]
    fn is_win10_vt_enabled() -> bool {
        true // N/A on non-Windows
    }
}
