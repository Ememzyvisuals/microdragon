"""
microdragon/modules/simulation/src/engine.py
MICRODRAGON Simulation Module — Desktop automation, visual AI, app control
"""

import subprocess
import sys
import os
import json
import time
from dataclasses import dataclass
from typing import Optional, Tuple
from pathlib import Path


@dataclass
class SimResult:
    success: bool
    output: str = ""
    screenshot_path: Optional[str] = None
    error: str = ""


class ScreenCapture:
    """Cross-platform screen capture."""

    def capture(self, output_path: str = "screen.png") -> SimResult:
        try:
            import pyautogui
            img = pyautogui.screenshot()
            img.save(output_path)
            return SimResult(success=True, output=f"Screenshot saved: {output_path}",
                             screenshot_path=output_path)
        except ImportError:
            return self._capture_fallback(output_path)
        except Exception as e:
            return SimResult(success=False, error=str(e))

    def _capture_fallback(self, path: str) -> SimResult:
        """OS-native screenshot fallback."""
        if sys.platform == "win32":
            cmd = ["powershell", "-Command",
                   f"Add-Type -AssemblyName System.Windows.Forms; "
                   f"[System.Windows.Forms.Screen]::PrimaryScreen | "
                   f"ForEach-Object {{ $bmp = New-Object System.Drawing.Bitmap($_.Bounds.Width,$_.Bounds.Height); "
                   f"[System.Windows.Forms.Screen]::PrimaryScreen }}"]
        elif sys.platform == "darwin":
            cmd = ["screencapture", "-x", path]
        else:
            cmd = ["scrot", path]

        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=10)
            return SimResult(success=result.returncode == 0, output=path,
                             screenshot_path=path if result.returncode == 0 else None)
        except Exception as e:
            return SimResult(success=False, error=str(e))

    def capture_region(self, x: int, y: int, width: int, height: int,
                        output_path: str = "region.png") -> SimResult:
        try:
            import pyautogui
            from PIL import Image
            full = pyautogui.screenshot()
            region = full.crop((x, y, x + width, y + height))
            region.save(output_path)
            return SimResult(success=True, output=output_path, screenshot_path=output_path)
        except Exception as e:
            return SimResult(success=False, error=str(e))


class VisionAnalyzer:
    """OpenCV-based visual analysis for automation."""

    def find_element(self, screenshot_path: str, template_path: str,
                     threshold: float = 0.8) -> Optional[Tuple[int, int]]:
        """Find a UI element in a screenshot using template matching."""
        try:
            import cv2
            import numpy as np

            screenshot = cv2.imread(screenshot_path)
            template = cv2.imread(template_path)
            if screenshot is None or template is None:
                return None

            result = cv2.matchTemplate(screenshot, template, cv2.TM_CCOEFF_NORMED)
            _, max_val, _, max_loc = cv2.minMaxLoc(result)

            if max_val >= threshold:
                h, w = template.shape[:2]
                center_x = max_loc[0] + w // 2
                center_y = max_loc[1] + h // 2
                return (center_x, center_y)
            return None
        except ImportError:
            print("[Vision] OpenCV not available. Install: pip install opencv-python-headless")
            return None

    def extract_text_from_image(self, image_path: str) -> str:
        """OCR via tesseract if available."""
        try:
            import pytesseract
            from PIL import Image
            img = Image.open(image_path)
            return pytesseract.image_to_string(img)
        except ImportError:
            return "[OCR not available — install pytesseract]"

    def describe_screen(self, screenshot_path: str) -> dict:
        """Basic screen analysis — dimensions, dominant colors, text regions."""
        try:
            import cv2
            import numpy as np
            img = cv2.imread(screenshot_path)
            if img is None:
                return {"error": "Could not read image"}

            h, w = img.shape[:2]
            # Dominant colors (top 5)
            pixels = img.reshape(-1, 3).astype(float)
            # Simplified: just get mean color per quadrant
            quadrants = {
                "top_left": img[:h//2, :w//2].mean(axis=(0,1)).tolist(),
                "top_right": img[:h//2, w//2:].mean(axis=(0,1)).tolist(),
                "bottom_left": img[h//2:, :w//2].mean(axis=(0,1)).tolist(),
                "bottom_right": img[h//2:, w//2:].mean(axis=(0,1)).tolist(),
            }
            return {"width": w, "height": h, "quadrant_colors": quadrants}
        except ImportError:
            return {"error": "OpenCV not installed"}


class AppController:
    """Launch and control desktop applications."""

    def launch(self, app_name: str) -> SimResult:
        """Launch an application by name."""
        platform_cmds = {
            "win32": ["start", app_name],
            "darwin": ["open", "-a", app_name],
        }
        if sys.platform in platform_cmds:
            cmd = platform_cmds[sys.platform]
            try:
                subprocess.Popen(cmd, shell=(sys.platform == "win32"))
                return SimResult(success=True, output=f"Launched: {app_name}")
            except Exception as e:
                return SimResult(success=False, error=str(e))
        else:
            # Linux: try common launchers
            for launcher in [app_name, f"{app_name}.AppImage"]:
                try:
                    subprocess.Popen([launcher])
                    return SimResult(success=True, output=f"Launched: {launcher}")
                except FileNotFoundError:
                    continue
            return SimResult(success=False, error=f"Could not launch {app_name}")

    def list_windows(self) -> list:
        """List open window titles."""
        if sys.platform == "win32":
            try:
                import ctypes
                # Simplified: use tasklist
                result = subprocess.run(["tasklist", "/fo", "csv"],
                                        capture_output=True, text=True)
                return [line.split(",")[0].strip('"') for line in result.stdout.splitlines()[1:]]
            except Exception:
                return []
        elif sys.platform == "darwin":
            try:
                result = subprocess.run(
                    ["osascript", "-e",
                     'tell application "System Events" to get name of every process whose background only is false'],
                    capture_output=True, text=True
                )
                return result.stdout.strip().split(", ")
            except Exception:
                return []
        return []


class MouseKeyboard:
    """Safe mouse and keyboard control with validation."""

    SAFE_MODE = True

    def move_mouse(self, x: int, y: int, duration: float = 0.3) -> SimResult:
        try:
            import pyautogui
            pyautogui.moveTo(x, y, duration=duration)
            return SimResult(success=True, output=f"Mouse moved to ({x}, {y})")
        except Exception as e:
            return SimResult(success=False, error=str(e))

    def click(self, x: int, y: int, button: str = "left") -> SimResult:
        try:
            import pyautogui
            pyautogui.click(x=x, y=y, button=button)
            return SimResult(success=True, output=f"Clicked ({x}, {y})")
        except Exception as e:
            return SimResult(success=False, error=str(e))

    def type_text(self, text: str, interval: float = 0.05) -> SimResult:
        if self.SAFE_MODE and len(text) > 500:
            return SimResult(success=False,
                             error="Text too long for safe typing (limit 500 chars in safe mode)")
        try:
            import pyautogui
            pyautogui.typewrite(text, interval=interval)
            return SimResult(success=True, output=f"Typed {len(text)} characters")
        except Exception as e:
            return SimResult(success=False, error=str(e))

    def hotkey(self, *keys) -> SimResult:
        try:
            import pyautogui
            pyautogui.hotkey(*keys)
            return SimResult(success=True, output=f"Hotkey: {'+'.join(keys)}")
        except Exception as e:
            return SimResult(success=False, error=str(e))


class SimulationEngine:
    """Unified simulation engine — orchestrates all automation tools."""

    def __init__(self):
        self.screen = ScreenCapture()
        self.vision = VisionAnalyzer()
        self.app = AppController()
        self.io = MouseKeyboard()

    def run_workflow(self, steps: list[dict]) -> list[SimResult]:
        """Execute a list of automation steps."""
        results = []
        for step in steps:
            action = step.get("action", "")
            params = step.get("params", {})

            if action == "screenshot":
                r = self.screen.capture(params.get("path", "screen.png"))
            elif action == "click":
                r = self.io.click(params.get("x", 0), params.get("y", 0),
                                  params.get("button", "left"))
            elif action == "type":
                r = self.io.type_text(params.get("text", ""))
            elif action == "hotkey":
                r = self.io.hotkey(*params.get("keys", []))
            elif action == "launch":
                r = self.app.launch(params.get("app", ""))
            elif action == "wait":
                time.sleep(params.get("seconds", 1))
                r = SimResult(success=True, output=f"Waited {params.get('seconds', 1)}s")
            else:
                r = SimResult(success=False, error=f"Unknown action: {action}")

            results.append(r)
            if not r.success and step.get("abort_on_fail", False):
                break

        return results


if __name__ == "__main__":
    engine = SimulationEngine()
    print("[MICRODRAGON Simulation] Engine ready")
    print(f"  Platform: {sys.platform}")
    print(f"  Safe mode: {MouseKeyboard.SAFE_MODE}")
    windows = engine.app.list_windows()
    print(f"  Open windows: {len(windows)}")
