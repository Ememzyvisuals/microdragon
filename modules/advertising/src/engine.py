"""
microdragon/modules/advertising/src/engine.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON ADVERTISING PIPELINE
═══════════════════════════════════════════════════════════════════════════════

Full advertising workflow:
  1. Brand Research     — analyse target brand, competitors, audience
  2. Competitor Intel   — research 3-5 competitors, find positioning gaps
  3. Strategy           — messaging, tone, hooks, USP
  4. Script             — full video/audio script with timestamps
  5. Visual Direction   — shot list, B-roll suggestions, color palette
  6. Video Generation   — calls image_video module with provider
  7. Output Package     — script + visuals + brief

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

import asyncio
import os
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class AdCampaign:
    brand:       str
    product:     str
    audience:    str
    goal:        str       # awareness | conversion | retention | engagement
    platform:    str       # tiktok | youtube | instagram | linkedin | tv
    duration:    int       # seconds
    tone:        str       # energetic | professional | humorous | emotional
    usp:         str       # unique selling proposition
    competitors: list = field(default_factory=list)


@dataclass
class AdScript:
    brand:       str
    platform:    str
    duration:    int
    hook:        str           # first 3 seconds — must stop the scroll
    acts:        list[dict]    # [{time, visual, audio, text_overlay}]
    cta:         str
    hashtags:    list[str]
    music_brief: str
    color_palette: list[str]
    shot_list:   list[str]
    video_prompt: str          # prompt for video generation API


class AdvertisingEngine:
    """Full advertising pipeline from brief to production-ready script."""

    PLATFORM_SPECS = {
        "tiktok":    {"ratio": "9:16", "max_sec": 60,  "hook_sec": 3,  "optimal": 15},
        "youtube":   {"ratio": "16:9", "max_sec": 600, "hook_sec": 5,  "optimal": 90},
        "instagram": {"ratio": "9:16", "max_sec": 90,  "hook_sec": 3,  "optimal": 30},
        "linkedin":  {"ratio": "16:9", "max_sec": 300, "hook_sec": 5,  "optimal": 60},
        "tv":        {"ratio": "16:9", "max_sec": 60,  "hook_sec": 5,  "optimal": 30},
    }

    def build_script_prompt(self, campaign: AdCampaign) -> str:
        """Build an AI prompt that generates a complete ad script."""
        spec = self.PLATFORM_SPECS.get(campaign.platform, self.PLATFORM_SPECS["youtube"])
        return f"""You are a senior advertising creative director and copywriter.
Create a complete {campaign.duration}-second ad script for {campaign.brand}.

BRIEF:
- Product: {campaign.product}
- Platform: {campaign.platform} ({spec['ratio']}, optimal {spec['optimal']}s)
- Target Audience: {campaign.audience}
- Campaign Goal: {campaign.goal}
- Tone: {campaign.tone}
- USP: {campaign.usp}
- Competitors: {', '.join(campaign.competitors) if campaign.competitors else 'not specified'}

OUTPUT STRUCTURE (exactly this format):

## HOOK (0-{spec['hook_sec']}s)
[Must stop the scroll / capture attention immediately]
Visual: [describe scene]
Audio/VO: [exact words spoken]
Text overlay: [on-screen text]

## ACT 1 ({spec['hook_sec']}-{campaign.duration//3}s)
[Problem or desire]
Visual: [scene description]
Audio/VO: [script]
Text overlay: [text]

## ACT 2 ({campaign.duration//3}-{2*campaign.duration//3}s)
[Solution / product reveal]
Visual: [scene]
Audio/VO: [script]
Text overlay: [text]

## CTA ({2*campaign.duration//3}-{campaign.duration}s)
[Clear call to action]
Visual: [scene]
Audio/VO: [exact CTA words]
Text overlay: [URL or action]

## METADATA
Hook options (3 alternatives):
Music brief:
Color palette (3 hex codes):
Shot list (5 specific shots):
Video generation prompt (for AI video API):
Hashtags (15 for {campaign.platform}):

Make it {campaign.tone}, compelling, and optimised for {campaign.platform} algorithm.
Every second must earn its place."""

    def build_research_prompt(self, brand: str, product: str, competitors: list) -> str:
        """Build a research prompt for brand + competitor analysis."""
        return f"""You are a senior brand strategist and market researcher.
Analyse {brand} and their product: {product}

DELIVER:

## TARGET AUDIENCE ANALYSIS
- Primary demographics (age, income, psychographics)
- Pain points being solved
- Decision triggers
- Content they consume

## COMPETITOR ANALYSIS
For each competitor: {', '.join(competitors) if competitors else 'identify 3 main competitors'}
- Their messaging angle
- Where they are strong
- Where they are weak
- Positioning gap we can own

## BRAND POSITIONING
- Unique differentiator for {brand}
- Messaging hierarchy (primary, secondary, tertiary)
- Tone of voice keywords (5 words)
- What to avoid

## AD HOOK IDEAS
- 5 hooks based on psychology (curiosity, fear, desire, surprise, social proof)
- Specify which trigger each uses

Be specific. No generic advice. Think like you're billing $500/hour."""

    def format_script_output(self, brand: str, platform: str,
                              duration: int, ai_output: str) -> str:
        """Format the AI-generated script into a clean production document."""
        spec = self.PLATFORM_SPECS.get(platform, self.PLATFORM_SPECS["youtube"])
        return f"""
🐉 MICRODRAGON ADVERTISING SUITE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
BRAND:     {brand}
PLATFORM:  {platform.upper()} | {spec['ratio']} | {duration}s
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

{ai_output}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Generated by Microdragon · EMEMZYVISUALS DIGITALS
Next step: microdragon video generate "<video_prompt_from_above>"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
"""
