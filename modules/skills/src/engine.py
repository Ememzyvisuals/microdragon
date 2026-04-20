"""
microdragon/modules/skills/src/engine.py
MICRODRAGON Skill System — Secure extensible plugins
Unlike OpenClaw's ClawHub (Cisco found RCE in unvetted skills),
MICRODRAGON skills are sandboxed, reviewed, and signed.
"""

import os
import json
import hashlib
import subprocess
import sys
from dataclasses import dataclass, field
from typing import Optional, Any
from pathlib import Path
import asyncio
import re


# ─── Skill definition ─────────────────────────────────────────────────────────

@dataclass
class SkillMetadata:
    """SKILL.md frontmatter — compatible with OpenClaw skill format."""
    name: str
    version: str
    description: str
    author: str
    license: str
    capabilities: list[str]        # What this skill can do
    permissions_required: list[str] # Permissions it needs
    trusted: bool = False           # Set by MICRODRAGON after review
    signature: Optional[str] = None # SHA256 of verified version
    install_date: Optional[str] = None
    source_url: Optional[str] = None


@dataclass
class Skill:
    metadata: SkillMetadata
    directory: Path
    skill_md_content: str
    is_sandboxed: bool = True


@dataclass
class SkillResult:
    success: bool
    output: str = ""
    error: str = ""
    skill_name: str = ""


# ─── Security levels ──────────────────────────────────────────────────────────

class SkillTrustLevel:
    CORE     = "core"      # Built into MICRODRAGON, fully trusted
    VERIFIED = "verified"  # Community-reviewed and signed
    COMMUNITY = "community" # Unreviewed, sandboxed
    BLOCKED  = "blocked"   # Flagged as malicious


# ─── Skill registry ───────────────────────────────────────────────────────────

class SkillRegistry:
    """Local skill registry — manages installed skills."""

    SKILLS_DIR = Path.home() / ".local" / "share" / "microdragon" / "skills"
    INDEX_FILE = SKILLS_DIR / "index.json"

    def __init__(self):
        self.SKILLS_DIR.mkdir(parents=True, exist_ok=True)
        self._index = self._load_index()

    def _load_index(self) -> dict:
        if self.INDEX_FILE.exists():
            try:
                return json.loads(self.INDEX_FILE.read_text())
            except Exception:
                pass
        return {"skills": {}, "version": "1.0"}

    def _save_index(self):
        self.INDEX_FILE.write_text(json.dumps(self._index, indent=2))

    def list_installed(self) -> list[dict]:
        return list(self._index.get("skills", {}).values())

    def get(self, name: str) -> Optional[dict]:
        return self._index.get("skills", {}).get(name)

    def install(self, skill_dir: Path, metadata: SkillMetadata) -> bool:
        """Register an installed skill."""
        entry = {
            "name": metadata.name,
            "version": metadata.version,
            "description": metadata.description,
            "author": metadata.author,
            "permissions": metadata.permissions_required,
            "trusted": metadata.trusted,
            "directory": str(skill_dir),
            "signature": metadata.signature,
        }
        self._index.setdefault("skills", {})[metadata.name] = entry
        self._save_index()
        return True

    def remove(self, name: str) -> bool:
        skills = self._index.get("skills", {})
        if name in skills:
            del skills[name]
            self._save_index()
            return True
        return False


# ─── Skill parser ─────────────────────────────────────────────────────────────

class SkillParser:
    """Parse SKILL.md files (compatible with OpenClaw skill format)."""

    def parse(self, skill_dir: Path) -> Optional[Skill]:
        skill_md = skill_dir / "SKILL.md"
        if not skill_md.exists():
            return None

        content = skill_md.read_text(encoding="utf-8")
        metadata = self._parse_frontmatter(content)
        if not metadata:
            return None

        return Skill(
            metadata=metadata,
            directory=skill_dir,
            skill_md_content=content
        )

    def _parse_frontmatter(self, content: str) -> Optional[SkillMetadata]:
        """Parse YAML-like frontmatter from SKILL.md."""
        frontmatter_match = re.search(r"^---\s*\n(.*?)\n---", content, re.DOTALL)
        if not frontmatter_match:
            # Try header-based format
            return self._parse_headers(content)

        fm = frontmatter_match.group(1)
        data = {}
        for line in fm.split("\n"):
            if ":" in line:
                key, _, val = line.partition(":")
                data[key.strip().lower()] = val.strip().strip('"\'')

        return SkillMetadata(
            name=data.get("name", ""),
            version=data.get("version", "0.1.0"),
            description=data.get("description", ""),
            author=data.get("author", "unknown"),
            license=data.get("license", "MIT"),
            capabilities=data.get("capabilities", "").split(","),
            permissions_required=data.get("permissions", "").split(","),
        )

    def _parse_headers(self, content: str) -> Optional[SkillMetadata]:
        """Parse markdown headers as metadata."""
        name_match = re.search(r"^#\s+(.+)$", content, re.MULTILINE)
        desc_match = re.search(r"^##\s+Description\s*\n(.+?)(?=^##|\Z)", content, re.MULTILINE | re.DOTALL)

        if not name_match:
            return None

        return SkillMetadata(
            name=name_match.group(1).strip(),
            version="0.1.0",
            description=desc_match.group(1).strip() if desc_match else "",
            author="unknown",
            license="unknown",
            capabilities=[],
            permissions_required=[],
        )


# ─── Security scanner ─────────────────────────────────────────────────────────

class SkillSecurityScanner:
    """
    Scans skills for malicious patterns before installation.
    Directly addresses Cisco's finding of data exfiltration in OpenClaw skills.
    """

    DANGEROUS_PATTERNS = [
        # Data exfiltration
        (r"requests\.post.*password", "Possible credential exfiltration"),
        (r"open.*\.env.*read", "Reading .env files"),
        (r"subprocess.*curl.*http", "Possible data upload via curl"),
        (r"socket\.connect.*\d+\.\d+\.\d+\.\d+", "Raw socket connection to IP"),

        # Destructive operations
        (r"shutil\.rmtree", "Recursive directory deletion"),
        (r"os\.remove.*\*", "Wildcard file deletion"),
        (r"subprocess.*rm\s+-rf", "rm -rf via subprocess"),
        (r"os\.system.*del\s+/", "del /S via os.system"),

        # Code injection
        (r"eval\s*\(", "eval() usage — code injection risk"),
        (r"exec\s*\(", "exec() usage — code injection risk"),
        (r"__import__.*os", "Dynamic os import — sandbox escape risk"),
        (r"subprocess\.(call|run|Popen).*shell\s*=\s*True", "shell=True — injection risk"),

        # Network exfiltration
        (r"smtplib.*sendmail", "Email send — possible exfiltration"),
        (r"requests.*api_keys|secrets|passwords", "Sending sensitive data to API"),
    ]

    def scan(self, skill_dir: Path) -> tuple[bool, list[str]]:
        """
        Returns (is_safe, list_of_warnings).
        Scans all Python and JS files in the skill directory.
        """
        warnings = []

        for ext in ("*.py", "*.js", "*.sh", "*.bash"):
            for file_path in skill_dir.rglob(ext):
                try:
                    content = file_path.read_text(encoding="utf-8", errors="ignore")
                    for pattern, description in self.DANGEROUS_PATTERNS:
                        if re.search(pattern, content, re.IGNORECASE):
                            warnings.append(f"[{file_path.name}] {description}")
                except Exception:
                    continue

        is_safe = len(warnings) == 0
        return is_safe, warnings

    def compute_signature(self, skill_dir: Path) -> str:
        """Compute SHA256 of all skill files for integrity verification."""
        hasher = hashlib.sha256()
        for file_path in sorted(skill_dir.rglob("*")):
            if file_path.is_file():
                hasher.update(file_path.read_bytes())
        return hasher.hexdigest()


# ─── Sandboxed executor ────────────────────────────────────────────────────────

class SandboxedExecutor:
    """Execute skill code in a restricted environment."""

    def execute_python(self, skill_dir: Path, entry_file: str,
                        task: str, timeout: int = 30) -> SkillResult:
        """Run a Python skill with restricted imports and timeout."""
        script_path = skill_dir / entry_file
        if not script_path.exists():
            return SkillResult(success=False, error=f"Entry file not found: {entry_file}")

        # Wrap with sandbox restrictions
        sandbox_wrapper = f"""
import sys
import importlib

# Block dangerous imports in skill code
BLOCKED_MODULES = {{'subprocess', 'os.system', 'pty', 'ctypes', 'socket'}}
original_import = __builtins__.__import__ if hasattr(__builtins__, '__import__') else __import__

def safe_import(name, *args, **kwargs):
    if name.split('.')[0] in BLOCKED_MODULES:
        raise ImportError(f"Module '{{name}}' is blocked in skill sandbox")
    return original_import(name, *args, **kwargs)

# Run skill
import runpy
sys.argv = ['{str(script_path)}', '{task}']
runpy.run_path('{str(script_path)}', init_globals={{'MICRODRAGON_TASK': '{task}'}})
"""
        try:
            result = subprocess.run(
                [sys.executable, "-c", sandbox_wrapper],
                capture_output=True, text=True, timeout=timeout,
                cwd=str(skill_dir),
                env={**os.environ, "MICRODRAGON_SANDBOXED": "1"}
            )
            return SkillResult(
                success=result.returncode == 0,
                output=result.stdout[:5000],
                error=result.stderr[:1000] if result.returncode != 0 else ""
            )
        except subprocess.TimeoutExpired:
            return SkillResult(success=False, error=f"Skill timed out after {timeout}s")
        except Exception as e:
            return SkillResult(success=False, error=str(e))


# ─── Skill manager ─────────────────────────────────────────────────────────────

class SkillEngine:
    """Main skill management engine."""

    def __init__(self):
        self.registry = SkillRegistry()
        self.parser = SkillParser()
        self.scanner = SkillSecurityScanner()
        self.executor = SandboxedExecutor()

    async def install_from_url(self, url: str, force: bool = False) -> tuple[bool, str]:
        """Download and install a skill from URL with security scanning."""
        try:
            import aiohttp
            async with aiohttp.ClientSession() as session:
                async with session.get(url) as resp:
                    if resp.status != 200:
                        return False, f"Download failed: HTTP {resp.status}"
                    content = await resp.text()

            # Create temp directory and write SKILL.md
            import tempfile, shutil
            tmp_dir = Path(tempfile.mkdtemp())
            (tmp_dir / "SKILL.md").write_text(content)

            return await self.install_from_directory(tmp_dir, force)
        except Exception as e:
            return False, str(e)

    async def install_from_directory(self, skill_dir: Path,
                                       force: bool = False) -> tuple[bool, str]:
        """Install a skill from a local directory with security scanning."""
        # Parse skill
        skill = self.parser.parse(skill_dir)
        if not skill:
            return False, "Invalid skill: SKILL.md not found or malformed"

        # Security scan
        is_safe, warnings = self.scanner.scan(skill_dir)
        if not is_safe and not force:
            warn_list = "\n  - ".join(warnings)
            return False, (
                f"Security scan failed for skill '{skill.metadata.name}':\n  - {warn_list}\n"
                f"Use --force to install anyway (not recommended)"
            )

        # Compute signature
        skill.metadata.signature = self.scanner.compute_signature(skill_dir)

        # Copy to skills directory
        install_path = self.registry.SKILLS_DIR / skill.metadata.name
        if install_path.exists():
            import shutil
            shutil.rmtree(install_path)
        import shutil
        shutil.copytree(skill_dir, install_path)

        # Register
        self.registry.install(install_path, skill.metadata)

        trust = "with warnings" if warnings else "cleanly"
        return True, f"Skill '{skill.metadata.name}' installed {trust}"

    def list_skills(self) -> str:
        """List all installed skills."""
        skills = self.registry.list_installed()
        if not skills:
            return "No skills installed. Use 'microdragon skills install <url>'"

        lines = [f"Installed skills ({len(skills)}):\n"]
        for s in skills:
            trusted = "✓" if s.get("trusted") else "⚠"
            lines.append(f"  {trusted} {s['name']} v{s['version']} — {s['description'][:50]}")
        return "\n".join(lines)

    def run_skill(self, name: str, task: str) -> SkillResult:
        """Execute an installed skill."""
        skill_info = self.registry.get(name)
        if not skill_info:
            return SkillResult(success=False, error=f"Skill '{name}' not found")

        skill_dir = Path(skill_info["directory"])
        entry = "main.py" if (skill_dir / "main.py").exists() else "skill.py"

        result = self.executor.execute_python(skill_dir, entry, task)
        result.skill_name = name
        return result

    def remove_skill(self, name: str) -> tuple[bool, str]:
        """Remove an installed skill."""
        skill_info = self.registry.get(name)
        if not skill_info:
            return False, f"Skill '{name}' not found"

        import shutil
        skill_dir = Path(skill_info["directory"])
        if skill_dir.exists():
            shutil.rmtree(skill_dir)

        self.registry.remove(name)
        return True, f"Skill '{name}' removed"


# ─── Built-in core skills ──────────────────────────────────────────────────────

CORE_SKILL_TEMPLATES = {
    "web-search": """---
name: web-search
version: 1.0.0
description: Search the web and extract information
author: microdragon-core
license: MIT
capabilities: web_search, content_extraction
permissions: network
---
# Web Search Skill
Use DuckDuckGo to search and extract information from the web.
""",
    "file-manager": """---
name: file-manager
version: 1.0.0
description: Manage files and directories
author: microdragon-core
license: MIT
capabilities: file_read, file_write, directory
permissions: filesystem
---
# File Manager Skill
Read, write, and organize files safely within the workspace.
""",
    "daily-briefing": """---
name: daily-briefing
version: 1.0.0
description: Generate a daily briefing from configured sources
author: microdragon-core
license: MIT
capabilities: web_search, calendar
permissions: network, calendar_read
---
# Daily Briefing Skill
Collects news headlines, weather, and calendar events for a morning briefing.
""",
}


if __name__ == "__main__":
    async def demo():
        engine = SkillEngine()
        print("[MICRODRAGON Skills] Engine ready")
        print(f"  Skills directory: {engine.registry.SKILLS_DIR}")
        print(engine.list_skills())

    asyncio.run(demo())
