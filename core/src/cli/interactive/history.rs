// microdragon-core/src/cli/interactive/history.rs
// Command history with persistence

use std::path::PathBuf;
use std::fs;

pub struct CommandHistory {
    entries: Vec<String>,
    max_size: usize,
    cursor: usize,     // for up-arrow navigation
    dirty: bool,
}

impl CommandHistory {
    pub fn new(max_size: usize) -> Self {
        let mut h = Self {
            entries: Vec::new(),
            max_size,
            cursor: 0,
            dirty: false,
        };
        h.load_from_disk();
        h
    }

    /// Add entry; skip duplicates of the previous entry
    pub fn push(&mut self, entry: String) {
        if entry.trim().is_empty() { return; }
        if self.entries.last().map_or(false, |e| e == &entry) { return; }
        if self.entries.len() >= self.max_size {
            self.entries.remove(0);
        }
        self.entries.push(entry);
        self.cursor = self.entries.len();
        self.dirty = true;
        self.save_to_disk();
    }

    pub fn recent(&self, n: usize) -> Vec<String> {
        self.entries.iter().rev().take(n).rev().cloned().collect()
    }

    pub fn prev(&mut self) -> Option<&str> {
        if self.entries.is_empty() { return None; }
        if self.cursor > 0 { self.cursor -= 1; }
        self.entries.get(self.cursor).map(|s| s.as_str())
    }

    pub fn next(&mut self) -> Option<&str> {
        if self.cursor + 1 < self.entries.len() {
            self.cursor += 1;
            self.entries.get(self.cursor).map(|s| s.as_str())
        } else {
            self.cursor = self.entries.len();
            None
        }
    }

    pub fn reset_cursor(&mut self) {
        self.cursor = self.entries.len();
    }

    pub fn all(&self) -> &[String] {
        &self.entries
    }

    fn history_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("microdragon")
            .join("history.txt")
    }

    fn load_from_disk(&mut self) {
        let path = Self::history_path();
        if let Ok(content) = fs::read_to_string(&path) {
            self.entries = content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.to_string())
                .take(self.max_size)
                .collect();
            self.cursor = self.entries.len();
        }
    }

    fn save_to_disk(&self) {
        if !self.dirty { return; }
        let path = Self::history_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = self.entries.join("\n");
        let _ = fs::write(&path, content);
    }
}
