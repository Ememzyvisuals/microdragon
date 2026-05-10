# Microdragon — How Flyer Creation Works

## The Question

> "Help me create a professional flyer for my birthday / business..."

**Short answer: Yes. Microdragon downloads everything it needs BEFORE automating Photoshop.**

---

## What Happens Step by Step

### Step 1 — Microdragon Asks Questions First

Microdragon never proceeds blindly. When you ask for a flyer, it says:

```
  🐉 I can build that flyer for you.

  Here's exactly what happens:
  1. You answer a few quick questions (takes 2 minutes)
  2. I download background images from the web
  3. I download the fonts for your chosen style
  4. I load your logo or any images you provide
  5. I open Photoshop and place everything automatically
  6. You get: PNG + PDF + SVG

  Let's start:
```

**For a birthday flyer, it asks:**
- Whose birthday and how old?
- Date and time of the celebration?
- Where is the party?
- What vibe? (elegant / party / luxury / modern / neon)
- Any preferred colors or theme?
- RSVP contact?
- Any photo or logo to include?

**For a business flyer:**
- Business name and tagline?
- What are you promoting?
- Price or discount to show?
- Phone, email, website, or social?
- Any expiry date on the offer?
- Style preference?
- Logo URL or file?

---

### Step 2 — Asset Gathering (Before Any Design Starts)

Once you've answered the questions, Microdragon assembles everything:

```
  🐉 MICRODRAGON — Flyer Creation Pipeline
  ──────────────────────────────────────────
  Type:  Birthday
  Title: JOHN'S 30TH BIRTHDAY
  Size:  A4 @ 300 DPI
  Style: Luxury
  ──────────────────────────────────────────

  STEP 1 of 3 — Gathering assets...

  ✓ Downloaded: hero_image.jpg (birthday celebration lights confetti)
  ✓ Font loaded from system: Cormorant Garamond
  ✓ Downloaded: font_body.ttf (Raleway from Google Fonts)
  ✓ Loaded uploaded image: john_photo.jpg
  ✓ Downloaded: texture.png (luxury texture)

  ✓ Asset assembly complete (5 assets ready)
  📁 Staging folder: /tmp/microdragon_flyer_abc123
```

**What it downloads:**
- Background/hero image — searched on Unsplash by keyword, downloaded automatically
- Fonts — downloaded from Google Fonts CDN, or found in your system fonts
- Logo — downloaded from URL you provide, or loaded from file
- Textures — for styles like luxury, elegant, party
- Any image you describe or upload

---

### Step 3 — Design Generation

Microdragon checks if Photoshop is installed:

**Photoshop found:**
```
  ✓ Photoshop found: /Applications/Adobe Photoshop 2025/...
  Generating ExtendScript and opening Photoshop...
  ✓ Photoshop opened with flyer script
  ✓ Script will auto-export PNG + PDF when done
```

Microdragon generates a complete ExtendScript JSX file that:
1. Creates a new Photoshop document at the correct size and DPI
2. Places the background image
3. Adds gradient overlays for text readability
4. Places your logo in the correct position
5. Adds all text layers with the right fonts, sizes, and colors
6. Applies accent bars and effects
7. Auto-exports PNG + PDF to your Desktop

**Photoshop not installed:**
```
  Photoshop not found on this machine.
  I'll use my built-in SVG and PNG renderer — same design quality.
  Generating SVG flyer (opens in browser)...
  ✓ SVG generated: ~/Desktop/microdragon_flyers/flyer_final.svg
  Generating PNG with Pillow...
  ✓ PNG generated: ~/Desktop/microdragon_flyers/flyer_final.png
```

The built-in renderer uses Python Pillow + SVG generation to produce the same result without Photoshop.

---

### Step 4 — Output

You get three files, always:

| Format | Use case |
|---|---|
| `flyer_final.png` | Share on WhatsApp, Instagram, email |
| `flyer_final.pdf` | Print at any print shop (full bleed, 300 DPI) |
| `flyer_final.svg` | Open in Illustrator, Inkscape, or Figma to edit |

If Photoshop was used, you also get:
- `microdragon_flyer.jsx` — the ExtendScript used (rerun or modify manually)

---

### Step 5 — Iterate

After seeing the result:

```
you → Change the background to dark blue and make the title font bigger

  🐉 Updating flyer...
  ✓ Background color changed to #001a4d
  ✓ Title font size increased from 200pt to 240pt
  ✓ Regenerated: flyer_final.png
```

---

## The Design System

Microdragon's flyer designs are built on a proper design system:

**Typography by style:**
| Style | Heading Font | Body Font |
|---|---|---|
| Modern | Inter | Inter |
| Elegant | Playfair Display | Lato |
| Bold | Oswald | Roboto |
| Playful | Fredoka One | Nunito |
| Corporate | Montserrat | Open Sans |
| Luxury | Cormorant Garamond | Raleway |
| Party | Bebas Neue | Poppins |

**Color palettes:**
| Theme | Background | Primary | Accent |
|---|---|---|---|
| Birthday Classic | #1a0533 | #ff69b4 | #ffd700 |
| Birthday Modern | #0a0a1a | #00ff88 | #ff4488 |
| Business Dark | #0d1117 | #00ff88 | #4488ff |
| Business Light | #ffffff | #1a56db | #0ea5e9 |
| Luxury Gold | #0a0700 | #d4af37 | #f5e9c3 |
| Event Bold | #ff2d00 | #ffffff | #ffd700 |

---

## Commands

```bash
# Natural language (Simple Mode)
microdragon
> Create a birthday flyer for my daughter's 18th

# Pro Mode direct command
microdragon design flyer --type birthday --title "Sofia's 18th" \
  --date "May 15, 2026" --venue "The Grand Ballroom, Lagos" \
  --style elegant --output ~/Desktop/

# With your own image
microdragon design flyer --type business \
  --title "Grand Opening" \
  --image ./my_product_photo.jpg \
  --logo ./my_logo.png \
  --style corporate

# Specify output size
microdragon design flyer --type event --size story   # Instagram/WhatsApp story
microdragon design flyer --type event --size a4      # Print quality
microdragon design flyer --type event --size square  # Social media

# Ask Microdragon what it needs
microdragon design flyer --help
```

---

## Key Point

**Microdragon downloads everything BEFORE opening Photoshop.**

It doesn't open Photoshop and then try to find images.
It assembles the complete asset package first, then runs the design in one pass.
This means Photoshop only needs to open once, and the entire flyer is built in under 60 seconds.

---

*© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo*
