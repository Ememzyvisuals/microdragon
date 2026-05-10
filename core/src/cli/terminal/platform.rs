// microdragon-core/src/cli/terminal/platform.rs
// Platform detection helpers

use std::env;

#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os: Os,
    pub shell: Shell,
    pub is_ci: bool,
    pub is_interactive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Os {
    Windows,
    Linux,
    MacOs,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shell {
    Cmd,
    PowerShell,
    Bash,
    Zsh,
    Fish,
    Unknown,
}

pub fn detect_platform() -> PlatformInfo {
    let os = if cfg!(target_os = "windows") {
        Os::Windows
    } else if cfg!(target_os = "macos") {
        Os::MacOs
    } else if cfg!(target_os = "linux") {
        Os::Linux
    } else {
        Os::Unknown
    };

    let shell = detect_shell(&os);
    let is_ci = env::var("CI").is_ok()
        || env::var("GITHUB_ACTIONS").is_ok()
        || env::var("JENKINS_URL").is_ok()
        || env::var("TRAVIS").is_ok();

    use std::io::IsTerminal;
    let is_interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();

    PlatformInfo { os, shell, is_ci, is_interactive }
}

fn detect_shell(os: &Os) -> Shell {
    match os {
        Os::Windows => {
            if env::var("WT_SESSION").is_ok() || env::var("PSModulePath").is_ok() {
                Shell::PowerShell
            } else {
                Shell::Cmd
            }
        }
        _ => {
            let shell_path = env::var("SHELL").unwrap_or_default();
            if shell_path.contains("zsh") { Shell::Zsh }
            else if shell_path.contains("fish") { Shell::Fish }
            else if shell_path.contains("bash") { Shell::Bash }
            else { Shell::Unknown }
        }
    }
}
