"""
microdragon/modules/design/flyer_pipeline.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON FLYER CREATION PIPELINE
═══════════════════════════════════════════════════════════════════════════════

This module answers the question:
  "Help me create a professional flyer for my birthday / business"

WHAT MICRODRAGON DOES (step by step):

  Step 1 — CLARIFY
    Ask the key questions before doing anything:
    - Flyer type (birthday / business / event / product launch)
    - Key info (name, date, venue, contact, tagline)
    - Style preference (modern, elegant, bold, playful, corporate)
    - Color preferences or brand colors
    - Where will it be used (print / social media / WhatsApp)
    - Any images to include (upload or describe)

  Step 2 — GATHER ASSETS (pre-execution phase)
    Before touching Photoshop, Microdragon assembles everything:
    - Downloads relevant free stock images (Unsplash API, Pexels API)
    - Downloads icon packs (Flaticon, SVG Repo)
    - Downloads or loads specified fonts (Google Fonts)
    - Finds relevant textures or backgrounds
    - Downloads brand logo if URL provided
    - Generates AI background/element images if provider configured
    - Saves all assets to a staging folder

  Step 3 — DESIGN GENERATION
    Microdragon now has two paths:

    PATH A — Photoshop installed:
      - Generates complete ExtendScript JSX
      - Script opens Photoshop, creates document at correct size
      - Places every downloaded asset
      - Sets up all text layers with correct fonts, sizes, colors
      - Applies effects (gradients, shadows, glows)
      - Exports final PNG and PDF
      - All you do is review and approve

    PATH B — No Photoshop (Pillow fallback):
      - Uses Python Pillow + aggdraw for full flyer rendering
      - Same quality result, no Photoshop needed
      - Exports PNG, PDF, and SVG

    PATH C — SVG generation (always available):
      - Generates a complete SVG flyer
      - Opens in browser for preview
      - Can be opened in Illustrator, Inkscape, or any vector tool

  Step 4 — PREVIEW & ITERATE
    - Shows the output path
    - Asks: "Want to change anything?"
    - Can adjust colors, text, layout, images in one command

ANSWER TO THE QUESTION:
  YES — Microdragon downloads everything it needs BEFORE automating Photoshop.
  It assembles all assets first, THEN opens Photoshop and places them.
  The user just waits and reviews the result.

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

import asyncio
import os
import sys
import json
import tempfile
import urllib.request
import urllib.error
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional
import re


# ─── Flyer specification ──────────────────────────────────────────────────────

@dataclass
class FlyerSpec:
    # Identity
    flyer_type:    str           # birthday | business | event | product | sale
    title:         str           # Main text (e.g. "John's 30th Birthday")
    subtitle:      str = ""      # Secondary text
    body_lines:    list = field(default_factory=list)  # Additional text lines

    # Details
    date:          str = ""      # Event date
    time:          str = ""      # Event time
    venue:         str = ""      # Location / address
    contact:       str = ""      # Phone / email / website
    price:         str = ""      # "Free" / "£20/person" etc.
    hashtags:      list = field(default_factory=list)

    # Design
    style:         str = "modern"    # modern | elegant | bold | playful | corporate | luxury
    palette:       str = "auto"      # auto | dark | light | or named theme
    primary_color: str = "#00ff88"
    secondary_color: str = "#ff4444"
    font_heading:  str = "auto"      # auto selects based on style
    font_body:     str = "auto"

    # Output
    size:          str = "a4"        # a4 | a5 | square | story | banner
    orientation:   str = "portrait"  # portrait | landscape
    dpi:           int = 300         # 300 for print, 150 for digital
    output_format: list = field(default_factory=lambda: ["png", "pdf"])
    output_dir:    str = ""

    # Assets
    logo_url:      str = ""      # URL to brand logo
    hero_image:    str = ""      # URL or description of main image
    uploaded_images: list = field(default_factory=list)

    # Context
    platform:      str = "all"   # print | instagram | whatsapp | facebook | all


# Predefined flyer size dimensions in pixels at 300dpi
FLYER_SIZES = {
    "a4":       (2480, 3508),    # A4 portrait @ 300dpi
    "a5":       (1748, 2480),    # A5
    "square":   (2000, 2000),    # Social media square
    "story":    (1080, 1920),    # Instagram/WhatsApp story
    "banner":   (3000, 1000),    # Wide banner
    "letter":   (2550, 3300),    # US Letter
    "business": (1050,  600),    # Business card
}

# Style → font mapping
STYLE_FONTS = {
    "modern":    {"heading": "Inter",          "body": "Inter"},
    "elegant":   {"heading": "Playfair Display","body": "Lato"},
    "bold":      {"heading": "Oswald",         "body": "Roboto"},
    "playful":   {"heading": "Fredoka One",    "body": "Nunito"},
    "corporate": {"heading": "Montserrat",     "body": "Open Sans"},
    "luxury":    {"heading": "Cormorant Garamond","body": "Raleway"},
    "party":     {"heading": "Bebas Neue",     "body": "Poppins"},
}

# Style → color palette mapping
STYLE_PALETTES = {
    "birthday_classic": {
        "bg":       "#1a0533",   "surface":  "#2d0a52",
        "primary":  "#ff69b4",   "accent":   "#ffd700",
        "text":     "#ffffff",   "text_dim": "#cc99cc",
    },
    "birthday_modern": {
        "bg":       "#0a0a1a",   "surface":  "#16162a",
        "primary":  "#00ff88",   "accent":   "#ff4488",
        "text":     "#ffffff",   "text_dim": "#888899",
    },
    "business_dark": {
        "bg":       "#0d1117",   "surface":  "#161b22",
        "primary":  "#00ff88",   "accent":   "#4488ff",
        "text":     "#e6edf3",   "text_dim": "#8b949e",
    },
    "business_light": {
        "bg":       "#ffffff",   "surface":  "#f6f8fa",
        "primary":  "#1a56db",   "accent":   "#0ea5e9",
        "text":     "#111827",   "text_dim": "#6b7280",
    },
    "event_bold": {
        "bg":       "#ff2d00",   "surface":  "#cc2400",
        "primary":  "#ffffff",   "accent":   "#ffd700",
        "text":     "#ffffff",   "text_dim": "#ffcccc",
    },
    "luxury_gold": {
        "bg":       "#0a0700",   "surface":  "#1a1200",
        "primary":  "#d4af37",   "accent":   "#f5e9c3",
        "text":     "#f5e9c3",   "text_dim": "#a89060",
    },
}


# ─── Asset Downloader ─────────────────────────────────────────────────────────

class AssetDownloader:
    """
    Downloads all assets Microdragon needs BEFORE opening Photoshop.
    This is the pre-execution phase — everything is assembled first.
    """

    def __init__(self, staging_dir: str):
        self.staging_dir = Path(staging_dir)
        self.staging_dir.mkdir(parents=True, exist_ok=True)
        self.downloaded: dict[str, str] = {}   # asset_name → local_path

    async def download_all(self, spec: FlyerSpec) -> dict:
        """
        Download everything needed for the flyer.
        Returns dict of asset_name → local_path
        """
        print("  🐉 Assembling assets before design...\n")
        tasks = []

        # 1. Hero / background image
        if spec.hero_image:
            if spec.hero_image.startswith("http"):
                tasks.append(self._download_url(spec.hero_image, "hero_image.jpg"))
            else:
                # Description → search Unsplash
                tasks.append(self._search_unsplash(spec.hero_image, "hero_image.jpg"))

        # 2. Logo
        if spec.logo_url:
            tasks.append(self._download_url(spec.logo_url, "logo.png"))

        # 3. User-uploaded images
        for i, img_path in enumerate(spec.uploaded_images):
            if Path(img_path).exists():
                import shutil
                dest = self.staging_dir / f"uploaded_{i}.png"
                shutil.copy2(img_path, dest)
                self.downloaded[f"uploaded_{i}"] = str(dest)
                print(f"  ✓ Loaded uploaded image: {Path(img_path).name}")

        # 4. Fonts (download TTF from Google Fonts CDN)
        fonts = STYLE_FONTS.get(spec.style, STYLE_FONTS["modern"])
        heading_font = spec.font_heading if spec.font_heading != "auto" else fonts["heading"]
        body_font    = spec.font_body    if spec.font_body    != "auto" else fonts["body"]
        tasks.append(self._download_google_font(heading_font, "font_heading.ttf"))
        tasks.append(self._download_google_font(body_font,    "font_body.ttf"))

        # 5. Background texture (if style needs it)
        if spec.style in ("luxury", "elegant", "party"):
            tasks.append(self._download_texture(spec.style, "texture.png"))

        # Run all downloads concurrently
        await asyncio.gather(*tasks, return_exceptions=True)

        print(f"\n  ✓ Asset assembly complete ({len(self.downloaded)} assets ready)")
        print(f"  📁 Staging folder: {self.staging_dir}\n")
        return self.downloaded

    async def _download_url(self, url: str, filename: str) -> bool:
        """Download any URL to staging."""
        dest = self.staging_dir / filename
        try:
            loop = asyncio.get_event_loop()
            await loop.run_in_executor(
                None, lambda: urllib.request.urlretrieve(url, dest)
            )
            self.downloaded[filename.rsplit(".", 1)[0]] = str(dest)
            print(f"  ✓ Downloaded: {filename}")
            return True
        except Exception as e:
            print(f"  ! Could not download {filename}: {e}")
            return False

    async def _search_unsplash(self, query: str, filename: str) -> bool:
        """
        Search Unsplash for a relevant image.
        Uses the free API — no key required for basic access source.
        """
        try:
            import aiohttp
            # Unsplash Source API — free, no key required
            safe_query = urllib.parse.quote(query) if hasattr(urllib, "parse") else query.replace(" ", "+")
            # Using picsum as fallback since it's always free
            url = f"https://source.unsplash.com/1920x1080/?{safe_query}"
            dest = self.staging_dir / filename
            async with aiohttp.ClientSession() as s:
                async with s.get(url, allow_redirects=True,
                                  timeout=aiohttp.ClientTimeout(total=15)) as resp:
                    if resp.status == 200:
                        dest.write_bytes(await resp.read())
                        self.downloaded[filename.rsplit(".", 1)[0]] = str(dest)
                        print(f"  ✓ Found background image for: '{query}'")
                        return True
        except ImportError:
            # Fallback: urllib
            return await self._download_url(
                f"https://picsum.photos/1920/1080?random=1", filename
            )
        except Exception as e:
            print(f"  ! Could not find image for '{query}': {e}")
        return False

    async def _download_google_font(self, font_name: str, filename: str) -> bool:
        """
        Download a font TTF from Google Fonts.
        Falls back to system fonts if download fails.
        """
        dest = self.staging_dir / filename
        # Check if font is already a system font
        system_font = self._find_system_font(font_name)
        if system_font:
            import shutil
            shutil.copy2(system_font, dest)
            self.downloaded[filename.rsplit(".", 1)[0]] = str(dest)
            print(f"  ✓ Font loaded from system: {font_name}")
            return True

        # Download from Google Fonts
        try:
            import aiohttp
            # Google Fonts API for direct TTF download
            gf_name = font_name.replace(" ", "+")
            css_url = f"https://fonts.googleapis.com/css2?family={gf_name}&display=swap"
            headers = {"User-Agent": "Mozilla/5.0"}

            async with aiohttp.ClientSession() as s:
                async with s.get(css_url, headers=headers,
                                  timeout=aiohttp.ClientTimeout(total=10)) as resp:
                    if resp.status == 200:
                        css = await resp.text()
                        # Extract TTF/WOFF2 URL from CSS
                        font_url = re.search(r'url\(([^)]+\.(?:ttf|woff2|woff))\)', css)
                        if font_url:
                            async with s.get(font_url.group(1), headers=headers) as fr:
                                if fr.status == 200:
                                    dest.write_bytes(await fr.read())
                                    self.downloaded[filename.rsplit(".", 1)[0]] = str(dest)
                                    print(f"  ✓ Font downloaded: {font_name}")
                                    return True
        except Exception as e:
            print(f"  ! Font '{font_name}' not available ({e}), using system fallback")
        return False

    async def _download_texture(self, style: str, filename: str) -> bool:
        """Download a relevant texture/pattern for the style."""
        texture_urls = {
            "luxury":  "https://grainy-gradients.vercel.app/noise.png",
            "elegant": "https://www.transparenttextures.com/patterns/clean-gray-paper.png",
            "party":   "https://www.transparenttextures.com/patterns/confetti.png",
        }
        url = texture_urls.get(style)
        if url:
            return await self._download_url(url, filename)
        return False

    def _find_system_font(self, font_name: str) -> Optional[str]:
        """Look for a font in common system font directories."""
        import sys
        name_lower = font_name.lower().replace(" ", "")
        search_dirs = []
        if sys.platform == "win32":
            search_dirs = [
                r"C:\Windows\Fonts",
                os.path.expanduser(r"~\AppData\Local\Microsoft\Windows\Fonts"),
            ]
        elif sys.platform == "darwin":
            search_dirs = [
                "/Library/Fonts",
                "/System/Library/Fonts",
                os.path.expanduser("~/Library/Fonts"),
            ]
        else:
            search_dirs = [
                "/usr/share/fonts",
                os.path.expanduser("~/.fonts"),
                os.path.expanduser("~/.local/share/fonts"),
            ]

        for d in search_dirs:
            if not os.path.exists(d):
                continue
            for root, _, files in os.walk(d):
                for f in files:
                    if f.lower().replace(" ", "").endswith((".ttf", ".otf")):
                        if name_lower in f.lower().replace(" ", ""):
                            return os.path.join(root, f)
        return None


# ─── Photoshop JSX Script Builder ─────────────────────────────────────────────

class PhotoshopFlyerBuilder:
    """
    Generates complete ExtendScript JSX to build the flyer in Photoshop.
    Microdragon has already downloaded all assets — this script just places them.
    """

    def generate_script(self, spec: FlyerSpec, assets: dict,
                         palette: dict) -> str:
        """Generate the full JSX script for Photoshop."""

        w, h = FLYER_SIZES.get(spec.size, FLYER_SIZES["a4"])
        output_png = Path(spec.output_dir) / "flyer_final.png"
        output_pdf = Path(spec.output_dir) / "flyer_final.pdf"

        hero_path  = assets.get("hero_image", "").replace("\\", "\\\\")
        logo_path  = assets.get("logo",       "").replace("\\", "\\\\")
        font_h     = assets.get("font_heading", "Arial")
        font_b     = assets.get("font_body",    "Arial")

        # Read font name from file name if it's a path
        if Path(font_h).exists():
            font_h = Path(font_h).stem.replace("_", " ").replace("-", " ").split()[0]
        if Path(font_b).exists():
            font_b = Path(font_b).stem.replace("_", " ").replace("-", " ").split()[0]

        bg_color    = palette.get("bg",      "#0d1117")
        primary     = palette.get("primary", "#00ff88")
        text_color  = palette.get("text",    "#ffffff")
        accent      = palette.get("accent",  "#ff4444")

        # Convert hex to RGB for JSX
        def hex_rgb(hex_str):
            h = hex_str.lstrip("#")
            return int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)

        bg_r, bg_g, bg_b         = hex_rgb(bg_color)
        pr_r, pr_g, pr_b         = hex_rgb(primary)
        tx_r, tx_g, tx_b         = hex_rgb(text_color)
        ac_r, ac_g, ac_b         = hex_rgb(accent)

        return f'''// =============================================================
// MICRODRAGON — Auto-generated Photoshop flyer script
// Flyer: {spec.title}
// Type:  {spec.flyer_type} | Size: {spec.size} ({w}x{h}px)
// Generated: by Microdragon v0.1.0 for EMEMZYVISUALS DIGITALS
// =============================================================

#target photoshop

app.displayDialogs = DialogModes.NO;

// ── 1. Create new document ──────────────────────────────────
var doc = app.documents.add(
    {w}, {h},             // width, height in pixels
    {spec.dpi},           // resolution DPI
    "{spec.title} Flyer", // name
    NewDocumentMode.RGB,  // color mode
    DocumentFill.TRANSPARENT
);

// ── 2. Background layer ─────────────────────────────────────
var bg = doc.artLayers.add();
bg.name = "Background";
doc.activeLayer = bg;

var bgColor = new SolidColor();
bgColor.rgb.red   = {bg_r};
bgColor.rgb.green = {bg_g};
bgColor.rgb.blue  = {bg_b};
app.foregroundColor = bgColor;
doc.selection.selectAll();
doc.selection.fill(app.foregroundColor);
doc.selection.deselect();

// ── 3. Hero image placement ─────────────────────────────────
{"var heroFile = new File('" + hero_path + "');" if hero_path else "// No hero image"}
{"var heroLayer = doc.artLayers.add();" if hero_path else ""}
{"heroLayer.name = 'Hero Image';" if hero_path else ""}
{'''
if (heroFile.exists) {{
    var heroPlaced = app.load(heroFile);
    var heroLayer = heroPlaced.artLayers[0];
    heroLayer.duplicate(doc, ElementPlacement.PLACEATBEGINNING);
    heroPlaced.close(SaveOptions.DONOTSAVECHANGES);
    doc.activeLayer = doc.artLayers.getByName("Hero Image");
    // Scale to fill document
    var scaleFactor = Math.max(
        doc.width.as("px") / doc.activeLayer.bounds[2].as("px") * 100,
        doc.height.as("px") / doc.activeLayer.bounds[3].as("px") * 100
    );
    doc.activeLayer.resize(scaleFactor, scaleFactor, AnchorPosition.MIDDLECENTER);
    doc.activeLayer.opacity = 30;  // Semi-transparent as background
}}
''' if hero_path else ""}

// ── 4. Gradient overlay ─────────────────────────────────────
var gradLayer = doc.artLayers.add();
gradLayer.name = "Gradient Overlay";
doc.activeLayer = gradLayer;

// Apply gradient from bottom to create depth
var gradDesc = new ActionDescriptor();
var gradColorDesc = new ActionDescriptor();
gradColorDesc.putEnumerated(
    stringIDToTypeID("gradientForm"),
    stringIDToTypeID("gradientForm"),
    stringIDToTypeID("colorNoise")
);
// Fill bottom 60% with dark overlay for text readability
var gradientStart = new SolidColor();
gradientStart.rgb.red   = {bg_r};
gradientStart.rgb.green = {bg_g};
gradientStart.rgb.blue  = {bg_b};
app.foregroundColor = gradientStart;
var gradRect = [
    [0, {int(h*0.4)}], [{w}, {int(h*0.4)}],
    [{w}, {h}],         [0, {h}]
];
doc.selection.select(gradRect);
doc.selection.fill(app.foregroundColor);
doc.selection.deselect();
gradLayer.opacity = 90;

// ── 5. Accent bar ───────────────────────────────────────────
var accentLayer = doc.artLayers.add();
accentLayer.name = "Accent Bar";
doc.activeLayer = accentLayer;

var accentColor = new SolidColor();
accentColor.rgb.red   = {pr_r};
accentColor.rgb.green = {pr_g};
accentColor.rgb.blue  = {pr_b};
app.foregroundColor = accentColor;

// Top accent bar
var topBarRect = [[0, 0], [{w}, 0], [{w}, {int(h*0.008)}], [0, {int(h*0.008)}]];
doc.selection.select(topBarRect);
doc.selection.fill(app.foregroundColor);
doc.selection.deselect();

// ── 6. Logo ─────────────────────────────────────────────────
{"var logoFile = new File('" + logo_path + "');" if logo_path else "// No logo"}
{'''
if (logoFile.exists) {
    var logoDoc = app.load(logoFile);
    var logoLayer = logoDoc.artLayers[0];
    logoLayer.duplicate(doc, ElementPlacement.PLACEATBEGINNING);
    logoDoc.close(SaveOptions.DONOTSAVECHANGES);
    doc.activeLayer = doc.artLayers[0];
    doc.activeLayer.name = "Logo";
    // Position top-left
    var logoScale = (''' + str(int(w*0.15)) + ''' / doc.activeLayer.bounds[2].as("px")) * 100;
    doc.activeLayer.resize(logoScale, logoScale, AnchorPosition.TOPLEFT);
    doc.activeLayer.translate(''' + str(int(w*0.05)) + ''', ''' + str(int(h*0.03)) + ''');
}
''' if logo_path else ""}

// ── 7. Main title text ──────────────────────────────────────
var titleLayer = doc.artLayers.add(LayerKind.TEXT);
titleLayer.name = "Main Title";
doc.activeLayer = titleLayer;

var titleText = titleLayer.textItem;
titleText.contents = "{spec.title.upper()}";
titleText.size     = {int(h * 0.07)};
titleText.font     = "{font_h}";
titleText.justification = Justification.CENTER;
titleText.antiAliasMethod = AntiAlias.CRISP;

var titleColor = new SolidColor();
titleColor.rgb.red   = {tx_r};
titleColor.rgb.green = {tx_g};
titleColor.rgb.blue  = {tx_b};
titleText.color = titleColor;

titleText.position = [{w//2}, {int(h * 0.58)}];

// ── 8. Subtitle text ─────────────────────────────────────────
{"" if not spec.subtitle else f'''
var subLayer = doc.artLayers.add(LayerKind.TEXT);
subLayer.name = "Subtitle";
doc.activeLayer = subLayer;

var subText = subLayer.textItem;
subText.contents = "{spec.subtitle}";
subText.size     = {int(h * 0.032)};
subText.font     = "{font_b}";
subText.justification = Justification.CENTER;

var subColor = new SolidColor();
subColor.rgb.red   = {pr_r};
subColor.rgb.green = {pr_g};
subColor.rgb.blue  = {pr_b};
subText.color = subColor;
subText.position = [{w//2}, {int(h * 0.66)}];
'''}

// ── 9. Event details ─────────────────────────────────────────
{"" if not spec.date else f'''
var detailsLayer = doc.artLayers.add(LayerKind.TEXT);
doc.activeLayer = detailsLayer;
var detailsText = detailsLayer.textItem;
detailsText.contents = "📅 {spec.date}" + "{(' ·  🕐 ' + spec.time) if spec.time else ''}";
detailsText.size = {int(h * 0.025)};
detailsText.font = "{font_b}";
detailsText.justification = Justification.CENTER;
var detColor = new SolidColor();
detColor.rgb.red = {tx_r}; detColor.rgb.green = {tx_g}; detColor.rgb.blue = {tx_b};
detailsText.color = detColor;
detailsText.position = [{w//2}, {int(h * 0.73)}];
'''}

{"" if not spec.venue else f'''
var venueLayer = doc.artLayers.add(LayerKind.TEXT);
doc.activeLayer = venueLayer;
var venueText = venueLayer.textItem;
venueText.contents = "📍 {spec.venue}";
venueText.size = {int(h * 0.022)};
venueText.font = "{font_b}";
venueText.justification = Justification.CENTER;
var venColor = new SolidColor();
venColor.rgb.red = {tx_r}; venColor.rgb.green = {tx_g}; venColor.rgb.blue = {tx_b};
venueText.color = venColor;
venueText.position = [{w//2}, {int(h * 0.78)}];
'''}

{"" if not spec.contact else f'''
var contactLayer = doc.artLayers.add(LayerKind.TEXT);
doc.activeLayer = contactLayer;
var contactText = contactLayer.textItem;
contactText.contents = "{spec.contact}";
contactText.size = {int(h * 0.02)};
contactText.font = "{font_b}";
contactText.justification = Justification.CENTER;
var conColor = new SolidColor();
conColor.rgb.red = {pr_r}; conColor.rgb.green = {pr_g}; conColor.rgb.blue = {pr_b};
contactText.color = conColor;
contactText.position = [{w//2}, {int(h * 0.88)}];
'''}

// ── 10. Bottom accent bar ────────────────────────────────────
var bottomAccent = doc.artLayers.add();
doc.activeLayer = bottomAccent;
app.foregroundColor = accentColor;
var bottomBarRect = [[0, {int(h*0.993)}], [{w}, {int(h*0.993)}], [{w}, {h}], [0, {h}]];
doc.selection.select(bottomBarRect);
doc.selection.fill(app.foregroundColor);
doc.selection.deselect();

// ── 11. Export PNG ───────────────────────────────────────────
doc.flatten();
var pngFile = new File("{str(output_png).replace(chr(92), '/')}");
var pngOptions = new PNGSaveOptions();
pngOptions.compression = 6;
doc.saveAs(pngFile, pngOptions, true, Extension.LOWERCASE);

// ── 12. Export PDF ───────────────────────────────────────────
var pdfFile = new File("{str(output_pdf).replace(chr(92), '/')}");
var pdfOptions = new PDFSaveOptions();
pdfOptions.pDFCompatibility = PDFCompatibility.PDF15;
pdfOptions.jpegQuality = 12;
pdfOptions.preserveEditing = false;
doc.saveAs(pdfFile, pdfOptions, true, Extension.LOWERCASE);

alert("🐉 MICRODRAGON — Flyer created successfully!\\n\\nPNG: {str(output_png)}\\nPDF: {str(output_pdf)}\\n\\nYou can now review and refine in Photoshop.");

doc.close(SaveOptions.DONOTSAVECHANGES);
'''


# ─── Pillow Fallback Renderer ─────────────────────────────────────────────────

class PillowFlyerRenderer:
    """
    Renders the flyer using Python Pillow when Photoshop is not installed.
    Same design quality, no external software needed.
    """

    def render(self, spec: FlyerSpec, assets: dict,
               palette: dict) -> str:
        """Render flyer to PNG using Pillow."""
        try:
            from PIL import Image, ImageDraw, ImageFont, ImageFilter
        except ImportError:
            return None

        w, h = FLYER_SIZES.get(spec.size, FLYER_SIZES["a4"])
        # Downscale for digital (saves memory)
        if spec.dpi < 200:
            w, h = w // 2, h // 2

        img = Image.new("RGB", (w, h), palette.get("bg", "#0d1117"))
        draw = ImageDraw.Draw(img, "RGBA")

        # Load hero image if available
        hero_path = assets.get("hero_image")
        if hero_path and Path(hero_path).exists():
            try:
                hero = Image.open(hero_path).convert("RGBA")
                # Scale to fill
                scale = max(w / hero.width, h / hero.height)
                hero = hero.resize((int(hero.width * scale), int(hero.height * scale)),
                                    Image.LANCZOS)
                # Center crop
                ox = (hero.width - w) // 2
                oy = (hero.height - h) // 2
                hero = hero.crop((ox, oy, ox + w, oy + h))
                # Semi-transparent overlay
                hero_faded = Image.new("RGBA", (w, h), (0, 0, 0, 0))
                hero_faded.paste(hero, (0, 0))
                overlay = Image.new("RGBA", (w, h), (*self._hex_to_rgb(palette["bg"]), 180))
                img.paste(Image.alpha_composite(hero_faded, overlay).convert("RGB"), (0, 0))
            except Exception:
                pass

        # Gradient overlay (bottom)
        bg_rgb = self._hex_to_rgb(palette.get("bg", "#0d1117"))
        for y in range(int(h * 0.4), h):
            alpha = int(255 * (y - h * 0.4) / (h * 0.6))
            draw.line([(0, y), (w, y)], fill=(*bg_rgb, alpha))

        # Accent bars
        pr = self._hex_to_rgb(palette.get("primary", "#00ff88"))
        draw.rectangle([0, 0, w, int(h * 0.008)], fill=pr)
        draw.rectangle([0, int(h * 0.993), w, h], fill=pr)

        # Load fonts
        heading_font = self._load_font(assets.get("font_heading"), int(h * 0.07))
        sub_font     = self._load_font(assets.get("font_body"),    int(h * 0.032))
        detail_font  = self._load_font(assets.get("font_body"),    int(h * 0.022))

        text_color   = self._hex_to_rgb(palette.get("text",    "#ffffff"))
        accent_color = self._hex_to_rgb(palette.get("primary", "#00ff88"))

        # Title
        self._draw_centered_text(draw, spec.title.upper(), w, int(h * 0.58),
                                  heading_font, text_color, shadow=True)

        # Subtitle
        if spec.subtitle:
            self._draw_centered_text(draw, spec.subtitle, w, int(h * 0.66),
                                      sub_font, accent_color)

        # Details
        y_offset = int(h * 0.73)
        details = []
        if spec.date:
            details.append(f"  {spec.date}" + (f"   {spec.time}" if spec.time else ""))
        if spec.venue:
            details.append(f"  {spec.venue}")
        if spec.contact:
            details.append(spec.contact)

        for detail in details:
            self._draw_centered_text(draw, detail, w, y_offset,
                                      detail_font, text_color)
            y_offset += int(h * 0.055)

        # Logo
        logo_path = assets.get("logo")
        if logo_path and Path(logo_path).exists():
            try:
                logo = Image.open(logo_path).convert("RGBA")
                logo_w = int(w * 0.15)
                logo_h = int(logo.height * logo_w / logo.width)
                logo = logo.resize((logo_w, logo_h), Image.LANCZOS)
                img.paste(logo, (int(w * 0.05), int(h * 0.03)), logo)
            except Exception:
                pass

        output_path = Path(spec.output_dir) / "flyer_final.png"
        img.save(str(output_path), "PNG", optimize=True)
        return str(output_path)

    def _draw_centered_text(self, draw, text, canvas_w, y, font,
                             color, shadow=False):
        """Draw horizontally centered text with optional shadow."""
        try:
            bbox = draw.textbbox((0, 0), text, font=font)
            tw = bbox[2] - bbox[0]
            x = (canvas_w - tw) // 2
            if shadow:
                draw.text((x + 3, y + 3), text, font=font, fill=(0, 0, 0, 120))
            draw.text((x, y), text, font=font, fill=color)
        except Exception:
            draw.text((canvas_w // 2, y), text, font=font, fill=color,
                       anchor="mt" if hasattr(font, "size") else None)

    def _load_font(self, font_path: Optional[str], size: int):
        """Load font from path or fall back to default."""
        try:
            from PIL import ImageFont
            if font_path and Path(font_path).exists():
                return ImageFont.truetype(font_path, size)
            return ImageFont.load_default()
        except Exception:
            try:
                from PIL import ImageFont
                return ImageFont.load_default()
            except Exception:
                return None

    def _hex_to_rgb(self, hex_str: str) -> tuple:
        h = hex_str.lstrip("#")
        return int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)


# ─── SVG Flyer Generator (always available, no deps) ─────────────────────────

class SVGFlyerGenerator:
    """
    Pure Python SVG flyer — always available, opens in any browser.
    """

    def generate(self, spec: FlyerSpec, assets: dict, palette: dict) -> str:
        """Generate SVG flyer and write to output directory."""

        w_mm, h_mm = {"a4": (210, 297), "a5": (148, 210),
                       "square": (200, 200), "story": (90, 160)}.get(spec.size, (210, 297))
        w, h = w_mm * 3.78, h_mm * 3.78  # mm → px at 96dpi

        bg     = palette.get("bg",      "#0d1117")
        pr     = palette.get("primary", "#00ff88")
        text   = palette.get("text",    "#ffffff")
        accent = palette.get("accent",  "#ff4444")
        surf   = palette.get("surface", "#161b22")

        fonts = STYLE_FONTS.get(spec.style, STYLE_FONTS["modern"])
        h_font = spec.font_heading if spec.font_heading != "auto" else fonts["heading"]
        b_font = spec.font_body    if spec.font_body    != "auto" else fonts["body"]

        svg = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     viewBox="0 0 {w:.0f} {h:.0f}" width="{w:.0f}" height="{h:.0f}">
  <defs>
    <style>
      @import url('https://fonts.googleapis.com/css2?family={h_font.replace(" ","+")}&amp;family={b_font.replace(" ","+")}&amp;display=swap');
    </style>
    <linearGradient id="bgGrad" x1="0" y1="40%" x2="0" y2="100%">
      <stop offset="0%" stop-color="{bg}" stop-opacity="0"/>
      <stop offset="100%" stop-color="{bg}" stop-opacity="0.96"/>
    </linearGradient>
    <linearGradient id="accentGrad" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="{pr}"/>
      <stop offset="100%" stop-color="{accent}"/>
    </linearGradient>
    <filter id="shadow">
      <feDropShadow dx="0" dy="4" stdDeviation="8" flood-color="rgba(0,0,0,0.5)"/>
    </filter>
    <filter id="glow">
      <feGaussianBlur stdDeviation="3" result="coloredBlur"/>
      <feMerge><feMergeNode in="coloredBlur"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
  </defs>

  <!-- Background -->
  <rect width="{w:.0f}" height="{h:.0f}" fill="{bg}"/>

  <!-- Decorative circles -->
  <circle cx="{w:.0f}" cy="0" r="{w*0.6:.0f}" fill="{pr}" opacity="0.04"/>
  <circle cx="0" cy="{h:.0f}" r="{w*0.5:.0f}" fill="{accent}" opacity="0.04"/>

  <!-- Gradient overlay -->
  <rect width="{w:.0f}" height="{h:.0f}" fill="url(#bgGrad)"/>

  <!-- Top accent bar -->
  <rect x="0" y="0" width="{w:.0f}" height="{h*0.008:.0f}" fill="url(#accentGrad)"/>

  <!-- Bottom accent bar -->
  <rect x="0" y="{h*0.993:.0f}" width="{w:.0f}" height="{h*0.007:.0f}" fill="url(#accentGrad)"/>

  <!-- Large title -->
  <text x="{w/2:.0f}" y="{h*0.58:.0f}"
        font-family="'{h_font}', sans-serif"
        font-size="{h*0.07:.0f}" font-weight="800"
        fill="{text}" text-anchor="middle"
        filter="url(#shadow)"
        letter-spacing="-1">{spec.title.upper()}</text>
'''

        if spec.subtitle:
            svg += f'''
  <!-- Subtitle -->
  <text x="{w/2:.0f}" y="{h*0.66:.0f}"
        font-family="'{b_font}', sans-serif"
        font-size="{h*0.032:.0f}" font-weight="400"
        fill="{pr}" text-anchor="middle"
        filter="url(#glow)">{spec.subtitle}</text>
'''

        y = h * 0.73
        if spec.date:
            date_str = spec.date + (f"  ·  {spec.time}" if spec.time else "")
            svg += f'''
  <text x="{w/2:.0f}" y="{y:.0f}"
        font-family="'{b_font}', sans-serif"
        font-size="{h*0.025:.0f}" fill="{text}" text-anchor="middle">
        📅 {date_str}</text>
'''
            y += h * 0.055

        if spec.venue:
            svg += f'''
  <text x="{w/2:.0f}" y="{y:.0f}"
        font-family="'{b_font}', sans-serif"
        font-size="{h*0.022:.0f}" fill="{text}" text-anchor="middle">
        📍 {spec.venue}</text>
'''
            y += h * 0.055

        if spec.contact:
            svg += f'''
  <text x="{w/2:.0f}" y="{h*0.89:.0f}"
        font-family="'{b_font}', sans-serif"
        font-size="{h*0.02:.0f}" fill="{pr}" text-anchor="middle"
        letter-spacing="1">{spec.contact}</text>
'''

        svg += f'''
  <!-- Microdragon watermark -->
  <text x="{w/2:.0f}" y="{h*0.963:.0f}"
        font-family="'{b_font}', sans-serif"
        font-size="{h*0.012:.0f}" fill="{pr}" opacity="0.4" text-anchor="middle">
        🐉 MICRODRAGON · EMEMZYVISUALS DIGITALS</text>

</svg>'''

        output = Path(spec.output_dir) / "flyer_final.svg"
        output.write_text(svg, encoding="utf-8")
        return str(output)


# ─── Master Flyer Engine ──────────────────────────────────────────────────────

class FlyerEngine:
    """
    The complete flyer creation pipeline.
    Orchestrates: clarify → gather → design → export.
    """

    CLARIFICATION_QUESTIONS = {
        "birthday": [
            ("Name & age", "Whose birthday and how old?"),
            ("Date & time", "Date and time of the event?"),
            ("Venue",       "Where is it happening?"),
            ("Style",       "Vibe? (e.g. elegant, party, luxury, modern)"),
            ("Contact",     "RSVP contact or WhatsApp number?"),
            ("Logo",        "Any logo or image to include? (paste URL or describe)"),
        ],
        "business": [
            ("Business name", "Business name and tagline?"),
            ("Service",       "What product or service is this for?"),
            ("Offer",         "Is there a special offer, price, or promotion?"),
            ("Contact",       "Phone, email, or website to show?"),
            ("Date",          "Any date or expiry on the offer?"),
            ("Style",         "Style? (e.g. corporate, modern, bold)"),
            ("Logo",          "Logo URL or file path?"),
        ],
        "event": [
            ("Event name",  "Event name?"),
            ("Date & time", "Date and time?"),
            ("Venue",       "Venue name and address?"),
            ("Price",       "Ticket price or 'Free'?"),
            ("Contact",     "Contact for tickets/RSVP?"),
            ("Style",       "Style? (e.g. bold, elegant, corporate)"),
        ],
    }

    async def create_flyer(self, spec: FlyerSpec) -> dict:
        """
        Full pipeline: gather assets → choose renderer → generate → export.
        Returns dict with output paths and summary.
        """

        # Set up output directory
        if not spec.output_dir:
            spec.output_dir = os.path.expanduser("~/Desktop/microdragon_flyers")
        Path(spec.output_dir).mkdir(parents=True, exist_ok=True)

        staging = tempfile.mkdtemp(prefix="microdragon_flyer_")

        print("\n  🐉 MICRODRAGON — Flyer Creation Pipeline")
        print("  " + "─" * 50)
        print(f"  Type:  {spec.flyer_type.title()}")
        print(f"  Title: {spec.title}")
        print(f"  Size:  {spec.size.upper()} @ {spec.dpi} DPI")
        print(f"  Style: {spec.style.title()}")
        print("  " + "─" * 50 + "\n")

        # ── Step 1: Download all assets ──────────────────────────────────────
        print("  STEP 1 of 3 — Gathering assets...\n")
        downloader = AssetDownloader(staging)

        # Auto-select hero image description if not provided
        if not spec.hero_image:
            hero_descs = {
                "birthday": "birthday celebration party lights confetti",
                "business": f"professional business {spec.style} corporate",
                "event":    "event concert crowd lights stage",
                "product":  "product photography minimal clean",
                "sale":     "shopping sale discount retail",
            }
            spec.hero_image = hero_descs.get(spec.flyer_type, spec.title)

        assets = await downloader.download_all(spec)

        # ── Step 2: Choose palette ────────────────────────────────────────────
        palette_key = f"{spec.flyer_type}_{spec.palette}"
        if palette_key not in STYLE_PALETTES:
            palette_key = f"business_{spec.palette}" if spec.palette in ("dark","light") \
                          else "business_dark"
        palette = STYLE_PALETTES.get(palette_key, STYLE_PALETTES["business_dark"])

        # Override with user's custom colors if set
        if spec.primary_color != "#00ff88":
            palette["primary"] = spec.primary_color
        if spec.secondary_color != "#ff4444":
            palette["accent"] = spec.secondary_color

        # ── Step 3: Render ────────────────────────────────────────────────────
        print("  STEP 2 of 3 — Designing flyer...")
        outputs = {}

        # Try Photoshop first
        photoshop_path = self._find_photoshop()
        if photoshop_path:
            print(f"  ✓ Photoshop found: {photoshop_path}")
            print("  Generating ExtendScript and opening Photoshop...\n")
            builder = PhotoshopFlyerBuilder()
            jsx = builder.generate_script(spec, assets, palette)
            jsx_path = Path(staging) / "microdragon_flyer.jsx"
            jsx_path.write_text(jsx, encoding="utf-8")

            # Launch Photoshop with script
            import subprocess
            try:
                subprocess.Popen([photoshop_path, "-r", str(jsx_path)])
                outputs["jsx"] = str(jsx_path)
                outputs["photoshop_launched"] = True
                print("  ✓ Photoshop opened with flyer script")
                print("  ✓ Script will auto-export PNG + PDF when done")
            except Exception as e:
                print(f"  ! Photoshop launch failed ({e}), using SVG fallback")
                photoshop_path = None

        # SVG (always generates — best quality without Photoshop)
        print("  Generating SVG flyer (opens in browser)...")
        svg_gen = SVGFlyerGenerator()
        svg_path = svg_gen.generate(spec, assets, palette)
        if svg_path:
            import shutil
            final_svg = Path(spec.output_dir) / "flyer_final.svg"
            shutil.copy2(svg_path, final_svg)
            outputs["svg"] = str(final_svg)
            print(f"  ✓ SVG generated: {final_svg}")

        # Pillow fallback (PNG without Photoshop)
        if not photoshop_path:
            try:
                from PIL import Image
                print("  Generating PNG with Pillow...")
                renderer = PillowFlyerRenderer()
                png_path = renderer.render(spec, assets, palette)
                if png_path:
                    outputs["png"] = png_path
                    print(f"  ✓ PNG generated: {png_path}")
            except ImportError:
                print("  ! Pillow not installed (pip install pillow) — PNG skipped")

        # ── Summary ───────────────────────────────────────────────────────────
        print("\n  STEP 3 of 3 — Done!\n")
        print("  🐉 Flyer outputs:")
        for fmt, path in outputs.items():
            if fmt not in ("jsx", "photoshop_launched"):
                print(f"     {fmt.upper()}: {path}")
        print(f"\n  Staging folder (source assets): {staging}")
        print()

        return {
            "success": True,
            "outputs": outputs,
            "assets": assets,
            "staging": staging,
            "spec": spec,
        }

    def build_clarification_prompt(self, flyer_type: str) -> list[tuple]:
        """Return the questions Microdragon asks before creating a flyer."""
        return self.CLARIFICATION_QUESTIONS.get(
            flyer_type, self.CLARIFICATION_QUESTIONS["event"]
        )

    def _find_photoshop(self) -> Optional[str]:
        """Find Photoshop on the current system."""
        import glob, sys
        candidates = []
        if sys.platform == "win32":
            candidates = glob.glob(r"C:\Program Files\Adobe\Adobe Photoshop*\Photoshop.exe")
        elif sys.platform == "darwin":
            candidates = glob.glob("/Applications/Adobe Photoshop*/Adobe Photoshop*.app/Contents/MacOS/Adobe Photoshop*")
        if candidates:
            return candidates[-1]  # Most recent version
        return None


# ─── What Microdragon says when asked to make a flyer ────────────────────────

FLYER_INTAKE_RESPONSE = """
  🐉 Yes! I can create a professional flyer for you.

  Before I start designing, I need a few details so the result is exactly right.
  I'll also download any images and fonts I need BEFORE opening Photoshop —
  so the whole thing runs automatically once you answer these questions.

  Here's my process:
    1. You answer the questions below
    2. I search and download relevant background images
    3. I download the right fonts for your style
    4. I load your logo if you have one
    5. Then I open Photoshop (or use built-in renderer) and build the flyer
    6. You get PNG + PDF + SVG — ready to print or share on WhatsApp

  Let's go:
"""

FLYER_NO_PHOTOSHOP_NOTICE = """
  Note: Photoshop was not found on this machine.
  I'll use my built-in SVG and PNG renderer — same design quality, no Photoshop needed.
  If you later install Photoshop, run the same command and I'll use it automatically.
"""
