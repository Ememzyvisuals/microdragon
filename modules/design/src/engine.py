"""
microdragon/modules/design/src/engine.py
MICRODRAGON Design Module — UI generation, mockup creation, design automation
"""

import asyncio
import json
import os
import subprocess
import sys
from dataclasses import dataclass
from typing import Optional
from pathlib import Path


@dataclass
class DesignOutput:
    success: bool
    output_path: Optional[str] = None
    content: str = ""
    format: str = ""
    error: str = ""


class UIGenerator:
    """Generate UI code from natural language descriptions."""

    def generate_html(self, description: str, style: str = "modern") -> DesignOutput:
        """Generate a complete HTML/CSS page from description."""
        # This generates a structured prompt template — brain fills it
        template_prompt = f"""
Generate a complete, beautiful HTML page with embedded CSS for: {description}

Style: {style}
Requirements:
- Single file HTML with inline CSS
- Responsive design (mobile-first)
- Modern, professional appearance
- Working layout with all described elements
- CSS variables for theming
- No JavaScript dependencies required
- Semantic HTML5

Output ONLY the HTML code, no explanation.
"""
        return DesignOutput(
            success=True,
            content=template_prompt,
            format="html_prompt"
        )

    def generate_tailwind_component(self, description: str) -> DesignOutput:
        """Generate a Tailwind CSS component."""
        prompt = f"""
Generate a Tailwind CSS component for: {description}

Requirements:
- Use only Tailwind utility classes
- Clean, modern design
- Accessible (ARIA labels where needed)
- Dark mode compatible (use dark: variants)
- Output only the HTML with Tailwind classes

Output ONLY the HTML component code.
"""
        return DesignOutput(success=True, content=prompt, format="tailwind_prompt")

    def generate_svg(self, description: str) -> DesignOutput:
        """Generate an SVG graphic from description."""
        prompt = f"""
Generate a clean SVG graphic for: {description}

Requirements:
- Valid SVG code
- Viewbox="0 0 100 100" or appropriate dimensions
- Clean, minimal design
- Use paths, circles, rectangles as appropriate
- Include title element for accessibility

Output ONLY the SVG code.
"""
        return DesignOutput(success=True, content=prompt, format="svg_prompt")

    def save_design(self, content: str, filename: str, base_dir: str = ".") -> DesignOutput:
        """Save generated design to file."""
        try:
            path = Path(base_dir) / filename
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
            return DesignOutput(success=True, output_path=str(path),
                                content=content, format=path.suffix)
        except Exception as e:
            return DesignOutput(success=False, error=str(e))


class ColorPalette:
    """Color palette generation and manipulation."""

    def generate_palette(self, base_color: str, style: str = "complementary") -> dict:
        """Generate a color palette from a base color."""
        # Parse hex color
        try:
            r, g, b = self._hex_to_rgb(base_color.lstrip("#"))
        except Exception:
            return {"error": f"Invalid color: {base_color}"}

        h, s, v = self._rgb_to_hsv(r, g, b)

        palettes = {
            "complementary": [
                base_color,
                self._hsv_to_hex((h + 180) % 360, s, v),
                self._hsv_to_hex(h, s * 0.7, min(v * 1.2, 1.0)),
                self._hsv_to_hex(h, s * 0.4, min(v * 1.4, 1.0)),
                self._hsv_to_hex(h, s * 0.1, 0.95),
            ],
            "analogous": [
                self._hsv_to_hex((h - 30) % 360, s, v),
                self._hsv_to_hex((h - 15) % 360, s, v),
                base_color,
                self._hsv_to_hex((h + 15) % 360, s, v),
                self._hsv_to_hex((h + 30) % 360, s, v),
            ],
            "monochromatic": [
                self._hsv_to_hex(h, s, v * 0.4),
                self._hsv_to_hex(h, s, v * 0.6),
                self._hsv_to_hex(h, s, v * 0.8),
                base_color,
                self._hsv_to_hex(h, s * 0.5, min(v * 1.2, 1.0)),
            ],
        }

        return {
            "base": base_color,
            "style": style,
            "colors": palettes.get(style, palettes["complementary"]),
            "css_vars": self._to_css_vars(palettes.get(style, palettes["complementary"]))
        }

    def _hex_to_rgb(self, hex_str: str) -> tuple:
        h = hex_str.lstrip("#")
        return tuple(int(h[i:i+2], 16) / 255.0 for i in (0, 2, 4))

    def _rgb_to_hsv(self, r: float, g: float, b: float) -> tuple:
        mx = max(r, g, b)
        mn = min(r, g, b)
        diff = mx - mn
        v = mx
        s = diff / mx if mx != 0 else 0
        if diff == 0:
            h = 0
        elif mx == r:
            h = (60 * ((g - b) / diff) + 360) % 360
        elif mx == g:
            h = (60 * ((b - r) / diff) + 120) % 360
        else:
            h = (60 * ((r - g) / diff) + 240) % 360
        return h, s, v

    def _hsv_to_hex(self, h: float, s: float, v: float) -> str:
        h = h % 360
        i = int(h / 60) % 6
        f = h / 60 - i
        p = v * (1 - s)
        q = v * (1 - f * s)
        t = v * (1 - (1 - f) * s)
        rgb_map = [(v,t,p),(q,v,p),(p,v,t),(p,q,v),(t,p,v),(v,p,q)]
        r, g, b = rgb_map[i]
        return "#{:02x}{:02x}{:02x}".format(int(r*255), int(g*255), int(b*255))

    def _to_css_vars(self, colors: list) -> str:
        names = ["--color-primary", "--color-secondary", "--color-accent",
                 "--color-light", "--color-surface"]
        pairs = [f"  {names[i]}: {c};" for i, c in enumerate(colors[:5])]
        return ":root {\n" + "\n".join(pairs) + "\n}"


class ScreenshotToCode:
    """Convert screenshots/images to HTML code using vision AI."""

    async def convert(self, image_path: str) -> str:
        """Build a prompt for vision AI to convert image to code."""
        if not os.path.exists(image_path):
            return f"Image not found: {image_path}"

        return f"""
Analyze this UI screenshot and generate clean HTML/CSS code that recreates it.

Requirements:
- Match the layout, colors, and typography as closely as possible
- Use semantic HTML5
- Inline CSS only (no external files)
- Responsive and functional
- Include any visible text content

Output ONLY the HTML code.
[IMAGE: {image_path}]
"""


class DesignEngine:
    """Unified design engine for MICRODRAGON."""

    def __init__(self, output_dir: str = None):
        self.output_dir = output_dir or str(Path.home() / "microdragon_workspace" / "designs")
        Path(self.output_dir).mkdir(parents=True, exist_ok=True)
        self.generator = UIGenerator()
        self.palette = ColorPalette()
        self.s2c = ScreenshotToCode()

    def design_prompt(self, task: str, context: dict = None) -> str:
        """Build the best prompt for a design task."""
        ctx = context or {}
        style = ctx.get("style", "modern professional")
        framework = ctx.get("framework", "vanilla HTML/CSS")

        return f"""
You are an expert UI/UX designer and frontend developer.

Task: {task}
Style: {style}
Framework: {framework}
Output directory: {self.output_dir}

Produce complete, pixel-perfect, production-ready code.
Include all assets inline. Make it beautiful and functional.
"""


if __name__ == "__main__":
    engine = DesignEngine()
    print(f"[MICRODRAGON Design] Engine ready")
    print(f"  Output dir: {engine.output_dir}")

    # Test palette generation
    p = engine.palette.generate_palette("#2563eb", "analogous")
    print(f"  Sample palette: {p['colors']}")
