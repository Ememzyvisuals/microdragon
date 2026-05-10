// microdragon-core/src/ipc/bridge.rs
// Module bridge — spawns and communicates with Python/Node subprocesses

use anyhow::Result;
use std::process::{Child, Command, Stdio};
use tracing::info;

pub struct ModuleBridge {
    child: Option<Child>,
    module_name: String,
}

impl ModuleBridge {
    pub fn spawn_python_module(name: &str, script: &str) -> Result<Self> {
        info!("Spawning Python module: {}", name);
        let child = Command::new("python3")
            .arg(script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Self {
            child: Some(child),
            module_name: name.to_string(),
        })
    }

    pub fn spawn_node_module(name: &str, script: &str) -> Result<Self> {
        info!("Spawning Node.js module: {}", name);
        let child = Command::new("node")
            .arg(script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(Self {
            child: Some(child),
            module_name: name.to_string(),
        })
    }

    pub fn stop(&mut self) {
        if let Some(ref mut child) = self.child {
            child.kill().ok();
        }
    }
}

impl Drop for ModuleBridge {
    fn drop(&mut self) {
        self.stop();
    }
}
