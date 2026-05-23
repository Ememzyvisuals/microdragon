// microdragon-core/src/tools/modules.rs
//
// MICRODRAGON Module Bridge
// Wires every Python module to the Rust pipeline via shell execution.
// This is what makes ALL features real — the Rust agent generates content,
// then calls the right Python module to produce actual files or take actions.
//
// Features wired here:
//   • Document creation (docx, xlsx, pdf, pptx)
//   • Design (HTML, SVG, Pillow, Photoshop, GIMP)
//   • Voice (STT listen, TTS speak, wake word)
//   • Gaming (play any game, stop, status)
//   • GitHub (PR review, create issue, repo stats)
//   • Apps (open Photoshop, Excel, Word via OS)
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::tools::shell::{self, ShellResult};

// ─── Module root detection ────────────────────────────────────────────────────

/// Find the microdragon modules directory from wherever the binary is running
fn modules_dir() -> PathBuf {
    // Try: binary location / ../../modules
    if let Ok(exe) = std::env::current_exe() {
        let candidate = exe
            .parent().unwrap_or(Path::new("."))
            .parent().unwrap_or(Path::new("."))
            .parent().unwrap_or(Path::new("."))
            .join("modules");
        if candidate.exists() {
            return candidate;
        }
    }

    // Try: cwd/modules
    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join("modules");
        if candidate.exists() {
            return candidate;
        }
    }

    // Fallback
    PathBuf::from("modules")
}

fn python() -> &'static str {
    if cfg!(target_os = "windows") { "python" } else { "python3" }
}

fn module_script(sub: &str) -> String {
    modules_dir().join(sub).to_string_lossy().to_string()
}

// ─── Document creation ────────────────────────────────────────────────────────

pub struct DocumentModule;

impl DocumentModule {
    /// Create a real Word document (.docx)
    pub async fn create_docx(title: &str, content: &str, output_path: &str) -> Result<ShellResult> {
        let script = module_script("document/create.py");
        let cmd = format!(
            r#"{} "{}" docx "{}" "{}" "{}""#,
            python(), script,
            title.replace('"', "'"),
            content.replace('\n', "\\n").replace('"', "'"),
            output_path
        );
        shell::run(&cmd).await
    }

    /// Create a real Excel spreadsheet (.xlsx)
    pub async fn create_xlsx(title: &str, content: &str, output_path: &str) -> Result<ShellResult> {
        let script = module_script("document/create.py");
        let cmd = format!(
            r#"{} "{}" xlsx "{}" "{}" "{}""#,
            python(), script,
            title.replace('"', "'"),
            content.replace('\n', "\\n").replace('"', "'"),
            output_path
        );
        shell::run(&cmd).await
    }

    /// Create a real PDF
    pub async fn create_pdf(title: &str, content: &str, output_path: &str) -> Result<ShellResult> {
        let script = module_script("document/create.py");
        let cmd = format!(
            r#"{} "{}" pdf "{}" "{}" "{}""#,
            python(), script,
            title.replace('"', "'"),
            content.replace('\n', "\\n").replace('"', "'"),
            output_path
        );
        shell::run(&cmd).await
    }

    /// Create a PowerPoint presentation (.pptx)
    pub async fn create_pptx(title: &str, content: &str, output_path: &str) -> Result<ShellResult> {
        let script = module_script("document/create.py");
        let cmd = format!(
            r#"{} "{}" pptx "{}" "{}" "{}""#,
            python(), script,
            title.replace('"', "'"),
            content.replace('\n', "\\n").replace('"', "'"),
            output_path
        );
        shell::run(&cmd).await
    }

    /// Install document dependencies
    pub async fn install_deps() -> Result<ShellResult> {
        shell::run(&format!(
            "{} -m pip install python-docx openpyxl reportlab python-pptx pdfplumber --quiet",
            python()
        )).await
    }
}

// ─── Design module ────────────────────────────────────────────────────────────

pub struct DesignModule;

impl DesignModule {
    /// Create an HTML/CSS design file
    pub async fn create_html(content: &str, output_path: &str) -> Result<ShellResult> {
        use crate::tools::filesystem;
        filesystem::write_file(output_path, content)?;
        Ok(ShellResult {
            command: format!("write_html {}", output_path),
            stdout: format!("✓ HTML file created: {} ({:.1}KB)", output_path, content.len() as f64 / 1024.0),
            stderr: String::new(),
            exit_code: 0,
            success: true,
            duration_ms: 0,
        })
    }

    /// Create an SVG graphic
    pub async fn create_svg(content: &str, output_path: &str) -> Result<ShellResult> {
        use crate::tools::filesystem;
        filesystem::write_file(output_path, content)?;
        Ok(ShellResult {
            command: format!("write_svg {}", output_path),
            stdout: format!("✓ SVG file created: {}", output_path),
            stderr: String::new(),
            exit_code: 0,
            success: true,
            duration_ms: 0,
        })
    }

    /// Create a design using Pillow (programmatic image)
    pub async fn create_image(task: &str, output_path: &str, width: u32, height: u32) -> Result<ShellResult> {
        let script = module_script("apps/src/engine.py");
        let cmd = format!(
            r#"{} -c "
import asyncio, sys
sys.path.insert(0, '{}')
from engine import ImageEditor
async def main():
    ed = ImageEditor()
    result = await ed.create_design('{}', '{}', {}, {})
    print(result.output_path if result.success else result.error)
asyncio.run(main())
""#,
            python(),
            modules_dir().join("apps/src").to_string_lossy(),
            task.replace('\'', "\\'"),
            output_path,
            width,
            height
        );
        shell::run(&cmd).await
    }

    /// Open a design file in the default app
    pub async fn open_file(path: &str) -> Result<ShellResult> {
        let cmd = if cfg!(target_os = "windows") {
            format!("start \"\" \"{}\"", path)
        } else if cfg!(target_os = "macos") {
            format!("open \"{}\"", path)
        } else {
            format!("xdg-open \"{}\"", path)
        };
        shell::run(&cmd).await
    }
}

// ─── Voice module ─────────────────────────────────────────────────────────────

pub struct VoiceModule;

impl VoiceModule {
    /// Speak text aloud using configured TTS
    pub async fn speak(text: &str, ai_provider: &str) -> Result<ShellResult> {
        let script = module_script("voice/src/engine.py");
        let cmd = format!(
            r#"{} -c "
import asyncio, sys
sys.path.insert(0, '{}')
from engine import VoiceEngine
async def main():
    ve = VoiceEngine('{}')
    ok = await ve.say('{}')
    print('Speaking...' if ok else 'TTS not available')
asyncio.run(main())
""#,
            python(),
            modules_dir().join("voice/src").to_string_lossy(),
            ai_provider,
            text.replace('\'', "\\'").chars().take(500).collect::<String>()
        );
        shell::run(&cmd).await
    }

    /// Listen to microphone and transcribe
    pub async fn listen(duration_secs: u32, ai_provider: &str) -> Result<ShellResult> {
        let script = module_script("voice/src/engine.py");
        let cmd = format!(
            r#"{} -c "
import asyncio, sys
sys.path.insert(0, '{}')
from engine import VoiceEngine
async def main():
    ve = VoiceEngine('{}')
    text = await ve.listen({})
    print(text)
asyncio.run(main())
""#,
            python(),
            modules_dir().join("voice/src").to_string_lossy(),
            ai_provider,
            duration_secs
        );
        shell::run(&cmd).await
    }

    /// Get voice setup info
    pub async fn setup_info(ai_provider: &str) -> Result<ShellResult> {
        let cmd = format!(
            r#"{} -c "
import sys
sys.path.insert(0, '{}')
from engine import VoiceEngine
ve = VoiceEngine('{}')
print(ve.get_setup_notice())
""#,
            python(),
            modules_dir().join("voice/src").to_string_lossy(),
            ai_provider
        );
        shell::run(&cmd).await
    }

    /// Install voice dependencies
    pub async fn install_deps() -> Result<ShellResult> {
        shell::run(&format!(
            "{} -m pip install pyaudio faster-whisper groq openai pyttsx3 --quiet",
            python()
        )).await
    }
}

// ─── Gaming module ────────────────────────────────────────────────────────────

pub struct GamingModule;

impl GamingModule {
    /// Start MICRODRAGON playing a game
    pub async fn play(game_name: &str, duration_secs: u32) -> Result<ShellResult> {
        let script = module_script("gaming/src/cli_commands.py");
        let cmd = format!(
            r#"{} -c "
import asyncio, sys
sys.path.insert(0, '{}')
from cli_commands import cmd_play
result = asyncio.run(cmd_play('{}', {}))
print(result)
""#,
            python(),
            modules_dir().join("gaming/src").to_string_lossy(),
            game_name.replace('\'', "\\'"),
            duration_secs
        );
        shell::run(&cmd).await
    }

    /// Stop active game session
    pub async fn stop() -> Result<ShellResult> {
        let cmd = format!(
            r#"{} -c "
import sys
sys.path.insert(0, '{}')
from cli_commands import cmd_stop
print(cmd_stop())
""#,
            python(),
            modules_dir().join("gaming/src").to_string_lossy()
        );
        shell::run(&cmd).await
    }

    /// Get status of active game session
    pub async fn status() -> Result<ShellResult> {
        let cmd = format!(
            r#"{} -c "
import sys
sys.path.insert(0, '{}')
from cli_commands import cmd_status, cmd_list_games
print(cmd_status())
print()
print(cmd_list_games())
""#,
            python(),
            modules_dir().join("gaming/src").to_string_lossy()
        );
        shell::run(&cmd).await
    }

    /// Install gaming dependencies
    pub async fn install_deps() -> Result<ShellResult> {
        shell::run(&format!(
            "{} -m pip install mss pynput opencv-python-headless numpy --quiet",
            python()
        )).await
    }
}

// ─── GitHub module ────────────────────────────────────────────────────────────

pub struct GitHubModule;

impl GitHubModule {
    /// Review a pull request
    pub async fn review_pr(pr_url: &str, post_review: bool) -> Result<ShellResult> {
        let cmd = format!(
            r#"{} -c "
import asyncio, sys
sys.path.insert(0, '{}')
from engine import GitHubAgent
async def main():
    agent = GitHubAgent()
    result = await agent.review_pr('{}', post_review={})
    print(result)
asyncio.run(main())
""#,
            python(),
            modules_dir().join("github/src").to_string_lossy(),
            pr_url,
            if post_review { "True" } else { "False" }
        );
        shell::run(&cmd).await
    }

    /// Get repo overview
    pub async fn repo_overview(owner: &str, repo: &str) -> Result<ShellResult> {
        let cmd = format!(
            r#"{} -c "
import asyncio, sys
sys.path.insert(0, '{}')
from engine import GitHubAgent
async def main():
    agent = GitHubAgent()
    result = await agent.repo_overview('{}', '{}')
    print(result)
asyncio.run(main())
""#,
            python(),
            modules_dir().join("github/src").to_string_lossy(),
            owner, repo
        );
        shell::run(&cmd).await
    }

    /// Create issue
    pub async fn create_issue(owner: &str, repo: &str, title: &str, body: &str) -> Result<ShellResult> {
        let cmd = format!(
            r#"{} -c "
import asyncio, sys
sys.path.insert(0, '{}')
from engine import GitHubAgent
async def main():
    agent = GitHubAgent()
    url = await agent.create_issue('{}', '{}', '{}', '{}')
    print(f'Issue created: {{url}}')
asyncio.run(main())
""#,
            python(),
            modules_dir().join("github/src").to_string_lossy(),
            owner, repo,
            title.replace('\'', "\\'"),
            body.replace('\'', "\\'")
        );
        shell::run(&cmd).await
    }
}

// ─── App launcher ─────────────────────────────────────────────────────────────

pub struct AppLauncher;

impl AppLauncher {
    /// Open any application by name
    pub async fn open(app_name: &str) -> Result<ShellResult> {
        let cmd = if cfg!(target_os = "windows") {
            format!("start \"\" \"{}\"", app_name)
        } else if cfg!(target_os = "macos") {
            format!("open -a \"{}\"", app_name)
        } else {
            format!("{} &", app_name.to_lowercase())
        };
        shell::run(&cmd).await
    }

    /// Open a file with its default application
    pub async fn open_file(path: &str) -> Result<ShellResult> {
        let cmd = if cfg!(target_os = "windows") {
            format!("start \"\" \"{}\"", path)
        } else if cfg!(target_os = "macos") {
            format!("open \"{}\"", path)
        } else {
            format!("xdg-open \"{}\"", path)
        };
        shell::run(&cmd).await
    }

    /// Open Photoshop with a file
    pub async fn photoshop(file_path: Option<&str>) -> Result<ShellResult> {
        let cmd = format!(
            r#"{} -c "
import sys
sys.path.insert(0, '{}')
from engine import AppRegistry
import subprocess
ps = AppRegistry.photoshop_path()
if ps:
    subprocess.Popen([ps{}])
    print(f'Opened Photoshop: {{ps}}')
else:
    print('Photoshop not found on this system')
""#,
            python(),
            modules_dir().join("apps/src").to_string_lossy(),
            if let Some(f) = file_path {
                format!(", '{}'", f)
            } else {
                String::new()
            }
        );
        shell::run(&cmd).await
    }
}
