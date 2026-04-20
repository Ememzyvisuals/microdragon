// microdragon-core/src/security/sandbox.rs
// Sandboxed command execution — prevents the OpenClaw "deletes Gmail inbox" class of bugs

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};

pub struct ExecutionSandbox {
    enabled: bool,
    allowed_dirs: Vec<PathBuf>,
    blocked_commands: Vec<String>,
}

impl ExecutionSandbox {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            allowed_dirs: vec![
                dirs::home_dir().unwrap_or_default().join("microdragon_workspace"),
                std::env::temp_dir(),
            ],
            blocked_commands: vec![
                // Dangerous commands blocked by default
                "rm -rf /".to_string(),
                "mkfs".to_string(),
                "dd if=/dev/zero".to_string(),
                ":(){ :|:& };:".to_string(), // fork bomb
                "chmod -R 777 /".to_string(),
                "sudo rm".to_string(),
            ],
        }
    }

    pub fn check_command(&self, cmd: &str) -> Result<()> {
        if !self.enabled { return Ok(()); }

        let cmd_lower = cmd.to_lowercase();
        for blocked in &self.blocked_commands {
            if cmd_lower.contains(&blocked.to_lowercase()) {
                return Err(anyhow!("BLOCKED: command '{}' is not allowed in sandbox mode", blocked));
            }
        }

        // Warn on high-risk patterns
        let high_risk = ["rm ", "del ", "format ", "fdisk", "shutdown", "reboot", "halt"];
        for pat in &high_risk {
            if cmd_lower.contains(pat) {
                warn!("High-risk command pattern detected: {}", pat);
                // Require explicit confirmation — handled by caller
                return Err(anyhow!("CONFIRM_REQUIRED: '{}' requires explicit confirmation", cmd));
            }
        }

        Ok(())
    }

    pub fn safe_run(&self, cmd: &str, args: &[&str]) -> Result<String> {
        self.check_command(cmd)?;

        let output = Command::new(cmd)
            .args(args)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!("{}", String::from_utf8_lossy(&output.stderr)))
        }
    }
}
