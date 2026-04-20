// microdragon-core/src/cli/interactive/input.rs
// LineEditor — cross-platform input with history nav, editing keys
// Uses crossterm raw mode for rich terminals, falls back to simple readline

use anyhow::Result;
use std::io::{self, Write, BufRead};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, EnableBracketedPaste},
    execute, queue,
    terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType},
    cursor::{self, MoveToColumn},
    style::{Print, Color, SetForegroundColor, ResetColor},
};
use crate::cli::terminal as term;

pub struct LineEditor {
    buffer: Vec<char>,
    cursor_pos: usize,
    use_raw: bool,
}

impl LineEditor {
    pub fn new() -> Self {
        let use_raw = term::is_rich() && !term::caps().is_pipe;
        Self {
            buffer: Vec::new(),
            cursor_pos: 0,
            use_raw,
        }
    }

    /// Read a line of input, returning None on EOF/exit
    pub async fn readline(&mut self, prompt: &str) -> Result<Option<String>> {
        if self.use_raw {
            self.readline_raw(prompt).await
        } else {
            self.readline_simple(prompt)
        }
    }

    // ─── Raw mode (rich terminals) ─────────────────────────────────────────

    async fn readline_raw(&mut self, prompt: &str) -> Result<Option<String>> {
        self.buffer.clear();
        self.cursor_pos = 0;

        // Print prompt
        self.render_prompt(prompt);

        enable_raw_mode()?;

        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                match event::read()? {
                    // ─── Enter: submit ────────────────────────
                    Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                        disable_raw_mode()?;
                        println!();
                        let result = self.buffer.iter().collect::<String>();
                        self.buffer.clear();
                        self.cursor_pos = 0;
                        return Ok(Some(result));
                    }

                    // ─── Ctrl+C / Ctrl+D: exit ────────────────
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => {
                        disable_raw_mode()?;
                        println!();
                        return Ok(None);
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('d'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) if self.buffer.is_empty() => {
                        disable_raw_mode()?;
                        println!();
                        return Ok(None);
                    }

                    // ─── Backspace ────────────────────────────
                    Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                            self.buffer.remove(self.cursor_pos);
                            self.redraw(prompt);
                        }
                    }

                    // ─── Delete ───────────────────────────────
                    Event::Key(KeyEvent { code: KeyCode::Delete, .. }) => {
                        if self.cursor_pos < self.buffer.len() {
                            self.buffer.remove(self.cursor_pos);
                            self.redraw(prompt);
                        }
                    }

                    // ─── Arrow left/right ─────────────────────
                    Event::Key(KeyEvent { code: KeyCode::Left, .. }) => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                            self.move_cursor(prompt);
                        }
                    }
                    Event::Key(KeyEvent { code: KeyCode::Right, .. }) => {
                        if self.cursor_pos < self.buffer.len() {
                            self.cursor_pos += 1;
                            self.move_cursor(prompt);
                        }
                    }

                    // ─── Home/End ─────────────────────────────
                    Event::Key(KeyEvent { code: KeyCode::Home, .. }) => {
                        self.cursor_pos = 0;
                        self.move_cursor(prompt);
                    }
                    Event::Key(KeyEvent { code: KeyCode::End, .. }) => {
                        self.cursor_pos = self.buffer.len();
                        self.move_cursor(prompt);
                    }

                    // ─── Ctrl+A (beginning) ───────────────────
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('a'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => {
                        self.cursor_pos = 0;
                        self.move_cursor(prompt);
                    }

                    // ─── Ctrl+E (end) ─────────────────────────
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('e'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => {
                        self.cursor_pos = self.buffer.len();
                        self.move_cursor(prompt);
                    }

                    // ─── Ctrl+U (clear line) ──────────────────
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('u'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => {
                        self.buffer.clear();
                        self.cursor_pos = 0;
                        self.redraw(prompt);
                    }

                    // ─── Ctrl+W (delete word) ─────────────────
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('w'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => {
                        while self.cursor_pos > 0 {
                            let ch = self.buffer[self.cursor_pos - 1];
                            self.cursor_pos -= 1;
                            self.buffer.remove(self.cursor_pos);
                            if ch == ' ' { break; }
                        }
                        self.redraw(prompt);
                    }

                    // ─── Regular character ────────────────────
                    Event::Key(KeyEvent { code: KeyCode::Char(ch), modifiers, .. })
                        if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT =>
                    {
                        self.buffer.insert(self.cursor_pos, ch);
                        self.cursor_pos += 1;
                        self.redraw(prompt);
                    }

                    _ => {}
                }
            }

            // Allow async tasks to progress (non-blocking cooperative yield)
            tokio::task::yield_now().await;
        }
    }

    fn render_prompt(&self, prompt: &str) {
        // Prompt is already ANSI-encoded
        print!("{}", prompt);
        let _ = io::stdout().flush();
    }

    fn redraw(&self, prompt: &str) {
        let line: String = self.buffer.iter().collect();
        let prompt_len = self.visible_len(prompt);
        let _ = execute!(
            io::stdout(),
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            Print(prompt),
            Print(&line),
            MoveToColumn((prompt_len + self.cursor_pos) as u16),
        );
        let _ = io::stdout().flush();
    }

    fn move_cursor(&self, prompt: &str) {
        let prompt_len = self.visible_len(prompt);
        let _ = execute!(
            io::stdout(),
            MoveToColumn((prompt_len + self.cursor_pos) as u16),
        );
        let _ = io::stdout().flush();
    }

    /// Estimate visible length (strip ANSI escape codes)
    fn visible_len(&self, s: &str) -> usize {
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        re.replace_all(s, "").len()
    }

    // ─── Simple mode (piped / dumb terminal / CMD fallback) ───────────────

    fn readline_simple(&mut self, prompt: &str) -> Result<Option<String>> {
        // Strip ANSI from prompt for plain terminals
        let clean_prompt = self.strip_ansi(prompt);
        print!("{}", clean_prompt);
        let _ = io::stdout().flush();

        let stdin = io::stdin();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => Ok(Some(line.trim_end_matches('\n').trim_end_matches('\r').to_string())),
            Err(e) => Err(e.into()),
        }
    }

    fn strip_ansi(&self, s: &str) -> String {
        let re = regex::Regex::new(r"\x1b\[[0-9;]*[mGKHF]").unwrap();
        re.replace_all(s, "").to_string()
    }
}
