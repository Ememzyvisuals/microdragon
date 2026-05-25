// microdragon-core/src/cli/markdown.rs
//
// Terminal Markdown Renderer — renders AI output with full color, structure,
// and code highlighting. This is what makes Microdragon look agentic instead
// of chatbot-like.
//
// Handles:
//   # H1 / ## H2 / ### H3    → colored bold headers
//   **bold** / *italic*       → terminal bold/italic
//   `inline code`             → green highlighted
//   ```lang ... ```           → framed code blocks with language label
//   - / * / + bullet lists    → colored bullets
//   1. numbered lists         → colored numbers
//   > blockquotes             → dim indented
//   ---                       → horizontal rule
//   [text](url)               → text + dim URL
//   Plain text                → word-wrapped at terminal width
//
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use std::io::{self, Write};
use crossterm::style::{Color, Stylize};
use crate::cli::terminal::CAPS;
use crate::cli::theme::Theme;

// ─── Public entry point ───────────────────────────────────────────────────────

/// Render markdown text to the terminal with full color and structure.
/// This replaces every raw `println!("{}", result.response)` in the codebase.
pub fn render(text: &str) {
    render_with_indent(text, "  ");
}

/// Render with a custom left-indent prefix (default: two spaces)
pub fn render_with_indent(text: &str, indent: &str) {
    if !CAPS.ansi_color {
        // Fallback: strip markdown, print plain
        print_plain(text, indent);
        return;
    }

    let mut stdout = io::stdout();
    println!();

    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_buf: Vec<String> = Vec::new();
    #[allow(unused_assignments)]
    let mut list_counter = 0u32;
    let mut prev_blank = true;

    for line in text.lines() {
        let trimmed = line.trim_end();

        // ── Code block boundaries ─────────────────────────────────────────
        if trimmed.starts_with("```") {
            if in_code_block {
                // Close block
                flush_code_block(&mut stdout, &code_buf, &code_lang, indent);
                code_buf.clear();
                code_lang.clear();
                in_code_block = false;
            } else {
                // Open block
                code_lang = trimmed.trim_start_matches('`').to_string();
                in_code_block = true;
                list_counter = 0;
            }
            prev_blank = false;
            continue;
        }

        if in_code_block {
            code_buf.push(line.to_string());
            continue;
        }

        // ── Blank line ────────────────────────────────────────────────────
        if trimmed.is_empty() {
            if !prev_blank {
                writeln!(stdout).ok();
            }
            prev_blank = true;
            list_counter = 0;
            continue;
        }
        prev_blank = false;

        // ── Horizontal rule ───────────────────────────────────────────────
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            let width = CAPS.width.saturating_sub(6) as usize;
            let _ = writeln!(stdout, "{}{}",
                indent,
                "─".repeat(width).with(Color::DarkGrey)
            );
            continue;
        }

        // ── Headers ───────────────────────────────────────────────────────
        if let Some(rest) = trimmed.strip_prefix("### ") {
            let _ = writeln!(stdout, "\n{}{}",
                indent,
                rest.to_string().with(Theme::EMBER).bold()
            );
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            let _ = writeln!(stdout, "\n{}{}",
                indent,
                rest.to_string().with(Theme::GREEN).bold()
            );
            // Underline
            let width = rest.chars().count().min(56);
            let _ = writeln!(stdout, "{}{}",
                indent,
                "─".repeat(width).with(Color::DarkGrey)
            );
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# ") {
            let _ = writeln!(stdout, "\n{}{}",
                indent,
                rest.to_string().with(Theme::FIRE).bold()
            );
            let width = rest.chars().count().min(56);
            let _ = writeln!(stdout, "{}{}",
                indent,
                "═".repeat(width).with(Color::DarkGrey)
            );
            continue;
        }

        // ── Blockquote ────────────────────────────────────────────────────
        if let Some(rest) = trimmed.strip_prefix("> ") {
            let _ = writeln!(stdout, "{}{}  {}",
                indent,
                "│".with(Theme::EMBER),
                render_inline(rest).with(Color::DarkGrey).italic()
            );
            continue;
        }

        // ── Unordered list ────────────────────────────────────────────────
        let is_bullet = trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ");

        if is_bullet {
            list_counter = 0;
            let rest = &trimmed[2..];
            let bullet = "◆".with(Theme::GREEN);
            let _ = write!(stdout, "{}{}  ", indent, bullet);
            print_wrapped_continuation(rest, indent, 4, &mut stdout);
            continue;
        }

        // ── Ordered list ──────────────────────────────────────────────────
        if let Some((num_str, rest)) = parse_ordered_item(trimmed) {
            list_counter = num_str;
            let num_colored = format!("{}.", list_counter).with(Theme::EMBER).bold();
            let _ = write!(stdout, "{}{}  ", indent, num_colored);
            print_wrapped_continuation(rest, indent, 5, &mut stdout);
            continue;
        }

        // ── Regular paragraph text ────────────────────────────────────────
        let rendered = render_inline(trimmed);
        print_wrapped(stdout.lock(), &rendered, indent);
    }

    // Close any unclosed code block
    if in_code_block && !code_buf.is_empty() {
        flush_code_block(&mut stdout, &code_buf, &code_lang, indent);
    }

    println!();
    let _ = stdout.flush();
}

// ─── Code block renderer ──────────────────────────────────────────────────────

fn flush_code_block(
    stdout: &mut io::Stdout,
    lines: &[String],
    lang: &str,
    indent: &str,
) {
    let width = CAPS.width.saturating_sub(8) as usize;
    let lang_label = if lang.is_empty() { "code".to_string() } else { lang.to_uppercase() };

    // Top border with language label
    let border_inner = width.saturating_sub(lang_label.len() + 4);
    let _ = writeln!(stdout, "\n{}{}{}{}{}",
        indent,
        "╭─ ".with(Color::DarkGrey),
        lang_label.to_string().with(Theme::EMBER).bold(),
        " ".with(Color::DarkGrey),
        "─".repeat(border_inner).with(Color::DarkGrey)
    );

    // Code lines
    for line in lines {
        // Colorize keywords based on language
        let colored_line = colorize_code(line, lang);
        let display_line = if line.len() > width - 2 {
            format!("{}…", &line[..width - 3])
        } else {
            line.clone()
        };
        let _ = writeln!(stdout, "{}{}  {}",
            indent,
            "│".with(Color::DarkGrey),
            if CAPS.ansi_color { colored_line } else { display_line }
        );
    }

    // Bottom border
    let _ = writeln!(stdout, "{}{}",
        indent,
        "╰".to_string()
            + &"─".repeat(width)
            .with(Color::DarkGrey).to_string()
    );
    writeln!(stdout).ok();
}

fn colorize_code(line: &str, lang: &str) -> String {
    if !CAPS.ansi_color {
        return line.to_string();
    }

    // Keywords by language
    let rust_keywords = ["fn", "let", "mut", "pub", "use", "mod", "impl",
        "struct", "enum", "trait", "async", "await", "return", "if", "else",
        "match", "for", "while", "loop", "break", "continue", "true", "false",
        "Self", "self", "Result", "Ok", "Err", "Some", "None", "Vec", "String"];

    let python_keywords = ["def", "class", "import", "from", "return", "if",
        "else", "elif", "for", "while", "with", "as", "try", "except", "finally",
        "True", "False", "None", "async", "await", "lambda", "pass", "break",
        "continue", "yield", "in", "not", "and", "or", "is"];

    let js_keywords = ["function", "const", "let", "var", "return", "if",
        "else", "for", "while", "class", "extends", "async", "await",
        "import", "export", "default", "new", "this", "typeof", "true",
        "false", "null", "undefined", "try", "catch", "finally"];

    let lang_lower = lang.to_lowercase();
    let keywords: &[&str] = if lang_lower.contains("rust") {
        &rust_keywords
    } else if lang_lower.contains("python") || lang_lower.contains("py") {
        &python_keywords
    } else if lang_lower.contains("js") || lang_lower.contains("javascript")
           || lang_lower.contains("ts") || lang_lower.contains("typescript") {
        &js_keywords
    } else {
        return colorize_generic(line);
    };

    colorize_with_keywords(line, keywords)
}

fn colorize_generic(line: &str) -> String {
    // String literals in yellow, numbers in cyan, comments in grey
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") || trimmed.starts_with("#") {
        return line.to_string().with(Color::DarkGrey).italic().to_string();
    }
    line.to_string().with(Color::Rgb { r: 200, g: 230, b: 255 }).to_string()
}

fn colorize_with_keywords(line: &str, keywords: &[&str]) -> String {
    let trimmed = line.trim_start();

    // Comments
    if trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("--") {
        return line.to_string().with(Color::DarkGrey).italic().to_string();
    }

    // String literals — very basic detection
    if trimmed.contains('"') || trimmed.contains('\'') {
        return line.to_string().with(Color::Rgb { r: 255, g: 200, b: 100 }).to_string();
    }

    // Check if line starts with a keyword
    let first_word = trimmed.split_whitespace().next().unwrap_or("");
    if keywords.contains(&first_word) {
        return line.to_string().with(Color::Rgb { r: 130, g: 180, b: 255 }).bold().to_string();
    }

    // Type/function definitions (PascalCase or snake_case followed by paren)
    if trimmed.contains("(") || first_word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return line.to_string().with(Color::Rgb { r: 100, g: 220, b: 200 }).to_string();
    }

    line.to_string().with(Color::Rgb { r: 200, g: 230, b: 255 }).to_string()
}

// ─── Inline formatter ─────────────────────────────────────────────────────────

/// Render inline markdown: **bold**, *italic*, `code`, [text](url)
pub fn render_inline(text: &str) -> String {
    if !CAPS.ansi_color {
        return strip_inline_markdown(text);
    }

    let mut result = String::with_capacity(text.len() * 2);
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Bold: **text**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_closing(&chars, i + 2, "**") {
                let inner: String = chars[i + 2..end].iter().collect();
                result.push_str(&inner.bold().to_string());
                i = end + 2;
                continue;
            }
        }

        // Bold: __text__
        if i + 1 < chars.len() && chars[i] == '_' && chars[i + 1] == '_' {
            if let Some(end) = find_closing(&chars, i + 2, "__") {
                let inner: String = chars[i + 2..end].iter().collect();
                result.push_str(&inner.bold().to_string());
                i = end + 2;
                continue;
            }
        }

        // Italic: *text*
        if chars[i] == '*' && (i == 0 || chars[i - 1] != '*') {
            if let Some(end) = find_closing_single(&chars, i + 1, '*') {
                let inner: String = chars[i + 1..end].iter().collect();
                result.push_str(&inner.italic().to_string());
                i = end + 1;
                continue;
            }
        }

        // Italic: _text_
        if chars[i] == '_' && (i == 0 || chars[i - 1] != '_') {
            if let Some(end) = find_closing_single(&chars, i + 1, '_') {
                let inner: String = chars[i + 1..end].iter().collect();
                result.push_str(&inner.italic().to_string());
                i = end + 1;
                continue;
            }
        }

        // Inline code: `text`
        if chars[i] == '`' {
            if let Some(end) = find_closing_single(&chars, i + 1, '`') {
                let inner: String = chars[i + 1..end].iter().collect();
                result.push_str(
                    &inner.to_string()
                        .with(Color::Rgb { r: 0, g: 255, b: 136 })
                        .to_string()
                );
                i = end + 1;
                continue;
            }
        }

        // Link: [text](url)
        if chars[i] == '[' {
            if let Some(close_bracket) = find_closing_single(&chars, i + 1, ']') {
                if close_bracket + 1 < chars.len() && chars[close_bracket + 1] == '(' {
                    if let Some(close_paren) = find_closing_single(&chars, close_bracket + 2, ')') {
                        let link_text: String = chars[i + 1..close_bracket].iter().collect();
                        let url: String = chars[close_bracket + 2..close_paren].iter().collect();
                        result.push_str(&link_text.bold().to_string());
                        result.push(' ');
                        result.push_str(
                            &format!("({})", url)
                                .with(Color::DarkGrey)
                                .to_string()
                        );
                        i = close_paren + 1;
                        continue;
                    }
                }
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

// ─── Word wrapping ────────────────────────────────────────────────────────────

fn print_wrapped(mut stdout: io::StdoutLock<'_>, text: &str, indent: &str) {
    let term_width = CAPS.width.saturating_sub(6) as usize;
    let indent_len = indent.len();
    let avail = term_width.saturating_sub(indent_len);

    // Strip ANSI codes to measure display width
    let plain = strip_ansi(text);
    let words: Vec<&str> = plain.split(' ').collect();
    let ansi_words: Vec<&str> = text.split(' ').collect();

    let mut col = 0usize;
    let mut first = true;

    for (word, ansi_word) in words.iter().zip(ansi_words.iter()) {
        let w_len = word.chars().count();
        if col + w_len + (if first { 0 } else { 1 }) > avail && !first {
            writeln!(stdout).ok();
            write!(stdout, "{}", indent).ok();
            col = 0;
            write!(stdout, "{}", ansi_word).ok();
            col += w_len;
        } else {
            if first {
                write!(stdout, "{}{}", indent, ansi_word).ok();
            } else {
                write!(stdout, " {}", ansi_word).ok();
            }
            col += w_len + if first { 0 } else { 1 };
        }
        first = false;
    }
    writeln!(stdout).ok();
}

fn print_wrapped_continuation(
    text: &str,
    indent: &str,
    extra_indent: usize,
    stdout: &mut io::Stdout,
) {
    let rendered = render_inline(text);
    let plain = strip_ansi(&rendered);
    let term_width = CAPS.width.saturating_sub(6) as usize;
    let avail = term_width.saturating_sub(indent.len() + extra_indent);

    let words: Vec<&str> = plain.split(' ').collect();
    let ansi_words: Vec<&str> = rendered.split(' ').collect();
    let continuation_pad = format!("{}{}", indent, " ".repeat(extra_indent));

    let mut col = 0usize;
    let mut first = true;

    for (word, ansi_word) in words.iter().zip(ansi_words.iter()) {
        let w_len = word.chars().count();
        if col + w_len + 1 > avail && !first {
            writeln!(stdout).ok();
            write!(stdout, "{}", continuation_pad).ok();
            col = 0;
            write!(stdout, "{}", ansi_word).ok();
            col += w_len;
        } else {
            if !first { write!(stdout, " ").ok(); }
            write!(stdout, "{}", ansi_word).ok();
            col += w_len + if first { 0 } else { 1 };
        }
        first = false;
    }
    writeln!(stdout).ok();
}

// ─── Plain fallback ───────────────────────────────────────────────────────────

fn print_plain(text: &str, indent: &str) {
    for line in text.lines() {
        println!("{}{}", indent, strip_inline_markdown(line));
    }
}

fn strip_inline_markdown(text: &str) -> String {
    let mut result = text.to_string();
    // Remove **bold**, *italic*, `code`, [text](url)
    let patterns = [
        (r"\*\*(.+?)\*\*", "$1"),
        (r"\*(.+?)\*", "$1"),
        (r"__(.+?)__", "$1"),
        (r"_(.+?)_", "$1"),
        (r"`(.+?)`", "$1"),
    ];
    for (pat, rep) in &patterns {
        if let Ok(re) = regex::Regex::new(pat) {
            result = re.replace_all(&result, *rep).to_string();
        }
    }
    // Remove [text](url) → text
    if let Ok(re) = regex::Regex::new(r"\[(.+?)\]\(.+?\)") {
        result = re.replace_all(&result, "$1").to_string();
    }
    result
}

// ─── ANSI stripping ───────────────────────────────────────────────────────────

fn strip_ansi(text: &str) -> String {
    // Strip ESC[ ... m sequences
    let re = regex::Regex::new(r"\x1B\[[0-9;]*m").unwrap();
    re.replace_all(text, "").to_string()
}

// ─── Parser helpers ───────────────────────────────────────────────────────────

fn find_closing(chars: &[char], start: usize, marker: &str) -> Option<usize> {
    let m: Vec<char> = marker.chars().collect();
    let mlen = m.len();
    for i in start..chars.len().saturating_sub(mlen - 1) {
        if &chars[i..i + mlen] == m.as_slice() {
            return Some(i);
        }
    }
    None
}

fn find_closing_single(chars: &[char], start: usize, marker: char) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == marker {
            return Some(i);
        }
    }
    None
}

fn parse_ordered_item(line: &str) -> Option<(u32, &str)> {
    let dot_pos = line.find(". ")?;
    let num_str = &line[..dot_pos];
    if num_str.is_empty() || !num_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let num: u32 = num_str.parse().ok()?;
    Some((num, &line[dot_pos + 2..]))
}

// ─── Agentic step renderer ────────────────────────────────────────────────────

/// Render a labelled action step — used for agentic output headers
/// e.g.  render_step("SCANNING", "reading your codebase for vulnerabilities")
pub fn render_step(action: &str, detail: &str) {
    if CAPS.ansi_color {
        println!("  {} {}  {}",
            "▶".with(Theme::GREEN).bold(),
            action.to_string().with(Theme::GREEN).bold(),
            detail.to_string().with(Color::DarkGrey)
        );
    } else {
        println!("  [{}] {}", action, detail);
    }
}

/// Render a result status line
pub fn render_result(success: bool, label: &str, detail: &str) {
    if CAPS.ansi_color {
        let icon = if success {
            "✓".with(Theme::GREEN).bold().to_string()
        } else {
            "✗".with(Theme::FIRE).bold().to_string()
        };
        println!("  {} {}  {}", icon, label, detail.to_string().with(Color::DarkGrey));
    } else {
        println!("  [{}] {}: {}", if success { "OK" } else { "FAIL" }, label, detail);
    }
}

/// Print a section header — used between major output sections
pub fn render_section_header(title: &str) {
    let width = CAPS.width.saturating_sub(4) as usize;
    if CAPS.ansi_color {
        println!();
        println!("  {}", title.to_string().with(Theme::FIRE).bold());
        println!("  {}", "─".repeat(width.min(title.len() + 20)).with(Color::DarkGrey));
    } else {
        println!("\n  {}", title);
        println!("  {}", "-".repeat(title.len()));
    }
}
