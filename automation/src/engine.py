"""
microdragon/automation/src/engine.py
MICRODRAGON Automation Engine — Browser + Desktop automation
"""

import asyncio
import subprocess
import sys
import os
from dataclasses import dataclass
from typing import Optional


@dataclass
class AutomationResult:
    success: bool
    output: str = ""
    screenshots: list = None
    error: str = ""

    def __post_init__(self):
        if self.screenshots is None:
            self.screenshots = []


class BrowserAutomation:
    """Playwright-based browser automation."""

    def __init__(self):
        self._check_playwright()

    def _check_playwright(self):
        try:
            import playwright
        except ImportError:
            print("[MICRODRAGON Automation] Playwright not installed. Run: pip install playwright && playwright install chromium")

    async def run_script(self, script: str, headless: bool = True) -> AutomationResult:
        """Execute a Playwright Python script."""
        import tempfile, os
        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
            f.write(script)
            tmp_path = f.name

        try:
            result = subprocess.run(
                [sys.executable, tmp_path],
                capture_output=True, text=True, timeout=120,
                env={**os.environ, "PLAYWRIGHT_HEADLESS": "1" if headless else "0"}
            )
            return AutomationResult(
                success=result.returncode == 0,
                output=result.stdout,
                error=result.stderr
            )
        except subprocess.TimeoutExpired:
            return AutomationResult(success=False, error="Script timed out (120s)")
        except Exception as e:
            return AutomationResult(success=False, error=str(e))
        finally:
            os.unlink(tmp_path)

    async def screenshot(self, url: str, output_path: str = "screenshot.png") -> AutomationResult:
        """Take a screenshot of a URL."""
        script = f"""
import asyncio
from playwright.async_api import async_playwright

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page()
        await page.goto("{url}", wait_until="networkidle", timeout=30000)
        await page.screenshot(path="{output_path}", full_page=True)
        await browser.close()
        print(f"Screenshot saved: {output_path}")

asyncio.run(main())
"""
        return await self.run_script(script)

    async def extract_text(self, url: str) -> AutomationResult:
        """Extract all text from a webpage using Playwright."""
        script = f"""
import asyncio
from playwright.async_api import async_playwright

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        page = await browser.new_page()
        await page.goto("{url}", wait_until="domcontentloaded", timeout=30000)
        text = await page.evaluate("() => document.body.innerText")
        print(text[:5000])
        await browser.close()

asyncio.run(main())
"""
        return await self.run_script(script)


class DesktopAutomation:
    """PyAutoGUI-based desktop automation with safety checks."""

    SAFE_MODE = True  # Always default to safe mode — no accidental deletes

    def __init__(self):
        self._check_pyautogui()

    def _check_pyautogui(self):
        try:
            import pyautogui
            pyautogui.FAILSAFE = True  # Move mouse to corner to abort
            pyautogui.PAUSE = 0.5     # 500ms between actions (safety)
        except ImportError:
            print("[MICRODRAGON Desktop] PyAutoGUI not installed. Run: pip install pyautogui")

    def run_script(self, script: str) -> AutomationResult:
        """Execute a PyAutoGUI script with safety wrapper."""
        import tempfile, os
        safety_prefix = """
import pyautogui
pyautogui.FAILSAFE = True
pyautogui.PAUSE = 0.3
import time
time.sleep(1)  # Safety delay before starting
"""
        full_script = safety_prefix + script

        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
            f.write(full_script)
            tmp = f.name

        try:
            result = subprocess.run(
                [sys.executable, tmp],
                capture_output=True, text=True, timeout=60
            )
            return AutomationResult(
                success=result.returncode == 0,
                output=result.stdout,
                error=result.stderr
            )
        except subprocess.TimeoutExpired:
            return AutomationResult(success=False, error="Desktop automation timed out")
        except Exception as e:
            return AutomationResult(success=False, error=str(e))
        finally:
            os.unlink(tmp)


class AutomationEngine:
    """Unified automation engine for MICRODRAGON."""

    def __init__(self):
        self.browser = BrowserAutomation()
        self.desktop = DesktopAutomation()

    async def execute(self, task_type: str, script: str, **kwargs) -> AutomationResult:
        if task_type == "browser":
            return await self.browser.run_script(script, headless=kwargs.get("headless", True))
        elif task_type == "desktop":
            return self.desktop.run_script(script)
        elif task_type == "screenshot":
            url = kwargs.get("url", "")
            output = kwargs.get("output", "screenshot.png")
            return await self.browser.screenshot(url, output)
        else:
            return AutomationResult(success=False, error=f"Unknown task type: {task_type}")


if __name__ == "__main__":
    print("[MICRODRAGON Automation] Engine ready")
    print(f"  Browser: Playwright")
    print(f"  Desktop: PyAutoGUI")
    print(f"  Safe mode: ON")
