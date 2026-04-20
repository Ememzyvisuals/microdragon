"""
microdragon/modules/web_design/src/engine.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON WEB DESIGN EXPERT MODULE
═══════════════════════════════════════════════════════════════════════════════

Microdragon is a complete web design expert:
  - UI/UX principles (contrast, hierarchy, spacing, typography)
  - Modern CSS (Grid, Flexbox, Container Queries, CSS Variables)
  - Design systems (tokens, components, variants)
  - Responsive design (mobile-first, breakpoints)
  - Accessibility (WCAG 2.2, ARIA, color contrast)
  - Performance (CLS, LCP, FID, code splitting)
  - Frameworks (React, Vue, Svelte, Next.js, Nuxt)
  - Design tools knowledge (Figma, Framer, Webflow)
  - Animation (Framer Motion, GSAP, CSS transitions)
  - Brand identity and visual design

Generates production-ready code:
  - Responsive HTML/CSS/JS pages
  - React/Vue components with TypeScript
  - Tailwind CSS layouts
  - Complete design systems with CSS variables
  - Animated landing pages with glassmorphism
  - Dark/light mode themes

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

import os
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional


@dataclass
class DesignSpec:
    page_type:  str              # landing | dashboard | portfolio | saas | ecommerce
    style:      str = "modern"   # modern | minimal | bold | glassmorphism | corporate
    palette:    str = "dark"     # dark | light | auto | custom
    primary:    str = "#00ff88"  # primary accent
    secondary:  str = "#ff4444"  # secondary accent
    font:       str = "Inter"    # Inter | Geist | Plus Jakarta Sans
    features:   list = field(default_factory=list)  # hero, nav, pricing, faq, cta
    brand_name: str = ""
    tagline:    str = ""
    responsive: bool = True
    animations: bool = True
    dark_mode:  bool = True


# ─── Web Design Knowledge Base ────────────────────────────────────────────────

class WebDesignKnowledge:
    """
    Complete web design knowledge — principles, patterns, code recipes.
    This is what makes Microdragon a web design EXPERT, not just a code generator.
    """

    TYPOGRAPHY = {
        "hierarchy_rule": "Title (40-72px) → Heading (24-32px) → Body (16-18px) → Caption (12-14px)",
        "line_height": "Body: 1.5-1.6. Headings: 1.1-1.2. Display: 0.9-1.0",
        "letter_spacing": "Body: 0. Headings: -0.02em. All-caps labels: 0.1em",
        "font_weights": "Regular (400) body. SemiBold (600) headings. Bold (700) CTAs",
        "font_stacks": {
            "modern": "Inter, 'SF Pro', system-ui, -apple-system, sans-serif",
            "display": "'Plus Jakarta Sans', 'Bricolage Grotesque', sans-serif",
            "mono":    "'JetBrains Mono', 'Fira Code', monospace",
            "editorial": "'Playfair Display', Georgia, serif",
        },
        "fluid_typography": "clamp(1rem, 2.5vw, 1.5rem) — scales smoothly between breakpoints",
    }

    SPACING = {
        "scale": "4px base unit. Scale: 4, 8, 12, 16, 24, 32, 48, 64, 96, 128",
        "padding_card": "24px all sides. Large cards: 32-40px",
        "section_gap": "80-120px between major sections",
        "component_gap": "8-16px between related elements",
        "layout_margin": "max(24px, 8vw) for responsive side margins",
    }

    COLOR = {
        "contrast_aa": "4.5:1 minimum for body text. 3:1 for large text (18px+)",
        "contrast_aaa": "7:1 for body text (accessible for visually impaired)",
        "dark_bg_recipe": "bg: #0d1117 · surface: #161b22 · border: #30363d · text: #e6edf3",
        "light_bg_recipe": "bg: #ffffff · surface: #f6f8fa · border: #d0d7de · text: #1f2328",
        "semantic_colors": {
            "success": "#00ff88 or #22c55e",
            "error":   "#ff4444 or #ef4444",
            "warning": "#ff8800 or #f97316",
            "info":    "#4488ff or #3b82f6",
        },
        "glassmorphism": "background: rgba(255,255,255,0.05); backdrop-filter: blur(10px); border: 1px solid rgba(255,255,255,0.1)",
    }

    LAYOUT = {
        "grid": "CSS Grid for 2D layouts. 12-column: grid-template-columns: repeat(12, 1fr)",
        "flexbox": "1D layouts. justify-content + align-items for centering",
        "container": "max-width: 1280px; margin: 0 auto; padding: 0 max(24px, 8vw)",
        "breakpoints": {
            "xs":  "< 480px  — single column, larger tap targets",
            "sm":  "480-768px — 2 columns, mobile-first",
            "md":  "768-1024px — tablet, 3-4 columns",
            "lg":  "1024-1280px — desktop, full layout",
            "xl":  "> 1280px — wide screens, constrained max-width",
        },
        "card_grid": "grid-template-columns: repeat(auto-fill, minmax(300px, 1fr))",
    }

    ANIMATION = {
        "durations": "micro: 100ms · standard: 200ms · complex: 400ms · page: 600ms",
        "easing": {
            "default":  "cubic-bezier(0.4, 0, 0.2, 1) — Material standard",
            "enter":    "cubic-bezier(0, 0, 0.2, 1) — decelerate",
            "exit":     "cubic-bezier(0.4, 0, 1, 1) — accelerate",
            "bounce":   "cubic-bezier(0.34, 1.56, 0.64, 1)",
        },
        "entrance": "@keyframes fadeUp { from { opacity:0; transform:translateY(20px) } }",
        "hover": "transform: translateY(-2px); box-shadow: 0 8px 30px rgba(0,0,0,0.3)",
        "scroll_trigger": "IntersectionObserver API — trigger animations on scroll",
    }

    PERFORMANCE = {
        "core_web_vitals": {
            "LCP": "Largest Contentful Paint < 2.5s — optimize hero images",
            "FID": "First Input Delay < 100ms — minimize main thread blocking",
            "CLS": "Cumulative Layout Shift < 0.1 — set width/height on images",
        },
        "image_optimization": "Use WebP/AVIF. Lazy load below fold. Responsive srcset",
        "css_loading": "Critical CSS inline. Non-critical via <link rel=preload>",
        "font_loading": "font-display: swap. Preconnect to Google Fonts",
        "bundle_size": "Code split by route. Tree shake unused CSS. Use modern JS",
    }

    ACCESSIBILITY = {
        "semantic_html": "Use nav, main, article, section, aside, header, footer",
        "aria": "aria-label, aria-describedby, role for custom components",
        "focus": "Visible focus ring: outline: 2px solid #00ff88; outline-offset: 2px",
        "skip_link": '<a href="#main" class="skip-link">Skip to content</a>',
        "color_alone": "Never convey info by color alone — use icons + text too",
        "touch_targets": "Minimum 44x44px tap targets for mobile (WCAG 2.5.5)",
    }

    PATTERNS = {
        "hero": "Full-width, single clear CTA, value proposition in < 8 words, social proof below",
        "pricing": "3-column (most popular highlighted), annual/monthly toggle, feature comparison",
        "navigation": "Logo left, links center, CTA right. Mobile: hamburger menu",
        "card": "Consistent padding, clear hierarchy, single action per card",
        "form": "Labels above inputs, inline validation, clear error states, progress for multi-step",
        "cta": "High contrast, action verb, benefit-focused copy, urgency without deception",
    }


# ─── Landing Page Generator ────────────────────────────────────────────────────

class LandingPageGenerator:
    """Generates production-ready landing pages with modern design."""

    def generate_saas_landing(self, spec: DesignSpec) -> str:
        """Generate a complete SaaS landing page in HTML/CSS/JS."""

        bg = "#0d1117" if spec.palette == "dark" else "#ffffff"
        surface = "#161b22" if spec.palette == "dark" else "#f6f8fa"
        text = "#e6edf3" if spec.palette == "dark" else "#1f2328"
        text_dim = "#8b949e" if spec.palette == "dark" else "#656d76"
        border = "#30363d" if spec.palette == "dark" else "#d0d7de"
        primary = spec.primary
        secondary = spec.secondary
        brand = spec.brand_name or "Product"
        tagline = spec.tagline or "The future is here"

        return f'''<!DOCTYPE html>
<html lang="en" data-theme="{spec.palette}">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="description" content="{tagline}">
  <title>{brand}</title>

  <!-- Performance: preconnect to font origin -->
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&display=swap" rel="stylesheet">

  <style>
    /* ── Design Tokens ── */
    :root {{
      --bg:          {bg};
      --surface:     {surface};
      --surface-2:   {surface}cc;
      --border:      {border};
      --text:        {text};
      --text-dim:    {text_dim};
      --primary:     {primary};
      --secondary:   {secondary};
      --primary-15:  {primary}26;
      --primary-30:  {primary}4d;
      --radius:      12px;
      --radius-lg:   20px;
      --shadow:      0 4px 24px rgba(0,0,0,0.3);
      --shadow-lg:   0 12px 48px rgba(0,0,0,0.4);
      --font:        'Inter', system-ui, -apple-system, sans-serif;
      --transition:  200ms cubic-bezier(0.4, 0, 0.2, 1);
    }}

    /* ── Reset ── */
    *, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}
    html {{ scroll-behavior: smooth; }}
    body {{
      font-family: var(--font);
      background: var(--bg);
      color: var(--text);
      line-height: 1.6;
      -webkit-font-smoothing: antialiased;
    }}
    img, video {{ max-width: 100%; display: block; }}
    a {{ color: inherit; text-decoration: none; }}

    /* ── Layout ── */
    .container {{
      max-width: 1200px;
      margin: 0 auto;
      padding: 0 max(24px, 6vw);
    }}

    /* ── Navigation ── */
    nav {{
      position: sticky; top: 0; z-index: 100;
      background: color-mix(in srgb, var(--bg) 80%, transparent);
      backdrop-filter: blur(20px) saturate(180%);
      -webkit-backdrop-filter: blur(20px) saturate(180%);
      border-bottom: 1px solid var(--border);
    }}
    .nav-inner {{
      display: flex; align-items: center; justify-content: space-between;
      height: 64px;
    }}
    .logo {{
      font-size: 20px; font-weight: 700; color: var(--primary);
      letter-spacing: -0.5px;
    }}
    .logo span {{ color: var(--text); }}
    .nav-links {{
      display: flex; gap: 32px; list-style: none;
    }}
    .nav-links a {{
      font-size: 14px; font-weight: 500; color: var(--text-dim);
      transition: color var(--transition);
    }}
    .nav-links a:hover {{ color: var(--text); }}
    .nav-cta {{
      background: var(--primary); color: #000;
      padding: 8px 20px; border-radius: 8px;
      font-size: 14px; font-weight: 600;
      transition: all var(--transition);
    }}
    .nav-cta:hover {{ transform: translateY(-1px); box-shadow: 0 4px 16px {primary}40; }}

    /* ── Hero ── */
    .hero {{
      padding: 100px 0 80px;
      text-align: center;
      position: relative;
      overflow: hidden;
    }}
    .hero::before {{
      content: '';
      position: absolute; inset: 0;
      background: radial-gradient(ellipse 60% 50% at 50% 0%,
        {primary}18 0%, transparent 60%);
      pointer-events: none;
    }}
    .hero-badge {{
      display: inline-flex; align-items: center; gap: 6px;
      background: var(--primary-15); border: 1px solid var(--primary-30);
      color: var(--primary); padding: 4px 12px; border-radius: 999px;
      font-size: 13px; font-weight: 500; margin-bottom: 24px;
    }}
    .hero h1 {{
      font-size: clamp(2.5rem, 6vw, 4.5rem);
      font-weight: 800; letter-spacing: -0.04em; line-height: 1.05;
      margin-bottom: 24px;
    }}
    .hero h1 .accent {{ color: var(--primary); }}
    .hero .subtitle {{
      font-size: clamp(1rem, 2vw, 1.25rem);
      color: var(--text-dim); max-width: 560px;
      margin: 0 auto 40px; line-height: 1.6;
    }}
    .hero-actions {{
      display: flex; gap: 12px; justify-content: center; flex-wrap: wrap;
    }}
    .btn-primary {{
      background: var(--primary); color: #000;
      padding: 14px 28px; border-radius: var(--radius);
      font-size: 15px; font-weight: 600; border: none; cursor: pointer;
      transition: all var(--transition);
    }}
    .btn-primary:hover {{
      transform: translateY(-2px);
      box-shadow: 0 8px 30px {primary}40;
    }}
    .btn-secondary {{
      background: transparent; color: var(--text);
      padding: 14px 28px; border-radius: var(--radius);
      font-size: 15px; font-weight: 500;
      border: 1px solid var(--border); cursor: pointer;
      transition: all var(--transition);
    }}
    .btn-secondary:hover {{ border-color: var(--primary); color: var(--primary); }}

    /* Social proof */
    .social-proof {{
      margin-top: 64px; color: var(--text-dim); font-size: 13px;
    }}
    .proof-avatars {{
      display: flex; justify-content: center; margin-bottom: 8px;
    }}
    .proof-avatars .avatar {{
      width: 32px; height: 32px; border-radius: 50%;
      background: var(--primary-15); border: 2px solid var(--bg);
      margin-left: -8px; display: flex; align-items: center;
      justify-content: center; font-size: 14px;
    }}

    /* ── Features ── */
    .section {{ padding: 100px 0; }}
    .section-label {{
      font-size: 13px; font-weight: 600; letter-spacing: 0.08em;
      color: var(--primary); text-transform: uppercase; margin-bottom: 12px;
    }}
    .section-title {{
      font-size: clamp(1.75rem, 4vw, 2.75rem);
      font-weight: 700; letter-spacing: -0.03em; line-height: 1.2;
      margin-bottom: 16px;
    }}
    .section-subtitle {{
      font-size: 1.1rem; color: var(--text-dim);
      max-width: 520px; line-height: 1.6; margin-bottom: 60px;
    }}
    .features-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
      gap: 24px;
    }}
    .feature-card {{
      background: var(--surface); border: 1px solid var(--border);
      border-radius: var(--radius-lg); padding: 32px;
      transition: all var(--transition);
      position: relative; overflow: hidden;
    }}
    .feature-card::before {{
      content: '';
      position: absolute; top: 0; left: 0; right: 0; height: 1px;
      background: linear-gradient(90deg, transparent, var(--primary), transparent);
      opacity: 0; transition: opacity var(--transition);
    }}
    .feature-card:hover {{
      border-color: var(--primary-30);
      transform: translateY(-4px);
      box-shadow: var(--shadow-lg);
    }}
    .feature-card:hover::before {{ opacity: 1; }}
    .feature-icon {{
      width: 48px; height: 48px; border-radius: 12px;
      background: var(--primary-15); border: 1px solid var(--primary-30);
      display: flex; align-items: center; justify-content: center;
      font-size: 22px; margin-bottom: 20px;
    }}
    .feature-card h3 {{
      font-size: 18px; font-weight: 600; margin-bottom: 8px;
    }}
    .feature-card p {{ font-size: 15px; color: var(--text-dim); line-height: 1.6; }}

    /* ── Pricing ── */
    .pricing-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 24px; max-width: 960px; margin: 0 auto;
    }}
    .pricing-card {{
      background: var(--surface); border: 1px solid var(--border);
      border-radius: var(--radius-lg); padding: 36px;
      transition: all var(--transition);
    }}
    .pricing-card.popular {{
      border-color: var(--primary);
      background: color-mix(in srgb, var(--surface) 100%, {primary}08);
      position: relative;
    }}
    .popular-badge {{
      position: absolute; top: -12px; left: 50%; transform: translateX(-50%);
      background: var(--primary); color: #000;
      font-size: 11px; font-weight: 700; letter-spacing: 0.05em;
      padding: 4px 12px; border-radius: 999px; text-transform: uppercase;
    }}
    .price {{ font-size: 40px; font-weight: 800; margin: 16px 0 8px; }}
    .price span {{ font-size: 18px; font-weight: 400; color: var(--text-dim); }}
    .price-features {{ list-style: none; margin: 24px 0; }}
    .price-features li {{
      padding: 8px 0; border-bottom: 1px solid var(--border);
      font-size: 14px; display: flex; align-items: center; gap: 8px;
    }}
    .price-features li::before {{ content: '✓'; color: var(--primary); font-weight: 600; }}

    /* ── CTA Section ── */
    .cta-section {{
      padding: 100px 0; text-align: center;
      background: linear-gradient(180deg, transparent, var(--primary-15), transparent);
    }}
    .cta-section h2 {{
      font-size: clamp(2rem, 4vw, 3rem);
      font-weight: 700; letter-spacing: -0.03em; margin-bottom: 16px;
    }}
    .cta-section p {{
      color: var(--text-dim); font-size: 1.1rem; margin-bottom: 36px;
    }}

    /* ── Footer ── */
    footer {{
      border-top: 1px solid var(--border);
      padding: 48px 0 32px;
    }}
    .footer-grid {{
      display: grid; grid-template-columns: 2fr 1fr 1fr 1fr; gap: 48px;
      margin-bottom: 48px;
    }}
    .footer-brand p {{
      color: var(--text-dim); font-size: 14px; margin-top: 8px; max-width: 280px;
    }}
    .footer-col h4 {{
      font-size: 13px; font-weight: 600; letter-spacing: 0.05em;
      text-transform: uppercase; margin-bottom: 16px; color: var(--text-dim);
    }}
    .footer-col ul {{ list-style: none; }}
    .footer-col ul li {{ margin-bottom: 10px; }}
    .footer-col ul li a {{
      color: var(--text-dim); font-size: 14px; transition: color var(--transition);
    }}
    .footer-col ul li a:hover {{ color: var(--text); }}
    .footer-bottom {{
      display: flex; justify-content: space-between; align-items: center;
      font-size: 13px; color: var(--text-dim);
      border-top: 1px solid var(--border); padding-top: 24px;
    }}

    /* ── Animations ── */
    @keyframes fadeUp {{
      from {{ opacity: 0; transform: translateY(24px); }}
      to   {{ opacity: 1; transform: translateY(0); }}
    }}
    @keyframes glow {{
      0%, 100% {{ box-shadow: 0 0 20px {primary}20; }}
      50%       {{ box-shadow: 0 0 40px {primary}40; }}
    }}
    .animate-in {{ animation: fadeUp 0.6s cubic-bezier(0,0,0.2,1) both; }}
    .delay-1 {{ animation-delay: 0.1s; }}
    .delay-2 {{ animation-delay: 0.2s; }}
    .delay-3 {{ animation-delay: 0.3s; }}

    /* ── Responsive ── */
    @media (max-width: 768px) {{
      .nav-links {{ display: none; }}
      .features-grid {{ grid-template-columns: 1fr; }}
      .footer-grid {{ grid-template-columns: 1fr 1fr; }}
      .footer-bottom {{ flex-direction: column; gap: 12px; text-align: center; }}
    }}
    @media (max-width: 480px) {{
      .hero-actions {{ flex-direction: column; align-items: stretch; }}
      .hero-actions button {{ width: 100%; }}
      .footer-grid {{ grid-template-columns: 1fr; }}
    }}
  </style>
</head>
<body>

  <!-- Navigation -->
  <nav role="navigation">
    <div class="container nav-inner">
      <a href="/" class="logo">🐉{brand}</a>
      <ul class="nav-links" role="list">
        <li><a href="#features">Features</a></li>
        <li><a href="#pricing">Pricing</a></li>
        <li><a href="#about">About</a></li>
        <li><a href="#docs">Docs</a></li>
      </ul>
      <a href="#get-started" class="nav-cta">Get started →</a>
    </div>
  </nav>

  <!-- Hero Section -->
  <main id="main">
    <section class="hero" aria-labelledby="hero-title">
      <div class="container">
        <div class="hero-badge animate-in">✨ Now in public beta</div>
        <h1 id="hero-title" class="animate-in delay-1">
          {tagline.split()[0] if tagline else 'Build'}<br>
          <span class="accent">{' '.join(tagline.split()[1:]) if len(tagline.split()) > 1 else 'anything'}</span>
        </h1>
        <p class="subtitle animate-in delay-2">
          {brand} gives you the tools to move fast, ship confidently, and scale without limits.
        </p>
        <div class="hero-actions animate-in delay-3">
          <button class="btn-primary" onclick="location.href='#get-started'">
            Start for free →
          </button>
          <button class="btn-secondary" onclick="location.href='#features'">
            See how it works
          </button>
        </div>
        <div class="social-proof animate-in delay-3">
          <div class="proof-avatars" aria-hidden="true">
            <div class="avatar">👤</div>
            <div class="avatar">👤</div>
            <div class="avatar">👤</div>
            <div class="avatar">👤</div>
            <div class="avatar">👤</div>
          </div>
          <p>Trusted by <strong>10,000+</strong> developers and teams</p>
        </div>
      </div>
    </section>

    <!-- Features Section -->
    <section class="section" id="features" aria-labelledby="features-title">
      <div class="container">
        <p class="section-label">Features</p>
        <h2 class="section-title" id="features-title">Everything you need to move fast</h2>
        <p class="section-subtitle">
          Built for developers who want to ship without compromising on quality or security.
        </p>
        <div class="features-grid">
          <div class="feature-card js-animate">
            <div class="feature-icon">⚡</div>
            <h3>Lightning fast</h3>
            <p>Built for performance from day one. Every operation is optimized to get out of your way.</p>
          </div>
          <div class="feature-card js-animate">
            <div class="feature-icon">🔒</div>
            <h3>Secure by default</h3>
            <p>End-to-end encryption, zero telemetry, and full data ownership. Your data stays yours.</p>
          </div>
          <div class="feature-card js-animate">
            <div class="feature-icon">🐉</div>
            <h3>AI-powered</h3>
            <p>Intelligent automation that understands your intent and executes with precision.</p>
          </div>
          <div class="feature-card js-animate">
            <div class="feature-icon">🔌</div>
            <h3>Extensible</h3>
            <p>Plugin architecture lets you add capabilities exactly when you need them.</p>
          </div>
          <div class="feature-card js-animate">
            <div class="feature-icon">🌍</div>
            <h3>Works everywhere</h3>
            <p>Windows, macOS, Linux. Local or cloud. Your workflow, not ours.</p>
          </div>
          <div class="feature-card js-animate">
            <div class="feature-icon">📦</div>
            <h3>One install</h3>
            <p>npm install and you are running. No Docker, no setup scripts, no drama.</p>
          </div>
        </div>
      </div>
    </section>

    <!-- Pricing Section -->
    <section class="section" id="pricing" aria-labelledby="pricing-title">
      <div class="container" style="text-align: center;">
        <p class="section-label">Pricing</p>
        <h2 class="section-title" id="pricing-title">Simple, honest pricing</h2>
        <p class="section-subtitle" style="margin: 0 auto 60px;">
          Start free. Scale when you are ready. No surprises on your bill.
        </p>
        <div class="pricing-grid">
          <div class="pricing-card">
            <h3>Starter</h3>
            <div class="price">$0<span>/mo</span></div>
            <p style="color: var(--text-dim); font-size: 14px; margin-bottom: 8px;">For individuals</p>
            <ul class="price-features">
              <li>Local AI (Ollama, free forever)</li>
              <li>All core features</li>
              <li>Community support</li>
              <li>MIT license</li>
            </ul>
            <button class="btn-secondary" style="width:100%">Get started</button>
          </div>
          <div class="pricing-card popular">
            <div class="popular-badge">Most popular</div>
            <h3>Pro</h3>
            <div class="price">$12<span>/mo</span></div>
            <p style="color: var(--text-dim); font-size: 14px; margin-bottom: 8px;">For power users</p>
            <ul class="price-features">
              <li>All Starter features</li>
              <li>Cloud AI (Groq, Anthropic)</li>
              <li>Priority support</li>
              <li>Advanced security tools</li>
              <li>Custom skill marketplace</li>
            </ul>
            <button class="btn-primary" style="width:100%">Start Pro →</button>
          </div>
          <div class="pricing-card">
            <h3>Team</h3>
            <div class="price">$29<span>/mo</span></div>
            <p style="color: var(--text-dim); font-size: 14px; margin-bottom: 8px;">For teams</p>
            <ul class="price-features">
              <li>All Pro features</li>
              <li>Team memory sync</li>
              <li>Shared skill library</li>
              <li>Admin dashboard</li>
              <li>SSO/SAML</li>
            </ul>
            <button class="btn-secondary" style="width:100%">Contact sales</button>
          </div>
        </div>
      </div>
    </section>

    <!-- CTA -->
    <section class="cta-section" id="get-started">
      <div class="container">
        <h2>Ready to breathe fire on your workflow?</h2>
        <p>Install in 2 minutes. Start building immediately.</p>
        <div style="background: var(--surface); border: 1px solid var(--border);
                    border-radius: 10px; padding: 16px 24px; display: inline-block;
                    font-family: 'JetBrains Mono', monospace; font-size: 15px;
                    margin-bottom: 24px;">
          npm install -g @ememzyvisuals/microdragon
        </div>
        <br>
        <button class="btn-primary">Get started for free →</button>
      </div>
    </section>
  </main>

  <!-- Footer -->
  <footer role="contentinfo">
    <div class="container">
      <div class="footer-grid">
        <div class="footer-brand">
          <div class="logo" style="font-size: 24px;">🐉 {brand}</div>
          <p>The AI agent that turns intent into real-world outcomes. Local-first, developer-grade.</p>
        </div>
        <div class="footer-col">
          <h4>Product</h4>
          <ul>
            <li><a href="#features">Features</a></li>
            <li><a href="#pricing">Pricing</a></li>
            <li><a href="#changelog">Changelog</a></li>
            <li><a href="#roadmap">Roadmap</a></li>
          </ul>
        </div>
        <div class="footer-col">
          <h4>Resources</h4>
          <ul>
            <li><a href="#docs">Documentation</a></li>
            <li><a href="#skills">Skills</a></li>
            <li><a href="#github">GitHub</a></li>
            <li><a href="#discord">Discord</a></li>
          </ul>
        </div>
        <div class="footer-col">
          <h4>Company</h4>
          <ul>
            <li><a href="#about">About</a></li>
            <li><a href="https://x.com/ememzyvisuals">X / Twitter</a></li>
            <li><a href="#privacy">Privacy</a></li>
            <li><a href="#terms">Terms</a></li>
          </ul>
        </div>
      </div>
      <div class="footer-bottom">
        <span>© 2026 EMEMZYVISUALS DIGITALS. Emmanuel Ariyo.</span>
        <span>Made with 🐉 and Rust</span>
      </div>
    </div>
  </footer>

  <script>
    // Scroll animation with IntersectionObserver
    const observer = new IntersectionObserver(
      entries => entries.forEach(e => {{
        if (e.isIntersecting) {{
          e.target.classList.add('animate-in');
          observer.unobserve(e.target);
        }}
      }}),
      {{ threshold: 0.1 }}
    );
    document.querySelectorAll('.js-animate').forEach(el => observer.observe(el));
  </script>
</body>
</html>'''

    def get_design_advice(self, question: str) -> str:
        """Answer web design questions with expert knowledge."""
        q = question.lower()
        advice = []

        if "contrast" in q or "color" in q or "accessibility" in q:
            advice.append(WebDesignKnowledge.COLOR["contrast_aa"])
            advice.append(WebDesignKnowledge.ACCESSIBILITY["focus"])
        if "font" in q or "typography" in q or "text" in q:
            advice.append(WebDesignKnowledge.TYPOGRAPHY["hierarchy_rule"])
            advice.append(WebDesignKnowledge.TYPOGRAPHY["line_height"])
        if "spacing" in q or "padding" in q or "margin" in q:
            advice.append(WebDesignKnowledge.SPACING["scale"])
            advice.append(WebDesignKnowledge.SPACING["padding_card"])
        if "animation" in q or "motion" in q:
            advice.append(WebDesignKnowledge.ANIMATION["durations"])
            advice.append(WebDesignKnowledge.ANIMATION["hover"])
        if "performance" in q or "speed" in q or "lcp" in q:
            advice.append(str(WebDesignKnowledge.PERFORMANCE["core_web_vitals"]))
        if "layout" in q or "grid" in q or "flex" in q:
            advice.append(WebDesignKnowledge.LAYOUT["grid"])
            advice.append(WebDesignKnowledge.LAYOUT["container"])
        if "glass" in q or "glassmorphism" in q:
            advice.append(WebDesignKnowledge.COLOR["glassmorphism"])
        if "responsive" in q or "mobile" in q:
            advice.append(str(WebDesignKnowledge.LAYOUT["breakpoints"]))

        if not advice:
            return ("Microdragon knows: typography hierarchy, color contrast (WCAG), "
                    "CSS Grid/Flexbox, responsive design, animations, "
                    "glassmorphism, accessibility, and Core Web Vitals. "
                    "Ask a specific question for expert guidance.")

        return "\n\n".join(advice)
