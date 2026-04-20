"""
microdragon/modules/coding/src/engine.py
MICRODRAGON Coding Module — Code generation, debugging, testing, git
"""

import subprocess
import os
import sys
import json
import re
import tempfile
import shutil
from pathlib import Path
from typing import Optional
from dataclasses import dataclass, field


@dataclass
class CodeResult:
    success: bool
    code: str = ""
    language: str = ""
    tests: str = ""
    explanation: str = ""
    file_path: Optional[str] = None
    errors: list = field(default_factory=list)


class CodingEngine:
    """Core coding engine for MICRODRAGON — handles generation, debug, review, git."""

    SUPPORTED_LANGUAGES = {
        "python": {"ext": ".py", "run": "python3", "test": "pytest"},
        "rust": {"ext": ".rs", "run": "cargo run", "test": "cargo test"},
        "javascript": {"ext": ".js", "run": "node", "test": "jest"},
        "typescript": {"ext": ".ts", "run": "ts-node", "test": "jest"},
        "go": {"ext": ".go", "run": "go run", "test": "go test"},
        "bash": {"ext": ".sh", "run": "bash", "test": None},
        "cpp": {"ext": ".cpp", "run": "g++", "test": None},
    }

    def __init__(self, workspace_dir: Optional[str] = None):
        self.workspace = Path(workspace_dir or Path.home() / "microdragon_workspace" / "code")
        self.workspace.mkdir(parents=True, exist_ok=True)

    def save_code(self, code: str, language: str, filename: Optional[str] = None) -> Path:
        """Save generated code to workspace."""
        lang_info = self.SUPPORTED_LANGUAGES.get(language.lower(), {"ext": ".txt"})
        if not filename:
            import uuid
            filename = f"microdragon_gen_{uuid.uuid4().hex[:8]}{lang_info['ext']}"

        path = self.workspace / filename
        path.write_text(code, encoding="utf-8")
        return path

    def run_code(self, file_path: str, language: str) -> tuple[bool, str]:
        """Execute code in a safe subprocess."""
        lang_info = self.SUPPORTED_LANGUAGES.get(language.lower())
        if not lang_info:
            return False, f"Unsupported language: {language}"

        runner = lang_info["run"]
        try:
            result = subprocess.run(
                [runner, file_path],
                capture_output=True, text=True, timeout=30,
                cwd=self.workspace
            )
            output = result.stdout + result.stderr
            return result.returncode == 0, output
        except subprocess.TimeoutExpired:
            return False, "Execution timed out (30s limit)"
        except FileNotFoundError:
            return False, f"Runtime '{runner}' not found. Please install it."
        except Exception as e:
            return False, str(e)

    def run_tests(self, project_dir: str, language: str) -> tuple[bool, str]:
        """Run the test suite for a project."""
        lang_info = self.SUPPORTED_LANGUAGES.get(language.lower())
        if not lang_info or not lang_info.get("test"):
            return False, "No test runner configured for this language"

        try:
            result = subprocess.run(
                lang_info["test"].split(),
                capture_output=True, text=True, timeout=120,
                cwd=project_dir
            )
            return result.returncode == 0, result.stdout + result.stderr
        except Exception as e:
            return False, str(e)

    def lint_code(self, file_path: str, language: str) -> tuple[bool, str]:
        """Run linter on code file."""
        linters = {
            "python": ["ruff", "check", file_path],
            "javascript": ["eslint", file_path],
            "typescript": ["tsc", "--noEmit", file_path],
            "rust": ["cargo", "clippy"],
        }
        cmd = linters.get(language.lower())
        if not cmd:
            return True, "No linter configured"

        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            return result.returncode == 0, result.stdout + result.stderr
        except FileNotFoundError:
            return True, "Linter not installed (skipped)"
        except Exception as e:
            return False, str(e)

    def extract_code_blocks(self, text: str) -> list[tuple[str, str]]:
        """Extract ```language ... ``` blocks from AI response."""
        pattern = r"```(\w+)?\n(.*?)```"
        matches = re.findall(pattern, text, re.DOTALL)
        return [(lang or "text", code.strip()) for lang, code in matches]

    def detect_language(self, code: str, hint: Optional[str] = None) -> str:
        """Auto-detect programming language from code."""
        if hint:
            return hint.lower()

        indicators = {
            "python": ["def ", "import ", "print(", "class ", "if __name__"],
            "rust": ["fn main()", "let mut", "use std::", "impl ", "pub fn"],
            "javascript": ["const ", "let ", "function ", "=>", "require("],
            "typescript": ["interface ", ": string", ": number", "readonly "],
            "go": ["package main", "func main()", "import (", ":= "],
            "cpp": ["#include", "std::", "int main(", "cout <<"],
        }

        for lang, patterns in indicators.items():
            if sum(1 for p in patterns if p in code) >= 2:
                return lang

        return "text"


class GitIntegration:
    """Git operations wrapper for MICRODRAGON coding module."""

    def __init__(self, repo_path: Optional[str] = None):
        self.repo_path = repo_path or os.getcwd()

    def _git(self, *args, cwd: Optional[str] = None) -> tuple[bool, str]:
        try:
            result = subprocess.run(
                ["git"] + list(args),
                capture_output=True, text=True,
                cwd=cwd or self.repo_path
            )
            return result.returncode == 0, result.stdout + result.stderr
        except FileNotFoundError:
            return False, "git not installed"

    def status(self) -> str:
        _, output = self._git("status", "--short")
        return output or "Nothing to commit, working tree clean"

    def diff(self, staged: bool = False) -> str:
        args = ["diff"]
        if staged:
            args.append("--staged")
        _, output = self._git(*args)
        return output or "No changes"

    def log(self, n: int = 10) -> str:
        _, output = self._git("log", f"--oneline", f"-{n}", "--graph")
        return output

    def stage_all(self) -> tuple[bool, str]:
        return self._git("add", "-A")

    def commit(self, message: str) -> tuple[bool, str]:
        return self._git("commit", "-m", message)

    def generate_commit_message(self, diff: str) -> str:
        """Generate a conventional commit message from a diff summary."""
        # Classify change type
        if "def " in diff or "fn " in diff or "function " in diff:
            prefix = "feat"
        elif "fix" in diff.lower() or "bug" in diff.lower():
            prefix = "fix"
        elif "test" in diff.lower():
            prefix = "test"
        elif "README" in diff or ".md" in diff:
            prefix = "docs"
        else:
            prefix = "chore"

        # Count changed files
        added = diff.count("\n+") - diff.count("\n+++")
        removed = diff.count("\n-") - diff.count("\n---")
        return f"{prefix}: update {added} additions, {removed} deletions"

    def create_branch(self, name: str) -> tuple[bool, str]:
        return self._git("checkout", "-b", name)

    def push(self, remote: str = "origin", branch: str = "main") -> tuple[bool, str]:
        return self._git("push", remote, branch)


if __name__ == "__main__":
    # Module test
    engine = CodingEngine()
    print(f"[MICRODRAGON Coding] Workspace: {engine.workspace}")
    git = GitIntegration()
    print(f"[MICRODRAGON Git] Status:\n{git.status()}")
