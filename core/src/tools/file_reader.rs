// microdragon-core/src/tools/file_reader.rs
// Read files for the document and security commands.
// Handles: text, code, CSV, JSON, TOML, YAML — any UTF-8 file.
// For PDF/DOCX: returns instructions to install the Python module.
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use anyhow::{Result, anyhow};
use std::path::Path;

pub struct FileContent {
    pub path:        String,
    pub extension:   String,
    pub content:     String,
    pub size_kb:     f64,
    pub line_count:  usize,
}

pub fn read_file(path: &str) -> Result<FileContent> {
    let p = Path::new(path);

    if !p.exists() {
        return Err(anyhow!("File not found: {}", path));
    }

    let ext = p.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let meta = std::fs::metadata(p)?;
    let size_kb = meta.len() as f64 / 1024.0;

    // Binary / requires Python module
    match ext.as_str() {
        "pdf" => return Err(anyhow!(
            "PDF reading requires the Python module:\n\
             pip install pdfplumber\n\
             Then use: python3 modules/document/read_pdf.py \"{}\"",
            path
        )),
        "docx" | "doc" => return Err(anyhow!(
            "Word document reading requires the Python module:\n\
             pip install python-docx\n\
             Then use: python3 modules/document/read_docx.py \"{}\"",
            path
        )),
        "xlsx" | "xls" => return Err(anyhow!(
            "Excel reading requires the Python module:\n\
             pip install openpyxl\n\
             Then use: python3 modules/document/read_excel.py \"{}\"",
            path
        )),
        _ => {}
    }

    // Attempt UTF-8 text read
    let content = std::fs::read_to_string(p).map_err(|e| {
        anyhow!("Cannot read {}: {}. If this is a binary file, convert it to text first.", path, e)
    })?;

    let line_count = content.lines().count();

    // Truncate very large files (>200KB) to avoid context overflow
    let content = if content.len() > 200_000 {
        let truncated = &content[..200_000];
        format!(
            "{}\n\n[... FILE TRUNCATED at 200KB. Full file is {:.1}KB. Showing first 200KB.]\n",
            truncated, size_kb
        )
    } else {
        content
    };

    Ok(FileContent {
        path: path.to_string(),
        extension: ext,
        content,
        size_kb,
        line_count,
    })
}

/// Read all code files in a directory (recursively, up to 50 files)
pub fn read_directory(dir: &str, max_files: usize) -> Result<Vec<FileContent>> {
    let p = Path::new(dir);
    if !p.is_dir() {
        // Single file
        return Ok(vec![read_file(dir)?]);
    }

    let code_extensions = [
        "rs", "py", "js", "ts", "go", "java", "c", "cpp", "h",
        "cs", "php", "rb", "swift", "kt", "dart", "sh", "bash",
        "sql", "html", "css", "jsx", "tsx", "vue", "svelte",
        "toml", "yaml", "yml", "json", "env", "md",
    ];

    let mut files = Vec::new();
    collect_files(p, &code_extensions, &mut files, max_files)?;

    if files.is_empty() {
        return Err(anyhow!("No readable code files found in {}", dir));
    }

    Ok(files)
}

fn collect_files(
    dir: &Path,
    extensions: &[&str],
    out: &mut Vec<FileContent>,
    max: usize,
) -> Result<()> {
    if out.len() >= max { return Ok(()); }

    let entries = std::fs::read_dir(dir)?;
    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();
    paths.sort();

    for path in paths {
        if out.len() >= max { break; }
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') && name != "node_modules" && name != "target" && name != "__pycache__" {
                collect_files(&path, extensions, out, max)?;
            }
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.contains(&ext.to_lowercase().as_str()) {
                if let Ok(fc) = read_file(path.to_str().unwrap_or("")) {
                    out.push(fc);
                }
            }
        }
    }

    Ok(())
}

/// Format file contents for AI context
pub fn format_for_ai(files: &[FileContent]) -> String {
    let mut out = String::new();

    for f in files {
        out.push_str(&format!(
            "### File: {} ({:.1}KB, {} lines)\n```{}\n{}\n```\n\n",
            f.path, f.size_kb, f.line_count, f.extension, f.content
        ));
    }

    out
}
