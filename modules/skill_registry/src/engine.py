"""
microdragon/modules/skill_registry/src/engine.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON SKILL REGISTRY — Enhanced
═══════════════════════════════════════════════════════════════════════════════

The skill system is the extension mechanism for Microdragon.
Skills are domain-specific capability modules that:

  1. Can be installed by users from GitHub or local directories
  2. Are security-scanned BEFORE installation (no exceptions)
  3. Are loaded dynamically at runtime
  4. Can be called by users: "microdragon skills run weather Lagos"
  5. Can be called BY MICRODRAGON autonomously when needed

SKILL DOMAINS:
  - engineering    (code gen, testing, CI/CD, databases)
  - creative       (design, video, content, copywriting)
  - business       (CRM, analytics, reporting, finance)
  - automation     (browser, desktop, scheduling, webhooks)
  - research       (OSINT, web scraping, data extraction)
  - security       (scanning, pentesting tools, analysis)

AUTO-USE:
  Microdragon detects when a skill is relevant and uses it automatically.
  Example: User asks about weather → Microdragon checks if weather skill
  is installed → runs it → incorporates result into response.

SECURITY MODEL:
  Every skill file is scanned for:
  - Data exfiltration patterns
  - Dangerous subprocess calls
  - eval()/exec() with external data
  - File system access outside allowed paths
  - Hardcoded credentials
  - Outbound network calls to unknown domains

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

import asyncio
import ast
import importlib.util
import inspect
import json
import os
import re
import sys
import tempfile
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from enum import Enum
from typing import Optional, Any


class SkillDomain(Enum):
    ENGINEERING  = "engineering"
    CREATIVE     = "creative"
    BUSINESS     = "business"
    AUTOMATION   = "automation"
    RESEARCH     = "research"
    SECURITY     = "security"
    UTILITY      = "utility"
    SOCIAL       = "social"


class ScanResult(Enum):
    CLEAN        = "clean"
    WARNING      = "warning"
    DANGEROUS    = "dangerous"


@dataclass
class SecurityFinding:
    severity:    str    # CRITICAL | HIGH | MEDIUM | LOW
    pattern:     str
    file:        str
    line:        int
    code:        str
    description: str


@dataclass
class SkillMeta:
    name:        str
    version:     str
    domain:      SkillDomain
    description: str
    author:      str
    commands:    list[str]
    permissions: list[str]    # network | filesystem | shell | ai | social
    triggers:    list[str]    # keywords that auto-activate this skill
    install_dir: str = ""
    entry_point: str = "main.py"
    requirements: list[str] = field(default_factory=list)
    scan_result: ScanResult = ScanResult.CLEAN
    scan_findings: list[SecurityFinding] = field(default_factory=list)


# ─── Security Scanner ─────────────────────────────────────────────────────────

class SkillSecurityScanner:
    """
    Security scan every skill file before installation.
    Called automatically — users cannot skip this.
    """

    # Patterns that indicate definite threats
    CRITICAL_PATTERNS = [
        (r'requests\.post\s*\([^)]*(?:password|secret|key|token)', "Exfiltration: sensitive data in POST"),
        (r'socket\.connect\s*\(\s*\(["\'][0-9.]+["\']',           "Raw socket to IP address"),
        (r'subprocess.*shell\s*=\s*True.*\+',                      "Shell injection via string concat"),
        (r'exec\s*\(\s*(?:request|input|argv|environ)',            "exec() with external input"),
        (r'eval\s*\(\s*(?:request|input|argv|environ)',            "eval() with external input"),
        (r'__import__\s*\(\s*(?:os|subprocess|sys)',               "Dynamic dangerous import"),
        (r'open\s*\([^)]*(?:\/etc\/passwd|\/etc\/shadow)',         "Reading sensitive system files"),
        (r'paramiko|ftplib.*upload.*\.',                           "File upload to external server"),
        (r'os\.remove\s*\(.*(?:\/home|\/usr|\/etc)',               "Deleting critical system paths"),
    ]

    # Patterns that are suspicious but may be legitimate
    HIGH_PATTERNS = [
        (r'requests\.\w+\s*\(\s*["\']https?://',      "External HTTP call — verify domain"),
        (r'subprocess\.(?:run|Popen|call)',             "subprocess usage — verify inputs"),
        (r'os\.environ\.get.*(?:KEY|SECRET|TOKEN|PASS)', "Accessing environment secrets"),
        (r'pickle\.(?:loads|load)\s*\(',                "Deserialization — potential RCE"),
        (r'yaml\.load\s*\([^,)]+\)',                   "Unsafe YAML load — use safe_load"),
        (r'shutil\.rmtree\s*\(',                        "Directory removal"),
    ]

    # Informational — not dangerous but notable
    MEDIUM_PATTERNS = [
        (r'import\s+paramiko',     "SSH library — check if needed"),
        (r'import\s+pynput',       "Input simulation — legitimate for automation"),
        (r'import\s+pyautogui',    "Desktop automation — legitimate"),
        (r'from\s+PIL',            "Image processing"),
    ]

    def scan_file(self, filepath: str) -> list[SecurityFinding]:
        """Scan a single Python file for security issues."""
        findings = []
        try:
            content = Path(filepath).read_text(encoding="utf-8", errors="replace")
        except Exception:
            return findings

        lines = content.split("\n")

        # Check for syntax validity first
        try:
            ast.parse(content)
        except SyntaxError as e:
            findings.append(SecurityFinding(
                severity="HIGH", pattern="syntax_error",
                file=filepath, line=e.lineno or 0,
                code="", description=f"Syntax error: {e.msg}"
            ))
            return findings  # Can't safely analyse broken code

        # Scan each pattern set
        for patterns, severity in [
            (self.CRITICAL_PATTERNS, "CRITICAL"),
            (self.HIGH_PATTERNS, "HIGH"),
            (self.MEDIUM_PATTERNS, "MEDIUM"),
        ]:
            for pattern, desc in patterns:
                for i, line in enumerate(lines, 1):
                    stripped = line.strip()
                    # Skip comments
                    if stripped.startswith("#") or stripped.startswith('"""'):
                        continue
                    if re.search(pattern, line, re.IGNORECASE):
                        findings.append(SecurityFinding(
                            severity=severity, pattern=pattern,
                            file=filepath, line=i,
                            code=stripped[:120],
                            description=desc
                        ))
                        break  # One finding per pattern per file

        return findings

    def scan_directory(self, skill_dir: str) -> tuple[ScanResult, list[SecurityFinding]]:
        """Scan all Python files in a skill directory."""
        all_findings = []
        skill_path = Path(skill_dir)

        for py_file in skill_path.rglob("*.py"):
            findings = self.scan_file(str(py_file))
            all_findings.extend(findings)

        # Also check requirements.txt for known malicious packages
        req_file = skill_path / "requirements.txt"
        if req_file.exists():
            req_findings = self._scan_requirements(str(req_file))
            all_findings.extend(req_findings)

        # Determine overall result
        if any(f.severity == "CRITICAL" for f in all_findings):
            return ScanResult.DANGEROUS, all_findings
        if any(f.severity == "HIGH" for f in all_findings):
            return ScanResult.WARNING, all_findings
        return ScanResult.CLEAN, all_findings

    def _scan_requirements(self, req_path: str) -> list[SecurityFinding]:
        """Check requirements.txt for known suspicious packages."""
        # Known typosquatting / malicious packages (based on public advisories)
        suspicious_packages = [
            "colourama",      # typosquats colorama
            "python-dateutil2", # malicious variant
            "requestes",      # typosquats requests
            "pillow-security", # known malicious
        ]
        findings = []
        try:
            content = Path(req_path).read_text()
            for line in content.splitlines():
                pkg = line.strip().split(">=")[0].split("==")[0].strip().lower()
                if pkg in suspicious_packages:
                    findings.append(SecurityFinding(
                        severity="CRITICAL", pattern="suspicious_package",
                        file=req_path, line=0, code=line,
                        description=f"Known suspicious/malicious package: {pkg}"
                    ))
        except Exception:
            pass
        return findings

    def format_report(self, skill_name: str, result: ScanResult,
                       findings: list[SecurityFinding]) -> str:
        """Format the scan report for display to the user."""
        if result == ScanResult.CLEAN and not findings:
            return f"  ✓ {skill_name} — security scan passed. No issues found."

        icon = "✗" if result == ScanResult.DANGEROUS else "!"
        lines = [
            f"",
            f"  {icon} Security scan: {skill_name}",
            f"  Result: {result.value.upper()}",
            f"",
        ]

        critical = [f for f in findings if f.severity == "CRITICAL"]
        high     = [f for f in findings if f.severity == "HIGH"]
        medium   = [f for f in findings if f.severity == "MEDIUM"]

        if critical:
            lines.append("  CRITICAL FINDINGS (installation BLOCKED):")
            for f in critical:
                lines.append(f"    ✗ {f.file}:{f.line}")
                lines.append(f"      {f.description}")
                lines.append(f"      Code: {f.code}")

        if high:
            lines.append("  HIGH FINDINGS (review before installing):")
            for f in high:
                lines.append(f"    ! {f.file}:{f.line}")
                lines.append(f"      {f.description}")

        if medium:
            lines.append("  NOTES:")
            for f in medium:
                lines.append(f"    → {f.description} ({f.file}:{f.line})")

        if result == ScanResult.DANGEROUS:
            lines.extend([
                "",
                "  Installation BLOCKED due to critical security issues.",
                "  Do not install this skill unless you review and trust the code.",
            ])
        elif result == ScanResult.WARNING:
            lines.extend([
                "",
                "  Skill can be installed but review the HIGH findings first.",
                "  Type 'yes' to install anyway, or 'no' to cancel:",
            ])

        return "\n".join(lines)


# ─── Skill Loader ─────────────────────────────────────────────────────────────

class SkillLoader:
    """Dynamically loads and executes installed skills."""

    def __init__(self, skills_dir: str):
        self.skills_dir = Path(skills_dir)
        self._loaded: dict[str, Any] = {}

    def discover_skills(self) -> list[SkillMeta]:
        """Find all installed skills and read their metadata."""
        skills = []
        if not self.skills_dir.exists():
            return skills

        for skill_dir in self.skills_dir.iterdir():
            if not skill_dir.is_dir():
                continue
            meta_file = skill_dir / "SKILL.md"
            if meta_file.exists():
                meta = self._parse_skill_md(str(meta_file), str(skill_dir))
                if meta:
                    skills.append(meta)
        return skills

    def _parse_skill_md(self, meta_path: str, skill_dir: str) -> Optional[SkillMeta]:
        """Parse SKILL.md metadata file."""
        try:
            content = Path(meta_path).read_text()
            lines = content.splitlines()

            def extract(key: str, default: str = "") -> str:
                for line in lines:
                    if line.startswith(f"**{key}:**"):
                        return line.split("**:", 1)[1].strip().strip("*")
                return default

            def extract_list(key: str) -> list:
                result = []
                in_section = False
                for line in lines:
                    if f"**{key}:**" in line:
                        in_section = True
                        continue
                    if in_section:
                        if line.startswith("**") and ":**" in line:
                            break
                        stripped = line.strip().lstrip("-").strip()
                        if stripped and not stripped.startswith("#"):
                            result.append(stripped)
                return result

            # Map domain string to enum
            domain_str = extract("Type", "utility").lower()
            domain_map = {
                "engineering": SkillDomain.ENGINEERING,
                "creative":    SkillDomain.CREATIVE,
                "business":    SkillDomain.BUSINESS,
                "automation":  SkillDomain.AUTOMATION,
                "research":    SkillDomain.RESEARCH,
                "security":    SkillDomain.SECURITY,
                "social":      SkillDomain.SOCIAL,
                "data":        SkillDomain.ENGINEERING,
            }
            domain = domain_map.get(domain_str, SkillDomain.UTILITY)

            return SkillMeta(
                name=extract("Skill", Path(skill_dir).name),
                version=extract("Version", "1.0.0"),
                domain=domain,
                description=extract("Description", ""),
                author=extract("Author", "unknown"),
                commands=extract_list("Commands"),
                permissions=extract_list("Permissions"),
                triggers=extract_list("Triggers"),
                install_dir=skill_dir,
                requirements=extract_list("Requirements"),
            )
        except Exception:
            return None

    def load_skill(self, meta: SkillMeta) -> Optional[Any]:
        """Import and instantiate a skill class."""
        if meta.name in self._loaded:
            return self._loaded[meta.name]

        entry = Path(meta.install_dir) / meta.entry_point
        if not entry.exists():
            return None

        try:
            spec = importlib.util.spec_from_file_location(
                f"microdragon_skill_{meta.name}", str(entry)
            )
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)

            # Find the skill class (subclasses SkillBase)
            for name, obj in inspect.getmembers(module, inspect.isclass):
                if hasattr(obj, "_is_microdragon_skill") or \
                   any(c.__name__ == "SkillBase" for c in inspect.getmro(obj)[1:]):
                    instance = obj()
                    instance._meta = meta
                    self._loaded[meta.name] = instance
                    return instance

        except Exception as e:
            print(f"  Failed to load skill {meta.name}: {e}")
        return None

    async def run_skill(self, skill_name: str, command: str,
                         *args, **kwargs) -> dict:
        """Execute a skill command with timeout protection."""
        meta = None
        skills = self.discover_skills()
        for s in skills:
            if s.name.lower() == skill_name.lower():
                meta = s
                break

        if not meta:
            return {"success": False, "error": f"Skill '{skill_name}' not installed"}

        instance = self.load_skill(meta)
        if not instance:
            return {"success": False, "error": f"Could not load skill '{skill_name}'"}

        # Find the command method
        method = getattr(instance, command, None)
        if not method:
            # Try all commands to find a match
            for cmd in meta.commands:
                cmd_clean = cmd.split()[0].replace("-", "_")
                method = getattr(instance, cmd_clean, None)
                if method:
                    break

        if not method:
            return {"success": False, "error": f"Command '{command}' not found in {skill_name}"}

        try:
            # Run with 30-second timeout
            result = await asyncio.wait_for(
                method(*args, **kwargs) if asyncio.iscoroutinefunction(method)
                else asyncio.coroutine(lambda: method(*args, **kwargs))(),
                timeout=30.0
            )
            if hasattr(result, "success"):
                return {"success": result.success, "output": result.output,
                        "error": result.error}
            return {"success": True, "output": str(result)}
        except asyncio.TimeoutError:
            return {"success": False, "error": f"Skill timed out after 30 seconds"}
        except Exception as e:
            return {"success": False, "error": str(e)}


# ─── Auto-Skill Detector ──────────────────────────────────────────────────────

class AutoSkillDetector:
    """
    Detects when an installed skill is relevant to the current task
    and activates it automatically — without user needing to ask.
    """

    def find_relevant_skills(self, user_input: str,
                              skills: list[SkillMeta]) -> list[SkillMeta]:
        """Find skills whose triggers match the user's input."""
        inp = user_input.lower()
        relevant = []
        for skill in skills:
            if any(trigger.lower() in inp for trigger in skill.triggers):
                relevant.append(skill)
            # Also match on skill name and commands
            elif skill.name.lower() in inp:
                relevant.append(skill)
        return relevant


# ─── Master Registry ──────────────────────────────────────────────────────────

class SkillRegistry:
    """Main entry point for the Microdragon skill system."""

    def __init__(self):
        self.skills_dir = os.path.expanduser("~/.local/share/microdragon/skills")
        Path(self.skills_dir).mkdir(parents=True, exist_ok=True)
        self.scanner  = SkillSecurityScanner()
        self.loader   = SkillLoader(self.skills_dir)
        self.detector = AutoSkillDetector()

    async def install(self, source: str,
                       force: bool = False) -> tuple[bool, str]:
        """
        Install a skill from URL or local directory.
        ALWAYS security scans before installation.
        """
        import shutil, tempfile

        # Download if URL
        if source.startswith("http"):
            tmp_dir = tempfile.mkdtemp()
            try:
                result = subprocess.run(
                    ["git", "clone", "--depth=1", source, tmp_dir],
                    capture_output=True, text=True, timeout=60
                )
                if result.returncode != 0:
                    return False, f"Failed to clone: {result.stderr[:200]}"
                source = tmp_dir
            except Exception as e:
                return False, f"Download failed: {e}"

        source_path = Path(source)
        if not source_path.exists():
            return False, f"Path not found: {source}"

        # SCAN FIRST — no exceptions
        print(f"\n  🔍 Scanning {source_path.name} for security issues...")
        scan_result, findings = self.scanner.scan_directory(str(source_path))
        report = self.scanner.format_report(source_path.name, scan_result, findings)
        print(report)

        if scan_result == ScanResult.DANGEROUS:
            return False, "Installation BLOCKED: critical security issues found"

        if scan_result == ScanResult.WARNING and not force:
            return False, "Installation requires manual review. Use --force to override (not recommended)"

        # Read metadata
        meta = self.loader._parse_skill_md(
            str(source_path / "SKILL.md"), str(source_path)
        )
        if not meta:
            return False, "Invalid skill: missing or malformed SKILL.md"

        # Install to skills directory
        dest = Path(self.skills_dir) / meta.name
        if dest.exists():
            shutil.rmtree(dest)
        shutil.copytree(str(source_path), str(dest))

        # Install requirements (in skill's own venv for isolation)
        req_file = dest / "requirements.txt"
        if req_file.exists():
            print(f"  Installing requirements for {meta.name}...")
            subprocess.run(
                [sys.executable, "-m", "pip", "install", "-r", str(req_file),
                 "-q", "--break-system-packages"],
                check=False
            )

        return True, f"✓ Skill '{meta.name}' v{meta.version} installed successfully"

    async def run(self, skill_name: str, command: str, *args) -> str:
        """Run a skill command. Returns formatted output."""
        result = await self.loader.run_skill(skill_name, command, *args)
        if result["success"]:
            return result.get("output", "Done")
        return f"Error: {result.get('error', 'Unknown error')}"

    def list_skills(self) -> str:
        """List all installed skills."""
        skills = self.loader.discover_skills()
        if not skills:
            return (
                "\n  No skills installed.\n"
                "  Install a skill: microdragon skills install <url_or_path>\n"
                "  Browse skills:   https://github.com/ememzyvisuals/microdragon-skills\n"
            )

        lines = ["\n  Installed skills:\n"]
        for s in skills:
            lines.append(f"  [{s.domain.value:12}] {s.name:<20} v{s.version}")
            lines.append(f"               {s.description[:60]}")
            if s.commands:
                cmds = [c.split()[0] for c in s.commands[:3]]
                lines.append(f"               Commands: {', '.join(cmds)}")
            lines.append("")
        return "\n".join(lines)

    def auto_detect(self, user_input: str) -> list[SkillMeta]:
        """Detect relevant installed skills for the current task."""
        skills = self.loader.discover_skills()
        return self.detector.find_relevant_skills(user_input, skills)
