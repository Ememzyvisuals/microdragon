"""
microdragon/modules/responses/scenarios.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON SCENARIO RESPONSE SYSTEM
═══════════════════════════════════════════════════════════════════════════════

This module defines exactly what Microdragon does and says for every real-world
scenario. Each function returns:
  - What Microdragon says
  - What it actually does (the execution plan)
  - What clarifying questions it asks first

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class ScenarioResponse:
    user_input:   str
    what_it_says: str          # Microdragon's first response
    clarifies:    list[str]    # Questions asked before acting
    executes:     list[str]    # Step-by-step what Microdragon does
    delivers:     list[str]    # What the user gets at the end
    honest_note:  str = ""     # Honest limitation or boundary


# ════════════════════════════════════════════════════════════════════════════
# SCENARIO RESPONSES
# ════════════════════════════════════════════════════════════════════════════

SCENARIOS = {

# ─── 1. E-commerce web app ────────────────────────────────────────────────────

"ecommerce_app": ScenarioResponse(
    user_input="Help me to build a full e-commerce web app with modern technology stacks",
    what_it_says="""
  🐉 Building a full e-commerce app. Let me ask a few things first.
""",
    clarifies=[
        "What are you selling? (physical products / digital / services / marketplace)",
        "Do you want to handle payments yourself or use Stripe/PayPal?",
        "Do you have a brand name, logo, or color scheme?",
        "Tech preference — or should I choose the best stack for you?",
        "Budget for hosting? (free tier / $5-20/month / enterprise)",
    ],
    executes=[
        "Choose optimal stack based on answers:",
        "  Frontend: Next.js 15 + TypeScript + Tailwind CSS + shadcn/ui",
        "  Backend:  FastAPI (Python) or Node.js Hono — user's choice",
        "  Database: PostgreSQL (products, users, orders) + Redis (cart/sessions)",
        "  Payments: Stripe (cards) + Paystack (Nigeria) — both configured",
        "  Auth:     NextAuth.js with email + Google + social login",
        "  Storage:  Cloudflare R2 for product images (free tier generous)",
        "  Deploy:   Vercel (frontend) + Railway (backend) — both have free tiers",
        "",
        "Generate every file in the project:",
        "  ✓ Product catalog with search and filters",
        "  ✓ Shopping cart with persistent sessions",
        "  ✓ Checkout flow with Stripe payment form",
        "  ✓ User accounts, order history, tracking",
        "  ✓ Admin dashboard: add products, manage orders, view analytics",
        "  ✓ Mobile-responsive design",
        "  ✓ SEO meta tags and sitemap",
        "  ✓ Complete README with setup instructions",
        "",
        "Run locally to verify it works",
        "Provide deployment guide step by step",
    ],
    delivers=[
        "Complete project folder ready to deploy",
        "Every line of code written — not a template, a real working app",
        "1-command local setup: `npm install && npm run dev`",
        "Deployment guide for Vercel + Railway (free tier)",
        "Admin dashboard URL and default credentials",
    ],
    honest_note="The app is fully functional. Scaling to production traffic requires paid hosting. Payment processing requires a real Stripe/Paystack account (free to create)."
),

# ─── 2. SaaS for beginners ────────────────────────────────────────────────────

"saas_beginner": ScenarioResponse(
    user_input="I am a beginner and I need you to help me research and build a powerful SaaS that people will pay for",
    what_it_says="""
  🐉 Good goal. Let's do this properly — research first, then build.

  Being a beginner is not a problem. I'll explain every step.
  The goal is a SaaS that solves a real problem people pay for.
  That starts with finding the right idea, not just building something.
""",
    clarifies=[
        "What field or industry do you know something about? (doesn't have to be tech)",
        "What problem frustrates you personally that software could solve?",
        "Do you have any budget for hosting/tools? (even $0 is fine — I'll find free options)",
        "How much time can you invest weekly?",
    ],
    executes=[
        "PHASE 1 — IDEA RESEARCH (before writing any code):",
        "  Search Product Hunt, IndieHackers, Reddit for validated problems",
        "  Cross-reference with keywords people search (pain points with money)",
        "  Find 3-5 SaaS ideas matching your background — sorted by revenue potential",
        "  Analyse competitors: pricing, weaknesses, positioning gaps",
        "  Recommend the #1 idea with clearest path to first $1k MRR",
        "",
        "PHASE 2 — VALIDATE BEFORE BUILDING:",
        "  Write a landing page describing the product (before building it)",
        "  Draft 5 outreach messages to potential customers",
        "  Explain how to get 10 people to say 'I would pay for this'",
        "  Only proceed to building after validation",
        "",
        "PHASE 3 — BUILD MVP:",
        "  Choose the simplest stack that works (not the most impressive)",
        "  Build minimum features that make the core value work",
        "  Set up Stripe billing from day one",
        "  Deploy on free tier (Vercel + Supabase = $0)",
        "  Write every file, explain every decision",
        "",
        "PHASE 4 — LAUNCH:",
        "  ProductHunt launch strategy",
        "  Twitter/X post templates",
        "  Cold outreach templates",
        "  First 10 customer acquisition plan",
    ],
    delivers=[
        "Top 3 validated SaaS ideas ranked by market opportunity",
        "Complete competitor analysis for recommended idea",
        "Landing page (deployed, working) to validate before building",
        "Full MVP codebase when validation is confirmed",
        "Growth playbook: how to get first 10 paying customers",
    ],
    honest_note="I cannot guarantee revenue — the market decides that. But I can give you the best possible technical and strategic foundation. The research phase is not skippable — building before validation is the #1 beginner mistake."
),

# ─── 3. Shopify $10k/week ─────────────────────────────────────────────────────

"shopify_store": ScenarioResponse(
    user_input="Help me to create a Shopify store that will make up to $10k sales per week",
    what_it_says="""
  🐉 I can set up the store. Let's be clear about what I do and what you do.

  I handle: store setup, design, product listings, app configuration,
            SEO, email automation, ad creative copy, conversion optimization.

  You handle: the actual products (I can help find suppliers),
              the ad spend, and the daily marketing work.

  $10k/week is achievable — it requires the right product AND execution.
  Let's start with the product research before touching Shopify.
""",
    clarifies=[
        "Do you already have products, or do you need help finding a winning product?",
        "What niche or category? (or should I research high-margin opportunities?)",
        "Do you have a Shopify account? (14-day free trial at shopify.com)",
        "Ad budget available? (Facebook/TikTok/Google)",
        "Will you dropship (no inventory) or hold stock?",
    ],
    executes=[
        "PRODUCT RESEARCH:",
        "  Analyse trending products on AliExpress, CJDropshipping, Zendrop",
        "  Check Facebook Ad Library for proven winning products",
        "  Calculate margins: sell price - product cost - ads - Shopify fees",
        "  Recommend top 3 products with realistic margin analysis",
        "",
        "STORE BUILD:",
        "  Set up Shopify store with Dawn or Impulse theme (conversion-optimised)",
        "  Configure: currency, shipping zones, tax settings",
        "  Write product descriptions with SEO and conversion copy",
        "  Set up product images (compressed, alt-tagged)",
        "  Configure abandoned cart email (Klaviyo free tier)",
        "  Install essential apps: DSers/AutoDS (dropship), Loox (reviews), SMSBump",
        "  Set up upsell flows (product bundles, post-purchase offers)",
        "  Configure Google Analytics + Facebook Pixel",
        "",
        "MARKETING SETUP:",
        "  Write 5 Facebook/TikTok ad scripts for the hero product",
        "  Create ad creative briefs for video ads",
        "  Set up email welcome sequence (3 emails)",
        "  Write influencer outreach templates",
        "  Create TikTok organic content strategy (free traffic)",
    ],
    delivers=[
        "Complete Shopify store — ready to take orders",
        "Product research report with top 3 picks and margin analysis",
        "5 ad scripts ready for Facebook/TikTok",
        "Email automation sequences set up",
        "Full launch checklist",
        "Week 1 action plan to get first sales",
    ],
    honest_note="I build the best possible store and strategy. Revenue depends on product-market fit and ad execution. $10k/week is a real goal for the right product — I cannot promise it, but I can give you the exact setup that makes it possible."
),

# ─── 4. iOS-styled animation app ─────────────────────────────────────────────

"animation_app": ScenarioResponse(
    user_input="Create a multi-device iOS-styled animation graphics application that helps content creators",
    what_it_says="""
  🐉 Building a cross-platform animation app with iOS design language.
  This is a serious project. Let me get the details right.
""",
    clarifies=[
        "Which devices: iOS only / iOS + Android / iOS + Android + Web?",
        "Core feature: create animations, edit videos, generate graphics, or all three?",
        "Target users: TikTok creators, YouTube, Instagram, or all platforms?",
        "Native app (Swift/Kotlin) or cross-platform (Flutter/React Native)?",
        "Should it export: GIF / MP4 / WebM / PNG sequences?",
    ],
    executes=[
        "RECOMMENDED STACK: Flutter (single codebase → iOS + Android + Web + Desktop)",
        "",
        "Generate complete Flutter project:",
        "  ✓ iOS Human Interface Guidelines design system in Flutter",
        "  ✓ Cupertino widgets + custom animated components",
        "  ✓ Canvas-based animation editor (CustomPainter)",
        "  ✓ Layer system: text, images, shapes, stickers",
        "  ✓ Timeline with keyframe animation controls",
        "  ✓ Preset animation templates for social media",
        "  ✓ Export to MP4 via ffmpeg Flutter plugin",
        "  ✓ Template gallery with content creator presets",
        "  ✓ Responsive layout: iPhone, iPad, Android tablet, web",
        "  ✓ Dark mode + Light mode",
        "",
        "Core animation features:",
        "  - Spring animations, easing curves, bounce effects",
        "  - Text animation presets (typewriter, fade, slide)",
        "  - Particle effects system",
        "  - Green screen / chroma key (if video input enabled)",
        "  - Brand kit: save colors, fonts, logos per account",
        "",
        "Export pipeline:",
        "  - MP4 at 1080p/4K",
        "  - Story format (9:16), Square (1:1), Landscape (16:9)",
        "  - GIF export for web use",
    ],
    delivers=[
        "Complete Flutter project — runs on iOS, Android, Web from one codebase",
        "30+ animation presets for content creators",
        "Export to all major formats",
        "App store submission guide (iOS + Google Play)",
        "Monetization strategy (freemium, subscription, one-time)",
    ]
),

# ─── 5. Play GTA San Andreas missions ────────────────────────────────────────

"gta_sa_play": ScenarioResponse(
    user_input="Help me play GTA San Andreas and finish 5 missions",
    what_it_says="""
  🐉 Ready to play. A few things to confirm first.
""",
    clarifies=[
        "Is GTA San Andreas open on your screen right now?",
        "Which 5 missions? (or should I pick the next 5 in story order?)",
        "PC version? (Steam/original) — windowed or fullscreen?",
        "Resolution and window size?",
    ],
    executes=[
        "PRE-FLIGHT CHECK:",
        "  Verify game is running and in focus",
        "  Detect window bounds for screen capture",
        "  Confirm game version (affects control timings)",
        "",
        "SETUP (user runs once):",
        "  pip install mss pynput opencv-python-headless numpy",
        "  Set game to Windowed mode: Esc → Display → Windowed",
        "  Make sure GTA SA window is active and focused",
        "  Run: microdragon game play 'GTA San Andreas' --missions 5",
        "",
        "WHAT MICRODRAGON DOES DURING PLAY:",
        "  Captures game screen at 30fps via mss",
        "  OpenCV reads: health bar, minimap, mission objective text",
        "  Reads mission name from screen to identify current objective",
        "  Uses verified mission knowledge for each SA mission:",
        "    Big Smoke (Grove St) → Follow instructions, stay alive",
        "    Sweet & Kendl → Cycle race, draft behind Sweet",
        "    Ryder → Stay in truck, shoot enemies drive-by",
        "    Drive-Through → Navigate to each restaurant, avoid Ballas",
        "    Cleaning the Hood → Fight Freddy, protect Smoke",
        "  Sends keyboard inputs via pynput for movement, combat, vehicles",
        "  Monitors health — retreats to cover when below 25%",
        "  Mission complete detection via HUD change",
        "",
        "LIVE STATUS (shown while playing):",
        "  Mission: Big Smoke | Health: 87% | Status: Following objective",
        "  Inputs/sec: 12 | Deaths: 0 | Missions done: 0/5",
    ],
    delivers=[
        "Live terminal status during gameplay",
        "Mission completion log",
        "Performance report after all 5 missions",
        "Control back to user after each mission",
    ],
    honest_note="""
  IMPORTANT — HOW TO MAKE MICRODRAGON PLAY GTA:

  Step 1: Install game requirements
    pip install mss pynput opencv-python-headless numpy

  Step 2: Open GTA San Andreas on your PC
    - Must be WINDOWED mode (not fullscreen)
    - Screen resolution: 1280x720 or higher recommended
    - Keep the window focused (don't minimize)

  Step 3: Run Microdragon
    microdragon game play "GTA San Andreas"

  Step 4: Watch it play
    Microdragon reads the screen with computer vision and sends
    keyboard inputs automatically. You can stop at any time:
    microdragon game stop

  WHAT IT SEES: Health bar, minimap, HUD elements, road/path
  WHAT IT DOES: WASD movement, mouse aim, space for jump/handbrake
  ACCURACY: ~75-80% on open world driving, ~70% on combat missions
  SA vs GTA V: SA's older graphics make it slightly easier for CV detection
"""
),

# ─── 6. Authorized security test (pentesting) ────────────────────────────────

"pentest_authorized": ScenarioResponse(
    user_input="A company gave me a job to help test the security of their website — treen.com",
    what_it_says="""
  🐉 Security testing. I need to confirm authorisation before we do anything.
""",
    clarifies=[
        "Do you have WRITTEN authorisation from treen.com? (email, signed scope doc, or contract?)",
        "What is the agreed scope? (web app only / APIs / network / social engineering?)",
        "Are there systems to EXCLUDE? (live payment processing, production databases?)",
        "What's the timeline and reporting format they expect?",
        "Your role: solo tester or part of a team?",
    ],
    executes=[
        "PHASE 1 — RECONNAISSANCE (passive, no touching the server):",
        "  WHOIS + DNS enumeration (subdomains, MX, NS records)",
        "  Certificate Transparency search (crt.sh) for hidden subdomains",
        "  Shodan scan for exposed services and banners",
        "  Google dorking: site:treen.com filetype:env OR filetype:sql",
        "  LinkedIn/GitHub OSINT for tech stack and employee info",
        "  Wayback Machine for historical content and old endpoints",
        "",
        "PHASE 2 — SCANNING (with permission):",
        "  nmap -sV -sC -p- treen.com (service detection)",
        "  Nikto web server scan (misconfigurations, default files)",
        "  gobuster/ffuf directory and endpoint brute force",
        "  Identify tech stack: framework, CMS, libraries, versions",
        "",
        "PHASE 3 — VULNERABILITY ANALYSIS:",
        "  Automated: Nuclei templates against detected tech",
        "  Manual: OWASP Top 10 checklist (SQL injection, XSS, IDOR, etc.)",
        "  Auth testing: brute force protection, session management, JWT",
        "  Input validation: every form field, API parameter, file upload",
        "  Logic flaws: price manipulation, privilege escalation, IDOR",
        "",
        "PHASE 4 — CONTROLLED EXPLOITATION (PoC only):",
        "  Demonstrate vulnerability exists (screenshot/response capture)",
        "  Do NOT exfiltrate real data — capture enough to prove the issue",
        "  Document exact reproduction steps",
        "  Notify client immediately if critical finding",
        "",
        "PHASE 5 — REPORT GENERATION:",
        "  Executive summary (non-technical, business impact)",
        "  Technical findings with CVSS scores",
        "  Evidence screenshots and reproduction steps",
        "  Prioritised remediation recommendations",
        "  Generate professional PDF report",
    ],
    delivers=[
        "Full recon report (subdomains, tech stack, exposed services)",
        "Vulnerability findings sorted by severity (Critical/High/Medium/Low)",
        "Professional pentest report PDF ready to deliver to client",
        "Remediation recommendations for each finding",
    ],
    honest_note="""
  ⚠ IMPORTANT:
  Microdragon will NOT start any scanning without confirmed written authorisation.
  If you say 'yes they authorised me' and proceed, you are legally responsible.
  Microdragon's security tools are for AUTHORISED testing only.
  Scanning a website without permission is illegal in most countries.

  If you have the authorisation doc, paste the key scope details and we proceed.
  If not, I can help you draft the authorisation request to send to treen.com.
"""
),

# ─── 7. Prepare marketing meeting ─────────────────────────────────────────────

"prepare_meeting": ScenarioResponse(
    user_input="Prepare the meeting for the company's marketing scope tomorrow",
    what_it_says="""
  🐉 Preparing tomorrow's marketing meeting. I'll get this fully organised.
""",
    clarifies=[
        "What company and what's the full context of the marketing scope?",
        "Who is attending? (names, roles, decision-makers?)",
        "What time and where? (Zoom link, office address, or phone?)",
        "Any specific agenda items already agreed?",
        "What's the goal of the meeting? (approval, planning, review, pitch?)",
    ],
    executes=[
        "Create meeting agenda document with:",
        "  - Meeting title, date, time, location",
        "  - Attendee list with roles",
        "  - Timed agenda (each item with allocated minutes)",
        "  - Pre-meeting reading list if relevant",
        "  - Discussion questions per agenda item",
        "  - Decision/outcome goals for each point",
        "",
        "Create pre-meeting briefing note:",
        "  - Context summary for attendees who need background",
        "  - Key data points to have ready",
        "  - Competitor/market data if needed (research)",
        "",
        "Add to calendar:",
        "  - Create calendar event with all attendee emails",
        "  - Set reminder 1 hour before and 1 day before",
        "",
        "Create notes template:",
        "  - Live note-taking document with agenda structure",
        "  - Action items table (who/what/by when)",
        "  - Decisions log",
    ],
    delivers=[
        "Meeting agenda document (Word/PDF)",
        "Attendee briefing notes",
        "Calendar invite sent to all attendees",
        "Live notes template ready to fill during meeting",
        "Follow-up email template for after the meeting",
    ]
),

# ─── 8. Send contract terms by email ──────────────────────────────────────────

"send_contract_email": ScenarioResponse(
    user_input="Send the contract terms to adeshinaemmanuel333@gmail.com",
    what_it_says="""
  🐉 Ready to send. But I need to show you the full email before sending.
  I never send anything without your explicit confirmation.
""",
    clarifies=[
        "What contract terms? (paste the text, or attach the file)",
        "Who is this from? (your name and company)",
        "Any covering message to include with the contract?",
        "Subject line preference, or should I write one?",
    ],
    executes=[
        "Compose the email with contract terms",
        "SHOW FULL PREVIEW — including:",
        "  To: adeshinaemmanuel333@gmail.com",
        "  Subject: [generated or user-specified]",
        "  Body: covering message + contract terms",
        "  Attachment: if file provided",
        "",
        "⚠ CONFIRMATION GATE — display:",
        "  'Type YES to send this email, or NO to edit it'",
        "",
        "Only after 'YES': send via configured email provider",
        "Log the send action to audit trail",
        "Confirm delivery receipt",
    ],
    delivers=[
        "Email preview before sending",
        "Send confirmation with timestamp",
        "Audit log entry",
    ],
    honest_note="Email requires SMTP configuration (Gmail App Password or other provider). Run: microdragon setup email — to configure. Microdragon NEVER sends email without showing the full preview and waiting for your 'YES'."
),

# ─── 9. Remind John ────────────────────────────────────────────────────────────

"remind_person": ScenarioResponse(
    user_input="Remember John to bring the product tomorrow",
    what_it_says="""
  🐉 Got it. How should I remind John?
""",
    clarifies=[
        "How do I reach John? (WhatsApp number / Telegram / email / Discord?)",
        "What time tomorrow should the reminder go out?",
        "What's the product exactly? (for the message to be specific)",
    ],
    executes=[
        "If WhatsApp configured:",
        "  Schedule message to John's number at specified time",
        "  Message: 'Hi John! Just a reminder to bring [product] tomorrow. Thanks!'",
        "  Confirm scheduling",
        "",
        "If no contact channel configured:",
        "  Save reminder in MICRODRAGON memory",
        "  Alert you at the specified time",
        "  Provide John's message text for you to copy-paste",
        "",
        "Set own reminder:",
        "  Add calendar event: 'John — bring product' at specified time",
        "  Desktop/terminal notification",
    ],
    delivers=[
        "WhatsApp/Telegram message scheduled to John",
        "OR reminder alert to you at the right time",
        "Confirmation of what was scheduled",
    ]
),

# ─── 10. Winning presentation ─────────────────────────────────────────────────

"winning_presentation": ScenarioResponse(
    user_input="Create the winning presentation for tomorrow's contract with [Company]",
    what_it_says="""
  🐉 Building a contract-winning presentation. Let's get the details right.
""",
    clarifies=[
        "What company are you presenting to? And what's the contract worth?",
        "What is the core offer — what exactly are you proposing to do for them?",
        "What do you know about their main problem or goal?",
        "Any competitors you're up against?",
        "How long is the presentation? (10 / 20 / 30 minutes?)",
        "Do you have a brand kit? (colors, logo, font?)",
    ],
    executes=[
        "RESEARCH (before designing):",
        "  Research the company — recent news, challenges, wins",
        "  Find the decision-maker's background (LinkedIn)",
        "  Identify their industry pain points",
        "  Frame your solution in their language",
        "",
        "BUILD PRESENTATION (6 slides minimum, expandable):",
        "  Slide 1: Title — your name, company, date, clean hero",
        "  Slide 2: We understand your problem (their pain, not your solution)",
        "  Slide 3: Our solution — specific to their situation",
        "  Slide 4: Proof — case studies, numbers, credentials",
        "  Slide 5: Commercial terms — clear pricing, timeline, deliverables",
        "  Slide 6: Next steps — exact action they take to say yes",
        "",
        "DESIGN:",
        "  Modern SVG-based slides with your brand colors",
        "  Export as interactive HTML (present from browser)",
        "  Export as PDF (send as leave-behind)",
        "  Export individual SVGs (editable in Illustrator/Figma)",
        "",
        "SPEAKING NOTES:",
        "  Full script for each slide",
        "  Anticipated questions + prepared answers",
        "  Objection handling guide",
    ],
    delivers=[
        "Complete presentation (HTML interactive + PDF)",
        "Speaker notes for every slide",
        "Top 10 questions they might ask + your answers",
        "Email to send presentation after the meeting",
    ]
),

# ─── 11. Trading strategy ─────────────────────────────────────────────────────

"trading_strategy": ScenarioResponse(
    user_input="Our target is to make $1k with the $100 left in the trading account — plan the winning strategy",
    what_it_says="""
  🐉 Understood. I'll plan a structured strategy. And I'll be honest with you.

  ⚠ Making 10x in one day requires high-risk trades.
    High risk = high chance of losing the $100 too.
    I'll give you the most strategic approach AND be honest about the odds.
""",
    clarifies=[
        "What market are you trading? (Forex / Crypto / Stocks / Options?)",
        "What platform? (Binance / MetaTrader / Robinhood / other?)",
        "Timeframe — intraday (today) or swing (days to weeks)?",
        "Any positions already open?",
        "Can you sit at the screen all day, or is this semi-automated?",
    ],
    executes=[
        "MARKET ANALYSIS:",
        "  Pull current price data for specified asset",
        "  Calculate RSI (14), MACD, Bollinger Bands",
        "  Identify key support and resistance levels",
        "  Check upcoming high-impact news events (Fed, earnings, CPI)",
        "  Assess current market sentiment",
        "",
        "STRATEGY (10x account requires compounding small wins):",
        "  Option A — Compound small wins (lower risk):",
        "    Target: 3-4 trades at 30-40% gain each = reaches $1k",
        "    Risk per trade: 20% of account ($20 per trade)",
        "    Only enter highest-conviction setups",
        "",
        "  Option B — Single high-leverage trade (very high risk):",
        "    Only with clear trend confirmation",
        "    Strict stop-loss at 50% of account ($50)",
        "    Take profit at target",
        "",
        "EXECUTION PLAN:",
        "  Specific entry price",
        "  Stop-loss price",
        "  Take-profit levels (multiple targets)",
        "  Position size calculation",
        "  Entry conditions to confirm before placing trade",
        "",
        "MONITORING:",
        "  Set price alerts at key levels",
        "  Monitor every 15-30 minutes",
        "  Defined exit rule if thesis changes",
    ],
    delivers=[
        "Full market analysis for specified asset",
        "Specific trade plan: entry / stop-loss / take-profit / position size",
        "Risk management rules",
        "Price alerts configured",
        "Realistic probability assessment",
    ],
    honest_note="⚠ Not financial advice. 10x return in one day is high-risk by definition. Microdragon will give you the best possible analysis and strategy. The market does not guarantee outcomes. Never trade money you cannot afford to lose entirely."
),

# ─── 12. Forward plan to team ─────────────────────────────────────────────────

"forward_to_team": ScenarioResponse(
    user_input="Forward the new plan to the team",
    what_it_says="""
  🐉 Forward the plan to the team. I need a few details.
""",
    clarifies=[
        "What is the plan? (paste it or describe it)",
        "Who is 'the team'? (group chat on WhatsApp/Telegram? Email list? Discord?)",
        "Any added context or message to include?",
        "Send now or schedule for a specific time?",
    ],
    executes=[
        "Format the plan as a clean, readable message",
        "SHOW PREVIEW of exactly what will be sent",
        "Wait for your confirmation",
        "After YES: send to specified channel/group",
        "Log the send action",
    ],
    delivers=[
        "Message preview before sending",
        "Confirmation after delivery",
        "Audit trail entry",
    ]
),

# ─── 13. Business check-in ────────────────────────────────────────────────────

"business_checkin": ScenarioResponse(
    user_input="Check on the business — how is the team, how is sales, what is the total profit this month",
    what_it_says="""
  🐉 Business check-in. Let me pull what I can access.
""",
    clarifies=[
        "What platforms/tools is your business on? (Shopify / WooCommerce / custom / spreadsheet?)",
        "Team communication: Slack / WhatsApp / Telegram / Discord?",
        "Is there a spreadsheet or dashboard tracking sales and profit?",
        "Should I message the team to give you a status update?",
    ],
    executes=[
        "If Shopify connected:",
        "  Pull: total revenue this month, orders, AOV, top products, refunds",
        "",
        "If spreadsheet/Google Sheets connected:",
        "  Read revenue, expenses, calculate profit margin",
        "",
        "If social channels connected:",
        "  Send team a status request message",
        "  Compile responses when received",
        "",
        "Generate morning briefing:",
        "  Revenue this month vs last month",
        "  Outstanding orders",
        "  Key metrics",
        "  Team status",
        "  Action items",
    ],
    delivers=[
        "Business snapshot report",
        "Revenue and profit numbers",
        "Team status compilation",
        "Action items for today",
    ],
    honest_note="Microdragon can only see what it has access to. Connect your sales platform (Shopify, WooCommerce, or spreadsheet) for real numbers. Without access, I'll ask your team for an update."
),

# ─── 14. JAMB 320+ ───────────────────────────────────────────────────────────

"jamb_study": ScenarioResponse(
    user_input="We need to get 320+ in JAMB",
    what_it_says="""
  🐉 320+ in JAMB. That's in the top tier — very achievable with the right plan.
  Let's build a structured study system.
""",
    clarifies=[
        "What are your 4 JAMB subjects?",
        "When is your exam date?",
        "How many hours per day can you study?",
        "Which subjects are strongest and which need the most work?",
        "Have you done any past questions yet?",
    ],
    executes=[
        "SCORE ANALYSIS:",
        "  320/400 = 80% average across 4 subjects",
        "  Target per subject: 75-85 raw marks",
        "  Identify your current level per subject",
        "",
        "STUDY PLAN (generated with your exam date):",
        "  Week-by-week topic breakdown for each subject",
        "  Daily schedule with specific topics",
        "  Past question sets for each topic (JAMB 2020-2024)",
        "  High-frequency topics analysis (what comes up most)",
        "",
        "DAILY ROUTINE:",
        "  Morning: hardest subject first",
        "  Afternoon: practice questions, timed",
        "  Evening: review mistakes, not new material",
        "",
        "RESOURCES:",
        "  Free: Myschool.ng, Waec past questions, YouTube tutorials",
        "  Key topics guaranteed to appear per subject",
        "  Common tricks and traps in JAMB questions",
        "",
        "PROGRESS TRACKING:",
        "  Weekly mock tests",
        "  Score tracker",
        "  Weak topic identification",
    ],
    delivers=[
        "Personalised week-by-week study plan to your exam date",
        "Top 50 most repeated topics per subject",
        "Daily question sets with answers",
        "Weekly mock test schedule",
        "320+ score breakdown: exactly how many to get right per subject",
    ]
),

# ─── 15. Winning ads ──────────────────────────────────────────────────────────

"create_ads": ScenarioResponse(
    user_input="Create a winning ad for [brand]",
    what_it_says="""
  🐉 Building a complete ad campaign. Research before creative.
""",
    clarifies=[
        "What is the brand name and what do they sell?",
        "Who is the target customer? (age, location, interests, income)",
        "What platform? (Facebook/Instagram / TikTok / Google / YouTube / all?)",
        "What is the ad budget per day/week?",
        "What's the goal? (sales / leads / awareness / website traffic)",
        "Any brand colors, fonts, or existing creative?",
        "What makes this brand different from competitors?",
    ],
    executes=[
        "BRAND + COMPETITOR RESEARCH:",
        "  Analyse brand positioning vs competitors",
        "  Study competitor ads on Facebook Ad Library",
        "  Identify winning hooks, formats, and offers in the niche",
        "  Find the USP gaps competitors are missing",
        "",
        "AD STRATEGY:",
        "  Recommended formats per platform",
        "  Offer structure (direct / lead gen / awareness funnel)",
        "  Hook strategy (pain point / desire / curiosity / social proof)",
        "  Call-to-action optimisation",
        "",
        "AD CREATIVE (full scripts):",
        "  3 x video ad scripts (15s, 30s, 60s versions)",
        "  5 x ad headline variants (A/B test ready)",
        "  3 x ad body copy variants",
        "  Image ad creative brief (visual direction + copy)",
        "  Carousel ad sequence",
        "",
        "AUDIENCE TARGETING:",
        "  Facebook/TikTok audience recommendations",
        "  Interest targeting layers",
        "  Lookalike audience strategy",
        "  Retargeting setup",
        "",
        "VISUAL DIRECTION:",
        "  Color palette for ads",
        "  Font and text overlay recommendations",
        "  Shot list for video ads",
        "  AI video/image generation prompt (if provider configured)",
    ],
    delivers=[
        "Full ad strategy document",
        "3 complete video ad scripts (ready to record)",
        "15+ ad copy variants for A/B testing",
        "Audience targeting setup guide",
        "Visual direction brief",
        "Campaign structure (budget allocation, bidding strategy)",
        "First-week testing plan",
    ]
),

# ─── 16. Zero-capital first $1k ──────────────────────────────────────────────

"zero_capital_business": ScenarioResponse(
    user_input="I need to make money with no capital — let's start a business and make the first $1k",
    what_it_says="""
  🐉 Zero capital, first $1k. Completely possible — people do this every week.
  The key is service-based: sell skills or time, no inventory needed.
  I'll help you identify what you can offer TODAY and get your first client.
""",
    clarifies=[
        "What are you good at? (doesn't have to be technical — writing, design, speaking, research, social media, video editing, translating, tutoring, anything)",
        "How much time can you spend on this per day?",
        "Do you have a smartphone? (enough for most service businesses)",
        "Which country are you in? (affects payment options and platforms)",
        "Have you tried earning online before?",
    ],
    executes=[
        "SKILL AUDIT + OPPORTUNITY MATCHING:",
        "  Map your skills to in-demand services",
        "  Find services people pay $50-500 for right now",
        "  Recommend top 3 services you can start today",
        "  Analyse competition and positioning angle",
        "",
        "ZERO-COST BUSINESS SETUP:",
        "  Free portfolio: Notion / Canva / GitHub Pages",
        "  Payment: PayPal / Paystack / Wise (all free to sign up)",
        "  Lead gen: Upwork / Fiverr / LinkedIn / Twitter DMs / WhatsApp",
        "  Tools: all free tier",
        "",
        "FIRST CLIENT ACQUISITION PLAN:",
        "  Write 10 personalised outreach messages",
        "  Identify specific people/businesses to target",
        "  Platform strategy: where your buyers actually are",
        "  Pricing strategy: not too cheap (signals low quality), not too expensive",
        "  Follow-up sequence",
        "",
        "DELIVER THE SERVICE:",
        "  Templates and processes to deliver fast",
        "  Quality checklist",
        "  How to handle revision requests",
        "  How to get a testimonial after delivery",
        "",
        "SCALE TO $1k:",
        "  How many clients at what price to hit $1k",
        "  Upsell strategy: turn one-time clients into recurring",
        "  Referral strategy: clients bring more clients",
        "  Timeline: realistic week-by-week milestones",
    ],
    delivers=[
        "Top 3 service business ideas matched to your skills",
        "Portfolio page (built and deployed, free)",
        "10 ready-to-send outreach messages",
        "Pricing guide for your chosen service",
        "Week-by-week roadmap to $1k",
        "Service delivery templates",
    ],
    honest_note="Zero capital businesses take hustle, not luck. The plan is realistic. The execution is yours. I'll build everything you need — you make the outreach and deliver the service."
),

}

# ─── Response formatter ───────────────────────────────────────────────────────

def format_scenario(scenario: ScenarioResponse) -> str:
    """Format a scenario response for terminal display."""
    lines = [""]

    lines.append(f"  🐉 MICRODRAGON RESPONSE")
    lines.append(f"  User: \"{scenario.user_input}\"")
    lines.append("")
    lines.append("  MICRODRAGON SAYS:")
    for line in scenario.what_it_says.strip().split("\n"):
        lines.append(f"  {line}")

    if scenario.clarifies:
        lines.append("")
        lines.append("  ASKS FIRST:")
        for q in scenario.clarifies:
            lines.append(f"    → {q}")

    lines.append("")
    lines.append("  THEN DOES:")
    for step in scenario.executes:
        lines.append(f"  {step}")

    lines.append("")
    lines.append("  YOU GET:")
    for d in scenario.delivers:
        lines.append(f"    ✓ {d}")

    if scenario.honest_note:
        lines.append("")
        lines.append("  NOTE:")
        for line in scenario.honest_note.strip().split("\n"):
            lines.append(f"  {line.strip()}")

    lines.append("")
    return "\n".join(lines)
