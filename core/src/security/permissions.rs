// microdragon-core/src/security/permissions.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Capability {
    FileRead,
    FileWrite,
    FileDelete,
    ShellSafe,
    ShellPrivileged,
    NetworkRead,
    NetworkWrite,
    ProcessSpawn,
    ScreenCapture,
    InputSimulation,
    EmailSend,
    SocialPost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    pub allowed: Vec<Capability>,
}

impl PermissionSet {
    pub fn default_safe() -> Self {
        Self {
            allowed: vec![
                Capability::FileRead,
                Capability::FileWrite,
                Capability::ShellSafe,
                Capability::NetworkRead,
                Capability::ProcessSpawn,
                Capability::ScreenCapture,
            ],
        }
    }

    pub fn full() -> Self {
        Self {
            allowed: vec![
                Capability::FileRead,   Capability::FileWrite,
                Capability::FileDelete, Capability::ShellSafe,
                Capability::ShellPrivileged, Capability::NetworkRead,
                Capability::NetworkWrite, Capability::ProcessSpawn,
                Capability::ScreenCapture, Capability::InputSimulation,
                Capability::EmailSend, Capability::SocialPost,
            ],
        }
    }

    pub fn has(&self, cap: &Capability) -> bool {
        self.allowed.contains(cap)
    }
}
