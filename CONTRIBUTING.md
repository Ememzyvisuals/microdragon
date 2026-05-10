# Contributing to Microdragon

Microdragon is MIT licensed and designed to be forked, extended, and contributed to.
This guide covers everything you need: architecture, module patterns, skill development, and PR process.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Adding a New Module](#adding-a-new-module)
5. [Adding a New Agent](#adding-a-new-agent)
6. [Writing a Skill](#writing-a-skill)
7. [Rust Core Development](#rust-core-development)
8. [Testing Requirements](#testing-requirements)
9. [PR Process](#pr-process)
10. [Code Standards](#code-standards)

---

## Architecture Overview

```
User input
  ↓
Rust Core (microdragon binary)
  ├── CLI (simple_mode.rs / mod.rs)
  ├── Brain (model_router.rs) ─── AI provider API
  ├── Engine (agents.rs) ──────── route_task() classifies input
  ├── Pipeline (pipeline/mod.rs)  9-phase execution
  └── Memory (sqlite.rs + vector.rs)
         ↓
Python Modules (via IPC bridge)
  ├── modules/coding/        ← code gen, debug, review
  ├── modules/security_expert/  ← OWASP, SAST, threat model
  ├── modules/gaming/        ← computer vision + strategies
  ├── modules/training/      ← fine-tuning + self-debug
  ├── modules/image_video/   ← multi-provider media gen
  ├── modules/presentation/  ← SVG slide generation
  ├── modules/web_design/    ← expert web design + generation
  ├── modules/advertising/   ← full ad pipeline
  ├── modules/skill_registry/  ← dynamic skill loading
  └── ... (25+ modules)
```

---

## Development Setup

```bash
# 1. Clone
git clone https://github.com/ememzyvisuals/microdragon
cd microdragon

# 2. Rust toolchain
rustup update stable
cd core && cargo build    # debug build (fast, for dev)

# 3. Python deps
cd .. && pip install -r requirements.txt

# 4. Run benchmark to verify nothing is broken
python3 eval/benchmarks/microdragon_eval.py
# Must pass: 60/60 tests

# 5. Start dev mode
cargo run --manifest-path core/Cargo.toml
```

### Recommended tools

```bash
# Rust
cargo install cargo-watch    # auto-rebuild on save
cargo watch -x run           # start with auto-reload

# Python
pip install ruff mypy        # linter and type checker
ruff check modules/          # lint all modules
mypy modules/                # type check

# Testing
cargo test --manifest-path core/Cargo.toml   # Rust tests
python3 eval/benchmarks/microdragon_eval.py  # full benchmark
```

---

## Project Structure

```
microdragon/
├── core/                    Rust engine
│   └── src/
│       ├── main.rs          Entry point
│       ├── cli/             User interface
│       │   ├── first_launch.rs    Onboarding (OpenClaw-style)
│       │   ├── simple_mode.rs     Conversational default
│       │   ├── theme.rs           Colors + taglines
│       │   └── mod.rs             Pro CLI commands
│       ├── engine/          Task orchestration
│       │   ├── agents.rs          7 agents + routing
│       │   ├── autonomous.rs      Scheduler + self-improve
│       │   └── pipeline/          9-phase pipeline
│       ├── brain/           AI reasoning
│       │   ├── model_router.rs    5 providers unified
│       │   └── cost_optimizer.rs  Smart routing + caching
│       ├── memory/          Persistence
│       │   ├── sqlite.rs          7 tables, WAL mode
│       │   └── vector.rs          Semantic search
│       └── security/        Protection
│           ├── prompt_guard.rs    Injection detection
│           ├── sandbox.rs         Command blocklist
│           └── encryption.rs      AES-256-GCM

├── modules/                 Python capabilities
│   ├── [name]/src/engine.py Pattern: one engine.py per module
│   └── skill_registry/      Plugin system

├── eval/                    Benchmark suite
│   ├── benchmarks/microdragon_eval.py
│   └── reports/

├── npm/                     npm package
│   ├── bin/microdragon.js   CLI entry point
│   └── scripts/postinstall.js

└── .github/workflows/       CI/CD
    ├── release.yml          Build 5 platforms + npm publish
    └── ci.yml               PR checks
```

---

## Adding a New Module

Every Python module follows the same pattern. Here's how to add one:

### 1. Create the module directory

```bash
mkdir -p modules/my_module/src
```

### 2. Write the engine file

```python
# modules/my_module/src/engine.py
"""
microdragon/modules/my_module/src/engine.py
One-line description of what this module does.
© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

import asyncio
from dataclasses import dataclass
from typing import Optional


@dataclass
class MyModuleResult:
    success: bool
    output: str = ""
    error: Optional[str] = None


class MyModuleEngine:
    """Description of what this engine does."""

    def __init__(self):
        # Detect available backends, load config
        pass

    async def handle(self, task: str, **kwargs) -> MyModuleResult:
        """Main entry point — called by the Rust dispatcher."""
        try:
            result = await self._do_work(task, **kwargs)
            return MyModuleResult(success=True, output=result)
        except Exception as e:
            return MyModuleResult(success=False, error=str(e))

    async def _do_work(self, task: str, **kwargs) -> str:
        # Implementation here
        return f"Done: {task}"


# Always include a demo/test in __main__
if __name__ == "__main__":
    async def demo():
        engine = MyModuleEngine()
        result = await engine.handle("test task")
        print(result)
    asyncio.run(demo())
```

### 3. Register the module in the Rust dispatcher

Open `core/src/engine/dispatcher.rs` and add your module:

```rust
// In the dispatch() match arm:
"my_module" => {
    // Spawn Python subprocess
    self.run_python_module("modules/my_module/src/engine.py", &task).await
}
```

### 4. Add routing in agents.rs

Add keyword routing so Microdragon auto-routes relevant tasks:

```rust
// In route_task():
if lower.contains("my_keyword") || lower.contains("another_keyword") {
    return AgentRole::Master; // or a specific agent
}
```

### 5. Add to requirements.txt (if new deps needed)

```bash
echo "my-package>=1.0.0  # for my_module" >> requirements.txt
```

### 6. Add a test in the benchmark

```python
# In eval/benchmarks/microdragon_eval.py, add a test function:
def test_my_module(ev):
    content = ev.file_content("modules/my_module/src/engine.py")
    required = ["MyModuleEngine", "handle", "MyModuleResult"]
    ok, missing = ev.has_all_patterns(content, required)
    if ok:
        return "PASS", 1.0, "My module has all required components", None, None
    return "FAIL", 0.0, f"Missing: {missing}", "tool", "Complete module"

results.append(ev.run("MOD-XXX", "MyModule", "Module structure check", test_my_module))
```

---

## Adding a New Agent

Agents are defined in `core/src/engine/agents.rs`.

### 1. Add the role to the enum

The `AgentRole` enum is in `autonomous.rs`:

```rust
// In autonomous.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentRole {
    Master,
    Coding,
    Research,
    Business,
    Automation,
    Writing,
    Security,
    MyNewAgent,    // ← add here
}

impl fmt::Display for AgentRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // ...existing
            AgentRole::MyNewAgent => write!(f, "my_new_agent"),
        }
    }
}
```

### 2. Add agent config in agents.rs

```rust
// In AgentConfig::for_role():
AgentRole::MyNewAgent => Self {
    role,
    name: "MICRODRAGON MyNewAgent".to_string(),
    system_prompt: MY_NEW_AGENT_PROMPT.to_string(),
    max_tokens: 4096,
    temperature: 0.5,
    allowed_tools: vec!["specific_tool".to_string()],
    enabled: true,
},
```

### 3. Add the system prompt

```rust
const MY_NEW_AGENT_PROMPT: &str = "You are MICRODRAGON MyNewAgent — describe the role. \
Focus on specific capability. \
Always do X, never do Y.";
```

### 4. Add routing keywords

```rust
// In route_task():
if lower.contains("keyword1") || lower.contains("keyword2") {
    return AgentRole::MyNewAgent;
}
```

---

## Writing a Skill

Skills are the plugin system. See `modules/skills/SKILLS_GUIDE.md` for the full guide.

### Minimal skill structure

```
my_skill/
├── SKILL.md
└── main.py
```

**SKILL.md:**
```markdown
# Skill: MySkill
**Author:** your_github
**Version:** 1.0.0
**Type:** utility
**Description:** What it does
**Commands:**
  - my-command <arg> : description
**Permissions:** network
**Triggers:** keyword1, keyword2
```

**main.py:**
```python
from dataclasses import dataclass
from typing import Optional

@dataclass
class SkillResult:
    success: bool
    output: str = ""
    error: Optional[str] = None

class MySkill:
    _is_microdragon_skill = True
    name = "my-skill"

    async def my_command(self, arg: str = "", **kwargs) -> SkillResult:
        return SkillResult(success=True, output=f"Done: {arg}")
```

**Test your skill before submitting:**
```bash
microdragon skills scan ./my_skill/    # must show CLEAN
microdragon skills install ./my_skill/
microdragon skills run my-skill my-command "test"
```

---

## Rust Core Development

### Key files and what they do

| File | Responsibility |
|---|---|
| `main.rs` | Entry point, mode selection, daemon startup |
| `engine/mod.rs` | `MicrodragonEngine` — central orchestrator |
| `engine/agents.rs` | 7 agents + intent routing |
| `engine/pipeline/mod.rs` | 9-phase execution pipeline |
| `engine/autonomous.rs` | TokenGuard, self-improvement, scheduler |
| `brain/model_router.rs` | Unified AI API for all providers |
| `brain/cost_optimizer.rs` | Smart routing, cache, budget |
| `cli/simple_mode.rs` | Default conversational interface |
| `cli/first_launch.rs` | OpenClaw-style onboarding |
| `cli/theme.rs` | Dragon green/fire colors + taglines |
| `memory/sqlite.rs` | 7-table SQLite schema, WAL mode |
| `memory/vector.rs` | Cosine similarity search |
| `security/prompt_guard.rs` | Injection attack detection |

### Rust style rules

```rust
// Use anyhow for errors
use anyhow::{Result, Context};

// Prefer ? over unwrap()
let config = NexaConfig::load().context("Failed to load config")?;

// Use Arc for shared state
let engine: Arc<MicrodragonEngine> = Arc::new(engine);

// RwLock for read-heavy shared state
pub memory: Arc<RwLock<MemoryStore>>,

// tokio::spawn for background tasks
tokio::spawn(async move { watch_daemon(engine).await });

// tracing for logging (not println!)
info!("Engine ready");
error!("Pipeline failed: {}", e);
```

---

## Testing Requirements

Before every PR, run:

```bash
# 1. Rust tests
cd core && cargo test

# 2. Full benchmark — must stay at 60/60
python3 eval/benchmarks/microdragon_eval.py

# 3. Python syntax check
python3 -c "
import ast, os
for root, dirs, files in os.walk('modules'):
    dirs[:] = [d for d in dirs if d != '__pycache__']
    for fn in files:
        if fn.endswith('.py'):
            path = os.path.join(root, fn)
            ast.parse(open(path).read())
print('All Python files: OK')
"

# 4. Name consistency check (must return 0)
grep -r '\bnexa\b\|\bdinoclaw\b' core/ modules/ npm/ --include='*.rs' \
  --include='*.py' --include='*.js' | wc -l
# Must output: 0
```

**PR requirements:**
- `cargo test` passes
- `microdragon_eval.py` shows 60/60 (or higher if you added tests)
- Zero Python syntax errors
- Zero stale name references
- New feature has at least one benchmark test

---

## PR Process

1. **Fork** the repo
2. **Branch**: `git checkout -b feat/my-feature`
3. **Build**: make your changes
4. **Test**: run all tests above
5. **Commit**: `git commit -m "feat: clear description of what changed"`
6. **Push**: `git push origin feat/my-feature`
7. **PR**: open a pull request with description of what and why

### Commit message format

```
feat: add weather skill with Groq + OpenWeather providers
fix: correct SQLite timestamp handling on Windows
docs: add CONTRIBUTING.md sections for skill development
test: add benchmark tests for image_video module
refactor: simplify cost_optimizer routing logic
```

---

## Code Standards

### Python

- Type hints on all function signatures
- Docstring on every class and public method
- `async/await` for all I/O
- Graceful degradation when optional deps missing
- Never `print()` in production paths — use structured return values
- Every module has `if __name__ == "__main__"` demo

### Rust

- `anyhow::Result<T>` for all fallible functions
- `?` operator instead of `unwrap()`
- `tracing::info/error/warn` for logging
- `Arc<T>` for shared ownership
- Comments on non-obvious logic

### Both

- No hardcoded credentials or API keys
- No `TODO` comments in merged code
- Error messages must be actionable (tell user what to do)
- Security-sensitive operations go through `security/` modules

---

## Questions

Open an issue or reach out:

- **X / Twitter:** [@ememzyvisuals](https://x.com/ememzyvisuals) — DM open
- **GitHub Issues:** https://github.com/ememzyvisuals/microdragon/issues
- **Discussions:** https://github.com/ememzyvisuals/microdragon/discussions

*Every contribution matters. Microdragon is built by the community.*

---

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
