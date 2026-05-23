// microdragon-core/src/tools/filesystem.rs
//
// MICRODRAGON File System Tool
//
// The agent's ability to navigate and operate on the real PC file system.
// This is what makes MICRODRAGON different from a chatbot:
//   - Knows where it is (cwd)
//   - Reads ANY file type on the machine
//   - Creates files and folders anywhere
//   - Navigates the full directory tree
//   - Writes generated code directly to disk
//   - Finds files by pattern across the whole drive
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

// ─── Directory listing ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub name:       String,
    pub path:       String,
    pub kind:       EntryKind,
    pub size_kb:    f64,
    pub extension:  String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
}

/// List the contents of a directory (up to max_entries)
pub fn list_dir(path: &str, max_entries: usize) -> Result<Vec<DirEntry>> {
    let p = resolve_path(path)?;

    if !p.is_dir() {
        return Err(anyhow!("'{}' is not a directory", path));
    }

    let mut entries = Vec::new();
    let read = fs::read_dir(&p)
        .map_err(|e| anyhow!("Cannot read '{}': {}", path, e))?;

    let mut raw: Vec<_> = read
        .filter_map(|e| e.ok())
        .collect();

    // Dirs first, then files, both alphabetical
    raw.sort_by(|a, b| {
        let a_dir = a.path().is_dir();
        let b_dir = b.path().is_dir();
        if a_dir == b_dir {
            a.file_name().cmp(&b.file_name())
        } else if a_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater }
    });

    for entry in raw.into_iter().take(max_entries) {
        let ep = entry.path();
        let name = ep.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if name.starts_with('.') { continue; } // skip hidden

        let ext = ep.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let size_kb = if ep.is_file() {
            ep.metadata().map(|m| m.len() as f64 / 1024.0).unwrap_or(0.0)
        } else { 0.0 };

        let kind = if ep.is_dir() {
            EntryKind::Directory
        } else if ep.is_symlink() {
            EntryKind::Symlink
        } else {
            EntryKind::File
        };

        entries.push(DirEntry {
            name,
            path: ep.to_string_lossy().to_string(),
            kind,
            size_kb,
            extension: ext,
        });
    }

    Ok(entries)
}

/// Format directory listing for display and AI context
pub fn format_dir_listing(path: &str, entries: &[DirEntry]) -> String {
    let mut out = format!("📁  {}\n\n", path);
    for e in entries {
        let icon = match e.kind {
            EntryKind::Directory => "📁",
            EntryKind::File      => file_icon(&e.extension),
            EntryKind::Symlink   => "🔗",
        };
        let size = if e.kind == EntryKind::File {
            if e.size_kb > 1024.0 {
                format!("  {:.1}MB", e.size_kb / 1024.0)
            } else {
                format!("  {:.0}KB", e.size_kb)
            }
        } else {
            String::new()
        };
        out.push_str(&format!("  {}  {}{}\n", icon, e.name, size));
    }
    out
}

fn file_icon(ext: &str) -> &'static str {
    match ext {
        "rs"  => "🦀", "py"  => "🐍", "js" | "ts" => "📜",
        "go"  => "🐹", "java" => "☕", "cpp" | "c" => "⚙️",
        "md"  => "📝", "txt" => "📄", "json" => "🗂️",
        "toml" | "yaml" | "yml" => "⚙️",
        "pdf" => "📕", "docx" | "doc" => "📘",
        "xlsx" | "xls" => "📗", "pptx" | "ppt" => "📙",
        "png" | "jpg" | "jpeg" | "gif" | "svg" => "🖼️",
        "mp4" | "mov" | "avi" => "🎬",
        "mp3" | "wav" | "ogg" => "🎵",
        "zip" | "tar" | "gz" => "📦",
        "html" | "css" => "🌐",
        "sh" | "bash" | "ps1" => "💻",
        _ => "📄",
    }
}

// ─── File reading ─────────────────────────────────────────────────────────────

/// Read any readable file from anywhere on the PC
pub fn read_file(path: &str) -> Result<String> {
    let p = resolve_path(path)?;

    if !p.exists() {
        return Err(anyhow!("File not found: {}", path));
    }

    if p.is_dir() {
        let entries = list_dir(path, 50)?;
        return Ok(format_dir_listing(path, &entries));
    }

    let ext = p.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Binary formats — call Python helper or explain
    match ext.as_str() {
        "pdf" => return read_with_python(&p, "pdf"),
        "docx" | "doc" => return read_with_python(&p, "docx"),
        "xlsx" | "xls" => return read_with_python(&p, "xlsx"),
        "pptx" | "ppt" => return read_with_python(&p, "pptx"),
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" => {
            let size_kb = p.metadata().map(|m| m.len() as f64 / 1024.0).unwrap_or(0.0);
            return Ok(format!("[Image file: {} ({:.1}KB)]\nTo analyse image contents, describe what you see or use a vision-capable model.", path, size_kb));
        }
        "mp4" | "avi" | "mov" | "mkv" | "mp3" | "wav" => {
            return Ok(format!("[Media file: {} — cannot read binary content]", path));
        }
        _ => {}
    }

    // Read as UTF-8 text
    let content = fs::read_to_string(&p)
        .map_err(|e| anyhow!("Cannot read '{}': {}", path, e))?;

    // Truncate very large files
    if content.len() > 200_000 {
        Ok(format!("{}\n\n[... truncated at 200KB. File is {:.0}KB total]",
            &content[..200_000],
            content.len() as f64 / 1024.0))
    } else {
        Ok(content)
    }
}

fn read_with_python(path: &Path, format: &str) -> Result<String> {
    let path_str = path.to_string_lossy();

    let script = match format {
        "pdf" => format!(
            r#"
try:
    import pdfplumber
    with pdfplumber.open(r"{}") as pdf:
        text = "\n\n".join(p.extract_text() or "" for p in pdf.pages)
    print(text[:8000])
except ImportError:
    try:
        import PyPDF2
        with open(r"{}", "rb") as f:
            r = PyPDF2.PdfReader(f)
            print("\n".join(p.extract_text() or "" for p in r.pages)[:8000])
    except:
        print("[PDF reader not available. pip install pdfplumber]")
except Exception as e:
    print(f"[Error reading PDF: {{e}}]")
"#, path_str, path_str),

        "docx" => format!(
            r#"
try:
    from docx import Document
    doc = Document(r"{}")
    text = "\n".join(p.text for p in doc.paragraphs if p.text.strip())
    print(text[:8000])
except ImportError:
    print("[python-docx not installed. pip install python-docx]")
except Exception as e:
    print(f"[Error reading DOCX: {{e}}]")
"#, path_str),

        "xlsx" => format!(
            r#"
try:
    import openpyxl
    wb = openpyxl.load_workbook(r"{}", read_only=True)
    out = []
    for ws in wb.worksheets:
        out.append(f"Sheet: {{ws.title}}")
        for row in list(ws.iter_rows(values_only=True))[:50]:
            out.append("\t".join(str(c) if c is not None else "" for c in row))
    print("\n".join(out)[:8000])
except ImportError:
    print("[openpyxl not installed. pip install openpyxl]")
except Exception as e:
    print(f"[Error reading XLSX: {{e}}]")
"#, path_str),

        _ => format!("print('[Cannot read {} files]')", format),
    };

    let result = std::process::Command::new("python3")
        .args(["-c", &script])
        .output();

    match result {
        Ok(out) => Ok(String::from_utf8_lossy(&out.stdout).to_string()),
        Err(_) => {
            // Try python on Windows
            let result2 = std::process::Command::new("python")
                .args(["-c", &script])
                .output();
            match result2 {
                Ok(out) => Ok(String::from_utf8_lossy(&out.stdout).to_string()),
                Err(e) => Err(anyhow!("Python not available: {}", e)),
            }
        }
    }
}

// ─── File writing ─────────────────────────────────────────────────────────────

/// Write content to a file, creating parent directories as needed
pub fn write_file(path: &str, content: &str) -> Result<()> {
    let p = resolve_path(path)?;

    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow!("Cannot create directory '{}': {}", parent.display(), e))?;
    }

    fs::write(&p, content)
        .map_err(|e| anyhow!("Cannot write to '{}': {}", path, e))?;

    Ok(())
}

/// Create a directory (including all parents)
pub fn create_dir(path: &str) -> Result<()> {
    let p = resolve_path(path)?;
    fs::create_dir_all(&p)
        .map_err(|e| anyhow!("Cannot create directory '{}': {}", path, e))
}

/// Create an entire project scaffold
pub fn create_project_structure(base: &str, structure: &[(&str, Option<&str>)]) -> Result<Vec<String>> {
    let base_path = resolve_path(base)?;
    fs::create_dir_all(&base_path)?;

    let mut created = Vec::new();

    for (relative_path, content) in structure {
        let full_path = base_path.join(relative_path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if let Some(text) = content {
            fs::write(&full_path, text)?;
            created.push(format!("  ✓ {}", relative_path));
        } else {
            fs::create_dir_all(&full_path)?;
            created.push(format!("  📁 {}/", relative_path));
        }
    }

    Ok(created)
}

// ─── File search ──────────────────────────────────────────────────────────────

/// Find files matching a pattern (by name or extension)
pub fn find_files(root: &str, pattern: &str, max_results: usize) -> Result<Vec<String>> {
    let p = resolve_path(root)?;
    let pattern_lower = pattern.to_lowercase();
    let mut results = Vec::new();

    find_recursive(&p, &pattern_lower, &mut results, max_results);
    Ok(results)
}

fn find_recursive(dir: &Path, pattern: &str, results: &mut Vec<String>, max: usize) {
    if results.len() >= max { return; }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        if results.len() >= max { break; }

        let path = entry.path();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if name.starts_with('.') { continue; }

        let name_lower = name.to_lowercase();

        // Match by name or extension
        if name_lower.contains(pattern)
            || path.extension().and_then(|e| e.to_str()).map(|e| e == pattern).unwrap_or(false)
        {
            results.push(path.to_string_lossy().to_string());
        }

        if path.is_dir() && !name.eq_ignore_ascii_case("node_modules")
            && !name.eq_ignore_ascii_case("target")
            && !name.eq_ignore_ascii_case(".git")
        {
            find_recursive(&path, pattern, results, max);
        }
    }
}

// ─── Path resolution ──────────────────────────────────────────────────────────

/// Resolve a path, expanding ~ and relative paths
pub fn resolve_path(path: &str) -> Result<PathBuf> {
    let expanded = if path.starts_with('~') {
        let home = home_dir()?;
        PathBuf::from(home).join(&path[2..])
    } else {
        PathBuf::from(path)
    };

    Ok(expanded)
}

/// Get user home directory
pub fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))
}

/// Get Desktop path
pub fn desktop_dir() -> Result<PathBuf> {
    dirs::desktop_dir()
        .ok_or_else(|| anyhow!("Cannot determine Desktop directory"))
}

/// Get Downloads path
pub fn downloads_dir() -> Result<PathBuf> {
    dirs::download_dir()
        .ok_or_else(|| anyhow!("Cannot determine Downloads directory"))
}

/// Get Documents path
pub fn documents_dir() -> Result<PathBuf> {
    dirs::document_dir()
        .ok_or_else(|| anyhow!("Cannot determine Documents directory"))
}

/// Current working directory
pub fn current_dir() -> Result<PathBuf> {
    std::env::current_dir().map_err(|e| anyhow!("Cannot get cwd: {}", e))
}

/// PC overview — paths and sizes
pub fn pc_overview() -> String {
    let home    = home_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".into());
    let desktop = desktop_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".into());
    let docs    = documents_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".into());
    let dl      = downloads_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".into());
    let cwd     = current_dir().map(|p| p.display().to_string()).unwrap_or_else(|_| "unknown".into());

    format!(
        "PC File System Overview:\n  Home:      {}\n  Desktop:   {}\n  Documents: {}\n  Downloads: {}\n  CWD:       {}\n\nOS: {}",
        home, desktop, docs, dl, cwd,
        std::env::consts::OS
    )
}
