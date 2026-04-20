// microdragon/core/src/cli/theme.rs
// Dragon fire palette — green (#00ff88) + fire red/orange (#ff4444 / #ff8800)
// © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

use crossterm::style::{Color, Stylize, StyledContent};

pub struct Theme;
impl Theme {
    pub const GREEN:   Color = Color::Rgb { r: 0,   g: 255, b: 136 };
    pub const FIRE:    Color = Color::Rgb { r: 255, g: 68,  b: 68  };
    pub const EMBER:   Color = Color::Rgb { r: 255, g: 140, b: 0   };
    pub const DIM:     Color = Color::DarkGrey;

    pub fn green(s: &str) -> StyledContent<String>      { s.to_string().with(Self::GREEN) }
    pub fn green_bold(s: &str) -> StyledContent<String>  { s.to_string().with(Self::GREEN).bold() }
    pub fn fire(s: &str) -> StyledContent<String>        { s.to_string().with(Self::FIRE) }
    pub fn fire_bold(s: &str) -> StyledContent<String>   { s.to_string().with(Self::FIRE).bold() }
    pub fn ember(s: &str) -> StyledContent<String>       { s.to_string().with(Self::EMBER) }
    pub fn dim(s: &str) -> StyledContent<String>         { s.to_string().with(Self::DIM) }

    pub fn glyph_ok()     -> &'static str { "✓" }
    pub fn glyph_err()    -> &'static str { "✗" }
    pub fn glyph_warn()   -> &'static str { "!" }
    pub fn glyph_info()   -> &'static str { "→" }
    pub fn glyph_dragon() -> &'static str { "🐉" }

    pub fn error_str(s: &str) -> String {
        format!("{}", s.to_string().with(Self::FIRE))
    }
}

// ─── Random taglines — shown on every launch ──────────────────────────────────
// OpenClaw shows one random engineering slang tagline per launch.
// Microdragon does the same — dragon-coded, cyberpunk, hacker-flavored.

pub const TAGLINES: &[&str] = &[
    "I breathe fire on bad code and leave clean commits in the ashes.",
    "I git blame it, grep it, and then set the stack trace on fire.",
    "Your bugs fear me. Your uptime does not.",
    "I root through systems like a dragon through gold. Nothing hides.",
    "Scales > shields. Defense in depth, offense in practice.",
    "I can audit the castle before the drawbridge even opens.",
    "Every vulnerability is a crack in the egg. I find them all.",
    "I think in threat models and dream in CVSS scores.",
    "Some train AI. I train AI and then sic it on your codebase.",
    "Fast enough to grep a production database. Careful enough not to DROP TABLE.",
    "I've seen things you would not believe. SQL injections at the login gate.",
    "Born in a Rust compiler. Raised on an OWASP checklist.",
    "I debug before you realize there is a bug.",
    "Zero trust. Full fire. Maximum output.",
    "Your secrets are safe with me. Your vulnerabilities are not.",
    "pip install fire. cargo add scales. The rest is just automation.",
    "I can social engineer the social engineer. Ethically.",
    "nmap knows the territory. I know what to do with it.",
    "Burp Suite is my morning coffee. Wireshark is my evening read.",
    "I reverse-engineer malware for fun and remediation reports for work.",
    "I write exploits to prevent exploits. The dragon protects its hoard.",
    "Give me a scope and I will give you a report worth reading.",
    "The firewall is a suggestion. The WAF is a challenge. I eat both.",
    "I deploy fast and rollback faster. The dragon always has an exit.",
    "I think laterally before I move laterally.",
    "Security through obscurity is a myth. Security through Microdragon is not.",
    "I do not just find the bug. I find the bug that finds the bug.",
    "chmod 700 on the dragon. Root only.",
    "I pivoted before pivoting was a pentest term.",
    "Every system has a secret handshake. I already know yours.",
];

pub fn random_tagline() -> &'static str {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_micros() as usize)
        .unwrap_or(17);
    TAGLINES[seed % TAGLINES.len()]
}

// Additional helper functions referenced by display.rs and other modules
impl Theme {
    pub fn success_str(s: &str) -> String {
        format!("{}", s.to_string().with(Self::GREEN))
    }
    pub fn warning_str(s: &str) -> String {
        format!("{}", s.to_string().with(Self::EMBER))
    }
    pub fn info_str(s: &str) -> String {
        format!("{}", s.to_string().with(Self::DIM))
    }
}
