// microdragon-core/src/tools/shell.rs
//
// MICRODRAGON Shell Tool — Autonomous Command Execution
//
// This is what separates an agent from a chatbot.
// The agent can run any command, see the output, and decide what to do next.
//
// Safety: Commands are shown to the user before execution in interactive mode.
// In autonomous mode (pipeline), commands are executed and output fed back
// into the reasoning loop.
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use anyhow::{Result, anyhow};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::io::{BufRead, BufReader};
use serde::{Deserialize, Serialize};

/// Result of a shell command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellResult {
    pub command:     String,
    pub stdout:      String,
    pub stderr:      String,
    pub exit_code:   i32,
    pub success:     bool,
    pub duration_ms: u64,
}

impl ShellResult {
    /// Combined output (stdout + stderr) for AI context
    pub fn combined_output(&self) -> String {
        let mut out = String::new();
        if !self.stdout.is_empty() {
            out.push_str(&self.stdout);
        }
        if !self.stderr.is_empty() {
            if !out.is_empty() { out.push('\n'); }
            out.push_str("[stderr]\n");
            out.push_str(&self.stderr);
        }
        out
    }

    /// Short summary for display
    pub fn summary(&self) -> String {
        if self.success {
            let lines = self.stdout.lines().count();
            format!("exit 0 · {} lines output · {}ms", lines, self.duration_ms)
        } else {
            let first_err = self.stderr.lines().next().unwrap_or("no output");
            format!("exit {} · {} · {}ms", self.exit_code, first_err, self.duration_ms)
        }
    }
}

/// Execute a shell command and return its output
pub async fn run(command: &str) -> Result<ShellResult> {
    run_in_dir(command, None).await
}

/// Execute a shell command in a specific directory
pub async fn run_in_dir(command: &str, working_dir: Option<&str>) -> Result<ShellResult> {
    let start = Instant::now();

    // Parse command into program + args
    let (program, _args) = parse_command(command);

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", command]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", command]);
        c
    };

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()
        .map_err(|e| anyhow!("Failed to spawn '{}': {}", program, e))?;

    // Collect stdout
    let stdout_handle = child.stdout.take().map(|out| {
        std::thread::spawn(move || {
            let reader = BufReader::new(out);
            reader.lines()
                .filter_map(|l| l.ok())
                .collect::<Vec<String>>()
                .join("\n")
        })
    });

    // Collect stderr
    let stderr_handle = child.stderr.take().map(|err| {
        std::thread::spawn(move || {
            let reader = BufReader::new(err);
            reader.lines()
                .filter_map(|l| l.ok())
                .collect::<Vec<String>>()
                .join("\n")
        })
    });

    // Wait for process with timeout (60 seconds)
    let status = tokio::time::timeout(
        Duration::from_secs(60),
        tokio::task::spawn_blocking(move || child.wait()),
    )
    .await
    .map_err(|_| anyhow!("Command timed out after 60s: {}", command))?
    .map_err(|e| anyhow!("Join error: {}", e))?
    .map_err(|e| anyhow!("Process wait failed: {}", e))?;

    let stdout = stdout_handle
        .and_then(|h| h.join().ok())
        .unwrap_or_default();

    let stderr = stderr_handle
        .and_then(|h| h.join().ok())
        .unwrap_or_default();

    let exit_code = status.code().unwrap_or(-1);

    Ok(ShellResult {
        command: command.to_string(),
        stdout: truncate_output(stdout, 8000),
        stderr: truncate_output(stderr, 4000),
        exit_code,
        success: exit_code == 0,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Parse command string into (program, args)
fn parse_command(cmd: &str) -> (String, Vec<String>) {
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let program = parts.first().unwrap_or(&"sh").to_string();
    let args = parts.get(1)
        .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
        .unwrap_or_default();
    (program, args)
}

fn truncate_output(s: String, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s
    } else {
        let kept = &s[..max_chars];
        format!("{}\n[... truncated, {} total chars]", kept, s.len())
    }
}

// ─── Command classifier ───────────────────────────────────────────────────────

/// Classify a proposed command by safety level
#[derive(Debug, Clone, PartialEq)]
pub enum CommandSafety {
    /// Safe — read-only, status commands
    Safe,
    /// Moderate — writes files, installs packages
    Moderate,
    /// Destructive — deletes, formats, drops databases
    Destructive,
}

pub fn classify_safety(cmd: &str) -> CommandSafety {
    let lower = cmd.to_lowercase();

    // Destructive patterns
    let destructive = [
        "rm -rf", "rmdir", "drop table", "drop database",
        "format ", "mkfs", "dd if=", "> /dev/", "chmod 000",
        "userdel", ":(){:|:&};:", "shutdown", "reboot",
    ];
    if destructive.iter().any(|d| lower.contains(d)) {
        return CommandSafety::Destructive;
    }

    // Moderate (write/install) patterns
    let moderate = [
        "npm install", "npm i ", "pip install", "cargo install",
        "cargo build", "cargo test", "npm run", "yarn", "pnpm",
        "git add", "git commit", "git push", "git pull",
        "mkdir", "touch", "cp ", "mv ", "cat >", "echo >",
        "wget", "curl -o", "apt install", "brew install",
        "npx", "python ", "node ", "cargo run",
    ];
    if moderate.iter().any(|m| lower.contains(m)) {
        return CommandSafety::Moderate;
    }

    CommandSafety::Safe
}

// ─── Command builder ──────────────────────────────────────────────────────────

/// Parse AI response to extract shell commands it wants to run
/// Looks for patterns like: `$ command`, ```bash\ncommand```, `run: command`
pub fn extract_commands_from_ai(response: &str) -> Vec<String> {
    let mut commands = Vec::new();

    for line in response.lines() {
        let trimmed = line.trim();

        // `$ command` pattern
        if let Some(cmd) = trimmed.strip_prefix("$ ") {
            if !cmd.is_empty() && !cmd.starts_with('#') {
                commands.push(cmd.to_string());
                continue;
            }
        }

        // `> command` pattern
        if let Some(cmd) = trimmed.strip_prefix("> ") {
            if !cmd.is_empty() && is_shell_command(cmd) {
                commands.push(cmd.to_string());
                continue;
            }
        }
    }

    // Also extract from ```bash blocks
    let mut in_bash = false;
    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```bash") || trimmed.starts_with("```sh") || trimmed.starts_with("```shell") {
            in_bash = true;
            continue;
        }
        if trimmed == "```" && in_bash {
            in_bash = false;
            continue;
        }
        if in_bash && !trimmed.is_empty() && !trimmed.starts_with('#') {
            commands.push(trimmed.to_string());
        }
    }

    // Deduplicate preserving order
    let mut seen = std::collections::HashSet::new();
    commands.retain(|c| seen.insert(c.clone()));
    commands
}

fn is_shell_command(s: &str) -> bool {
    let shell_programs = [
        "npm", "npx", "node", "cargo", "git", "pip", "python",
        "python3", "rustc", "go", "make", "cmake", "docker",
        "kubectl", "terraform", "ls", "cat", "grep", "find",
        "mkdir", "cd", "cp", "mv", "rm", "curl", "wget",
        "yarn", "pnpm", "deno", "bun",
    ];
    let first_word = s.split_whitespace().next().unwrap_or("");
    shell_programs.contains(&first_word)
}

// ─── Autonomous task sequences ────────────────────────────────────────────────

/// Common autonomous task sequences the agent runs automatically
pub struct TaskSequence;

impl TaskSequence {
    /// After generating a Node.js project
    pub fn nodejs_setup(dir: &str) -> Vec<(&'static str, String)> {
        vec![
            ("Installing dependencies", format!("npm install --prefix {}", dir)),
            ("Running lint check", format!("npm run lint --prefix {} --if-present", dir)),
            ("Running tests", format!("npm test --prefix {} --if-present", dir)),
        ]
    }

    /// After generating a Rust project
    pub fn rust_setup(dir: &str) -> Vec<(&'static str, String)> {
        vec![
            ("Checking compilation", format!("cargo check --manifest-path {}/Cargo.toml", dir)),
            ("Running tests", format!("cargo test --manifest-path {}/Cargo.toml", dir)),
        ]
    }

    /// Git operations sequence
    pub fn git_push(message: &str) -> Vec<(&'static str, String)> {
        vec![
            ("Staging changes", "git add .".to_string()),
            ("Committing", format!("git commit -m \"{}\"", message)),
            ("Pushing to origin", "git push origin main".to_string()),
        ]
    }

    /// Python project setup
    pub fn python_setup(dir: &str) -> Vec<(&'static str, String)> {
        vec![
            ("Installing dependencies", format!("pip install -r {}/requirements.txt", dir)),
            ("Running tests", format!("python -m pytest {}", dir)),
        ]
    }
}
