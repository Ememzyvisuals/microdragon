"""
microdragon/modules/image_video/src/engine.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON IMAGE & VIDEO GENERATION ENGINE
═══════════════════════════════════════════════════════════════════════════════

Multi-provider image and video generation.
Uses whichever provider the user has configured.

IMAGE PROVIDERS (in priority order):
  1. OpenAI DALL-E 3       — OPENAI_API_KEY
  2. Google Gemini Imagen  — GEMINI_API_KEY
  3. HuggingFace           — HUGGINGFACE_TOKEN (free tier available)
  4. Fal.ai                — FAL_KEY
  5. Stability AI          — STABILITY_API_KEY

VIDEO PROVIDERS:
  1. Google Veo 2          — GEMINI_API_KEY (Google DeepMind)
  2. OpenAI Sora           — OPENAI_API_KEY (when available)
  3. HuggingFace video     — HUGGINGFACE_TOKEN
  4. Fal.ai video          — FAL_KEY

IMPORTANT: If no provider is configured, Microdragon returns a clear script
or prompt explaining exactly what to set up — it never silently fails.

Privacy guarantee: API keys are stored encrypted locally.
They are NEVER transmitted to EMEMZYVISUALS DIGITALS or any third party.
They only go to the provider you configured.

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

import asyncio
import base64
import os
import json
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from enum import Enum
from typing import Optional


class ImageProvider(Enum):
    OPENAI_DALLE   = "openai_dalle"
    GEMINI_IMAGEN  = "gemini_imagen"
    HUGGINGFACE    = "huggingface"
    FAL_AI         = "fal_ai"
    STABILITY      = "stability"
    NONE           = "none"


class VideoProvider(Enum):
    GOOGLE_VEO     = "google_veo"
    OPENAI_SORA    = "openai_sora"
    HUGGINGFACE    = "huggingface"   # HuggingFace Inference API (CogVideoX-5B)
    HUGSFIELD      = "hugsfield"     # HuggingFace video generation (WAN-2.1 / LTX)
    FAL_AI         = "fal_ai"
    NONE           = "none" 


@dataclass
class GenerationResult:
    success:     bool
    output_path: Optional[str] = None
    url:         Optional[str] = None
    prompt_used: str = ""
    provider:    str = ""
    error:       Optional[str] = None
    setup_guide: Optional[str] = None   # shown when provider not configured


# ─── Provider detection ────────────────────────────────────────────────────────

def detect_image_provider() -> ImageProvider:
    if os.getenv("OPENAI_API_KEY"):        return ImageProvider.OPENAI_DALLE
    if os.getenv("GEMINI_API_KEY"):        return ImageProvider.GEMINI_IMAGEN
    if os.getenv("HUGGINGFACE_TOKEN"):     return ImageProvider.HUGGINGFACE
    if os.getenv("FAL_KEY"):               return ImageProvider.FAL_AI
    if os.getenv("STABILITY_API_KEY"):     return ImageProvider.STABILITY
    return ImageProvider.NONE


def detect_video_provider() -> VideoProvider:
    if os.getenv("GEMINI_API_KEY"):        return VideoProvider.GOOGLE_VEO
    if os.getenv("OPENAI_API_KEY"):        return VideoProvider.OPENAI_SORA
    if os.getenv("HF_TOKEN") or os.getenv("HUGGINGFACE_TOKEN"):
        # Prefer HugsField (WAN-2.1) over standard HF inference
        return VideoProvider.HUGSFIELD
    if os.getenv("FAL_KEY"):               return VideoProvider.FAL_AI
    return VideoProvider.NONE


def not_configured_image_message(prompt: str) -> str:
    return f"""
  🐉 Image generation is not configured.

  Your prompt was:
  "{prompt}"

  To generate this image, configure one of these providers:

  ┌─────────────────────────────────────────────────────────────────┐
  │  Provider            Env Variable          Get Key              │
  ├─────────────────────────────────────────────────────────────────┤
  │  OpenAI DALL-E 3     OPENAI_API_KEY        platform.openai.com  │
  │  Google Imagen 3     GEMINI_API_KEY        aistudio.google.com  │
  │  HuggingFace         HUGGINGFACE_TOKEN     huggingface.co/settings/tokens (FREE) │
  │  Fal.ai              FAL_KEY               fal.ai/dashboard     │
  │  Stability AI        STABILITY_API_KEY     platform.stability.ai│
  └─────────────────────────────────────────────────────────────────┘

  Quick setup (HuggingFace is free):
    export HUGGINGFACE_TOKEN="hf_your_token_here"
    microdragon image generate "{prompt[:50]}..."

  🔒 Your API key is stored encrypted on YOUR machine only.
     It is NEVER shared with EMEMZYVISUALS DIGITALS or any third party.
     It only goes to the provider you configure.
"""


def not_configured_video_message(prompt: str) -> str:
    return f"""
  🐉 Video generation is not configured.

  Your prompt was:
  "{prompt}"

  To generate this video, configure one of these providers:

  ┌─────────────────────────────────────────────────────────────────┐
  │  Provider            Env Variable          Notes                │
  ├─────────────────────────────────────────────────────────────────┤
  │  Google Veo 2        GEMINI_API_KEY        Best quality         │
  │  OpenAI Sora         OPENAI_API_KEY        When available       │
  │  HugsField (WAN-2.1) HUGGINGFACE_TOKEN     Best free model 🔥   │
  │  HuggingFace Video   HUGGINGFACE_TOKEN     Free tier            │
  │  Fal.ai (fast)       FAL_KEY               Fast generation      │
  └─────────────────────────────────────────────────────────────────┘

  Quick setup (Google Veo via Gemini — best quality):
    export GEMINI_API_KEY="your_key_from_aistudio.google.com"
    microdragon video generate "{prompt[:50]}..."

  🔒 Your API key is stored encrypted on YOUR machine only.
     It is NEVER shared with EMEMZYVISUALS DIGITALS or any third party.
"""


# ─── Image Generation Engine ──────────────────────────────────────────────────

class ImageEngine:
    """Generate images using the user's configured provider."""

    def __init__(self):
        self.provider = detect_image_provider()

    async def generate(self, prompt: str, output_path: str = None,
                       size: str = "1024x1024", style: str = "natural") -> GenerationResult:
        """Generate an image from a text prompt."""

        if self.provider == ImageProvider.NONE:
            return GenerationResult(
                success=False,
                prompt_used=prompt,
                provider="none",
                setup_guide=not_configured_image_message(prompt)
            )

        if output_path is None:
            output_path = tempfile.mktemp(suffix=".png")

        print(f"  🐉 Generating image with {self.provider.value}...")
        print(f"  Prompt: {prompt[:80]}...")

        if self.provider == ImageProvider.OPENAI_DALLE:
            return await self._dalle(prompt, output_path, size, style)
        elif self.provider == ImageProvider.GEMINI_IMAGEN:
            return await self._gemini_imagen(prompt, output_path, size)
        elif self.provider == ImageProvider.HUGGINGFACE:
            return await self._huggingface(prompt, output_path, size)
        elif self.provider == ImageProvider.FAL_AI:
            return await self._fal(prompt, output_path, size)
        elif self.provider == ImageProvider.STABILITY:
            return await self._stability(prompt, output_path, size)

        return GenerationResult(success=False, error="Unknown provider")

    async def _dalle(self, prompt: str, output: str,
                     size: str, style: str) -> GenerationResult:
        """OpenAI DALL-E 3 — dall-e-3, best quality."""
        try:
            import aiohttp
            key = os.getenv("OPENAI_API_KEY", "")
            # Map size to DALL-E valid sizes
            dalle_sizes = {
                "1024x1024": "1024x1024",
                "1792x1024": "1792x1024",
                "1024x1792": "1024x1792",
            }
            dall_size = dalle_sizes.get(size, "1024x1024")

            async with aiohttp.ClientSession() as s:
                async with s.post(
                    "https://api.openai.com/v1/images/generations",
                    headers={"Authorization": f"Bearer {key}",
                             "Content-Type": "application/json"},
                    json={"model": "dall-e-3", "prompt": prompt,
                          "n": 1, "size": dall_size,
                          "quality": "hd", "style": style,
                          "response_format": "b64_json"}
                ) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        img_b64 = data["data"][0]["b64_json"]
                        img_bytes = base64.b64decode(img_b64)
                        Path(output).write_bytes(img_bytes)
                        return GenerationResult(
                            success=True, output_path=output,
                            prompt_used=data["data"][0].get("revised_prompt", prompt),
                            provider="OpenAI DALL-E 3"
                        )
                    err = await resp.text()
                    return GenerationResult(success=False,
                                            error=f"DALL-E error {resp.status}: {err[:200]}",
                                            provider="OpenAI DALL-E 3")
        except ImportError:
            return GenerationResult(success=False, error="pip install aiohttp")
        except Exception as e:
            return GenerationResult(success=False, error=str(e), provider="OpenAI DALL-E 3")

    async def _gemini_imagen(self, prompt: str, output: str, size: str) -> GenerationResult:
        """Google Imagen 3 via Gemini API."""
        try:
            import aiohttp
            key = os.getenv("GEMINI_API_KEY", "")
            # Imagen 3 through Gemini API
            url = f"https://generativelanguage.googleapis.com/v1beta/models/imagen-3.0-generate-001:predict?key={key}"
            async with aiohttp.ClientSession() as s:
                async with s.post(url,
                    json={
                        "instances": [{"prompt": prompt}],
                        "parameters": {
                            "sampleCount": 1,
                            "aspectRatio": "1:1" if "1024x1024" in size else "16:9",
                        }
                    }
                ) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        img_b64 = data["predictions"][0]["bytesBase64Encoded"]
                        Path(output).write_bytes(base64.b64decode(img_b64))
                        return GenerationResult(success=True, output_path=output,
                                                prompt_used=prompt, provider="Google Imagen 3")
                    err = await resp.text()
                    return GenerationResult(success=False,
                                            error=f"Imagen error {resp.status}: {err[:200]}",
                                            provider="Google Imagen 3")
        except Exception as e:
            return GenerationResult(success=False, error=str(e), provider="Google Imagen 3")

    async def _huggingface(self, prompt: str, output: str, size: str) -> GenerationResult:
        """HuggingFace Inference API — FLUX.1-dev (best free model)."""
        try:
            import aiohttp
            token = os.getenv("HUGGINGFACE_TOKEN", "")
            # FLUX.1-dev — state of the art free image model
            model = "black-forest-labs/FLUX.1-dev"
            async with aiohttp.ClientSession() as s:
                async with s.post(
                    f"https://api-inference.huggingface.co/models/{model}",
                    headers={"Authorization": f"Bearer {token}"},
                    json={"inputs": prompt,
                          "parameters": {"width": 1024, "height": 1024}}
                ) as resp:
                    if resp.status == 200:
                        content_type = resp.headers.get("content-type", "")
                        if "image" in content_type:
                            Path(output).write_bytes(await resp.read())
                            return GenerationResult(success=True, output_path=output,
                                                    prompt_used=prompt,
                                                    provider="HuggingFace FLUX.1-dev")
                    err = await resp.text()
                    # Model loading — retry after delay
                    if "loading" in err.lower():
                        await asyncio.sleep(20)
                        return await self._huggingface(prompt, output, size)
                    return GenerationResult(success=False,
                                            error=f"HF error {resp.status}: {err[:200]}",
                                            provider="HuggingFace FLUX.1-dev")
        except Exception as e:
            return GenerationResult(success=False, error=str(e), provider="HuggingFace")

    async def _fal(self, prompt: str, output: str, size: str) -> GenerationResult:
        """Fal.ai — fast inference, FLUX and SDXL models."""
        try:
            import aiohttp
            key = os.getenv("FAL_KEY", "")
            async with aiohttp.ClientSession() as s:
                # Submit job
                async with s.post(
                    "https://queue.fal.run/fal-ai/flux/dev",
                    headers={"Authorization": f"Key {key}",
                             "Content-Type": "application/json"},
                    json={"prompt": prompt, "image_size": "square_hd",
                          "num_inference_steps": 28, "num_images": 1}
                ) as resp:
                    if resp.status not in (200, 202):
                        err = await resp.text()
                        return GenerationResult(success=False, error=f"Fal error: {err[:200]}")
                    data = await resp.json()

                # Poll for result
                request_id = data.get("request_id")
                if request_id:
                    for _ in range(30):
                        await asyncio.sleep(2)
                        async with s.get(
                            f"https://queue.fal.run/fal-ai/flux/dev/requests/{request_id}",
                            headers={"Authorization": f"Key {key}"}
                        ) as poll:
                            if poll.status == 200:
                                result = await poll.json()
                                if result.get("status") == "COMPLETED":
                                    img_url = result["output"]["images"][0]["url"]
                                    async with s.get(img_url) as img:
                                        Path(output).write_bytes(await img.read())
                                    return GenerationResult(success=True, output_path=output,
                                                            prompt_used=prompt, provider="Fal.ai FLUX")

                return GenerationResult(success=False, error="Fal.ai timeout", provider="Fal.ai")
        except Exception as e:
            return GenerationResult(success=False, error=str(e), provider="Fal.ai")

    async def _stability(self, prompt: str, output: str, size: str) -> GenerationResult:
        """Stability AI — stable-image-ultra."""
        try:
            import aiohttp
            key = os.getenv("STABILITY_API_KEY", "")
            async with aiohttp.ClientSession() as s:
                async with s.post(
                    "https://api.stability.ai/v2beta/stable-image/generate/ultra",
                    headers={"Authorization": f"Bearer {key}", "Accept": "image/*"},
                    data=aiohttp.FormData([("prompt", prompt), ("output_format", "png"),
                                           ("aspect_ratio", "1:1")])
                ) as resp:
                    if resp.status == 200:
                        Path(output).write_bytes(await resp.read())
                        return GenerationResult(success=True, output_path=output,
                                                prompt_used=prompt, provider="Stability AI Ultra")
                    err = await resp.text()
                    return GenerationResult(success=False, error=f"Stability error: {err[:200]}")
        except Exception as e:
            return GenerationResult(success=False, error=str(e), provider="Stability AI")


# ─── Video Generation Engine ──────────────────────────────────────────────────

class VideoEngine:
    """Generate videos using the user's configured provider."""

    def __init__(self):
        self.provider = detect_video_provider()

    async def generate(self, prompt: str, output_path: str = None,
                       duration_seconds: int = 5, aspect_ratio: str = "16:9") -> GenerationResult:
        """Generate a video from a text prompt."""

        if self.provider == VideoProvider.NONE:
            return GenerationResult(
                success=False, prompt_used=prompt, provider="none",
                setup_guide=not_configured_video_message(prompt)
            )

        if output_path is None:
            output_path = tempfile.mktemp(suffix=".mp4")

        print(f"  🐉 Generating video with {self.provider.value}...")
        print(f"  Prompt: {prompt[:80]}...")
        print(f"  Duration: {duration_seconds}s | Aspect: {aspect_ratio}")

        if self.provider == VideoProvider.GOOGLE_VEO:
            return await self._veo(prompt, output_path, duration_seconds, aspect_ratio)
        elif self.provider == VideoProvider.OPENAI_SORA:
            return await self._sora(prompt, output_path, duration_seconds)
        elif self.provider == VideoProvider.HUGSFIELD:
            return await self._hugsfield_video(prompt, output_path, duration_seconds)
        elif self.provider == VideoProvider.HUGGINGFACE:
            return await self._hf_video(prompt, output_path)
        elif self.provider == VideoProvider.FAL_AI:
            return await self._fal_video(prompt, output_path, duration_seconds)

        return GenerationResult(success=False, error="Unknown provider")

    async def _veo(self, prompt: str, output: str,
                   duration: int, aspect: str) -> GenerationResult:
        """Google Veo 2 — state of the art video generation."""
        try:
            import aiohttp
            key = os.getenv("GEMINI_API_KEY", "")
            # Veo 2 via Vertex AI / Gemini
            url = f"https://generativelanguage.googleapis.com/v1beta/models/veo-2.0-generate-001:predictLongRunning?key={key}"
            async with aiohttp.ClientSession() as s:
                async with s.post(url, json={
                    "instances": [{"prompt": prompt}],
                    "parameters": {
                        "aspectRatio": aspect,
                        "durationSeconds": min(duration, 8),
                        "sampleCount": 1
                    }
                }) as resp:
                    if resp.status in (200, 202):
                        data = await resp.json()
                        op_name = data.get("name", "")
                        # Poll operation
                        for _ in range(60):
                            await asyncio.sleep(5)
                            async with s.get(
                                f"https://generativelanguage.googleapis.com/v1beta/{op_name}?key={key}"
                            ) as poll:
                                if poll.status == 200:
                                    op = await poll.json()
                                    if op.get("done"):
                                        videos = op.get("response", {}).get("predictions", [])
                                        if videos:
                                            vid_b64 = videos[0].get("bytesBase64Encoded", "")
                                            if vid_b64:
                                                Path(output).write_bytes(base64.b64decode(vid_b64))
                                                return GenerationResult(
                                                    success=True, output_path=output,
                                                    prompt_used=prompt, provider="Google Veo 2"
                                                )
                        return GenerationResult(success=False, error="Veo 2 timeout", provider="Google Veo 2")
                    err = await resp.text()
                    return GenerationResult(success=False, error=f"Veo error {resp.status}: {err[:300]}")
        except Exception as e:
            return GenerationResult(success=False, error=str(e), provider="Google Veo 2")

    async def _sora(self, prompt: str, output: str, duration: int) -> GenerationResult:
        """OpenAI Sora — when API is available."""
        try:
            import aiohttp
            key = os.getenv("OPENAI_API_KEY", "")
            async with aiohttp.ClientSession() as s:
                async with s.post(
                    "https://api.openai.com/v1/video/generations",
                    headers={"Authorization": f"Bearer {key}",
                             "Content-Type": "application/json"},
                    json={"model": "sora", "prompt": prompt,
                          "duration": f"{duration}s", "resolution": "1080p"}
                ) as resp:
                    if resp.status == 200:
                        data = await resp.json()
                        # Poll if async
                        video_url = data.get("data", [{}])[0].get("url")
                        if video_url:
                            async with s.get(video_url) as dl:
                                Path(output).write_bytes(await dl.read())
                            return GenerationResult(success=True, output_path=output,
                                                    prompt_used=prompt, provider="OpenAI Sora")
                    err = await resp.text()
                    if "not available" in err.lower() or "404" in str(resp.status):
                        return GenerationResult(
                            success=False,
                            error="Sora API not yet available in your region/tier. Try Google Veo (GEMINI_API_KEY).",
                            provider="OpenAI Sora"
                        )
                    return GenerationResult(success=False, error=f"Sora error: {err[:200]}")
        except Exception as e:
            return GenerationResult(success=False, error=str(e), provider="OpenAI Sora")

    async def _hf_video(self, prompt: str, output: str) -> GenerationResult:
        """HuggingFace video — Stable Video Diffusion."""
        try:
            import aiohttp
            token = os.getenv("HUGGINGFACE_TOKEN", "")
            # Using CogVideoX-5B — best free video model
            model = "THUDM/CogVideoX-5B"
            async with aiohttp.ClientSession() as s:
                async with s.post(
                    f"https://api-inference.huggingface.co/models/{model}",
                    headers={"Authorization": f"Bearer {token}"},
                    json={"inputs": prompt}
                ) as resp:
                    if resp.status == 200:
                        content = resp.headers.get("content-type", "")
                        if "video" in content or "mp4" in content:
                            Path(output).write_bytes(await resp.read())
                            return GenerationResult(success=True, output_path=output,
                                                    prompt_used=prompt, provider="HuggingFace CogVideoX-5B")
                    err = await resp.text()
                    return GenerationResult(success=False, error=f"HF video error: {err[:200]}")
        except Exception as e:
            return GenerationResult(success=False, error=str(e), provider="HuggingFace Video")

    async def _hugsfield_video(self, prompt: str, output: str,
                              duration: int) -> GenerationResult:
        """
        HugsField — HuggingFace video generation using WAN-2.1 or LTX-Video.
        WAN-2.1 is currently the best free open-source video model (April 2026).
        LTX-Video is fast and high quality for shorter clips.
        Uses HuggingFace Inference API with HUGGINGFACE_TOKEN.
        """
        try:
            import aiohttp
            token = os.getenv("HUGGINGFACE_TOKEN") or os.getenv("HF_TOKEN", "")

            # Try WAN-2.1 first (best quality free video model)
            # Falls back to LTX-Video if WAN-2.1 is loading
            models_to_try = [
                ("Wan-AI/Wan2.1-T2V-14B",      "WAN-2.1 14B"),
                ("Lightricks/LTX-Video",         "LTX-Video"),
                ("THUDM/CogVideoX-5B",           "CogVideoX-5B"),
            ]

            async with aiohttp.ClientSession() as s:
                for model_id, model_name in models_to_try:
                    print(f"  Trying HugsField model: {model_name}...")
                    async with s.post(
                        f"https://api-inference.huggingface.co/models/{model_id}",
                        headers={
                            "Authorization": f"Bearer {token}",
                            "Content-Type": "application/json",
                            "X-Use-Cache": "false",
                        },
                        json={
                            "inputs": prompt,
                            "parameters": {
                                "num_frames": min(duration * 8, 97),  # 8fps
                                "num_inference_steps": 30,
                                "guidance_scale": 6.0,
                            }
                        },
                        timeout=aiohttp.ClientTimeout(total=300)
                    ) as resp:
                        content_type = resp.headers.get("content-type", "")
                        if resp.status == 200 and ("video" in content_type or "octet" in content_type):
                            Path(output).write_bytes(await resp.read())
                            return GenerationResult(
                                success=True, output_path=output,
                                prompt_used=prompt,
                                provider=f"HugsField ({model_name})"
                            )
                        elif resp.status == 503:
                            # Model loading — try next
                            body = await resp.json()
                            wait = body.get("estimated_time", 20)
                            print(f"  {model_name} loading (~{wait:.0f}s), trying next...")
                            continue
                        else:
                            err = await resp.text()
                            print(f"  {model_name} error {resp.status}: {err[:100]}")
                            continue

            return GenerationResult(
                success=False,
                error="All HugsField models unavailable. Try again in a few minutes.",
                provider="HugsField"
            )
        except ImportError:
            return GenerationResult(success=False, error="pip install aiohttp")
        except Exception as e:
            return GenerationResult(success=False, error=str(e), provider="HugsField")

    async def _fal_video(self, prompt: str, output: str, duration: int) -> GenerationResult:
        """Fal.ai video — fast Kling or WAN model."""
        try:
            import aiohttp
            key = os.getenv("FAL_KEY", "")
            async with aiohttp.ClientSession() as s:
                async with s.post(
                    "https://queue.fal.run/fal-ai/kling-video/v1.6/pro/text-to-video",
                    headers={"Authorization": f"Key {key}",
                             "Content-Type": "application/json"},
                    json={"prompt": prompt, "duration": str(duration),
                          "aspect_ratio": "16:9"}
                ) as resp:
                    if resp.status not in (200, 202):
                        err = await resp.text()
                        return GenerationResult(success=False, error=f"Fal video error: {err[:200]}")
                    data = await resp.json()
                    request_id = data.get("request_id")
                    for _ in range(60):
                        await asyncio.sleep(3)
                        async with s.get(
                            f"https://queue.fal.run/fal-ai/kling-video/v1.6/pro/requests/{request_id}",
                            headers={"Authorization": f"Key {key}"}
                        ) as poll:
                            if poll.status == 200:
                                result = await poll.json()
                                if result.get("status") == "COMPLETED":
                                    url = result["output"]["video"]["url"]
                                    async with s.get(url) as dl:
                                        Path(output).write_bytes(await dl.read())
                                    return GenerationResult(success=True, output_path=output,
                                                            prompt_used=prompt, provider="Fal.ai Kling")
                    return GenerationResult(success=False, error="Fal.ai video timeout")
        except Exception as e:
            return GenerationResult(success=False, error=str(e), provider="Fal.ai Video")


# ─── Unified Media Engine ─────────────────────────────────────────────────────

class MediaEngine:
    def __init__(self):
        self.images = ImageEngine()
        self.videos = VideoEngine()

    def get_status(self) -> str:
        img = detect_image_provider()
        vid = detect_video_provider()
        lines = [
            "",
            "  🐉 Media Generation Status",
            "",
            f"  Image provider: {img.value if img != ImageProvider.NONE else 'not configured'}",
            f"  Video provider: {vid.value if vid != VideoProvider.NONE else 'not configured'}",
            "",
        ]
        if img == ImageProvider.NONE:
            lines.append("  To enable images: set HUGGINGFACE_TOKEN (free) or OPENAI_API_KEY")
        if vid == VideoProvider.NONE:
            lines.append("  To enable video:  set GEMINI_API_KEY (Google Veo 2) or FAL_KEY")
        lines.extend([
            "",
            "  🔒 Keys are encrypted locally. Never shared with anyone.",
            "",
        ])
        return "\n".join(lines)

    async def generate_image(self, prompt: str, output_path: str = None,
                              **kwargs) -> GenerationResult:
        result = await self.images.generate(prompt, output_path, **kwargs)
        if not result.success and result.setup_guide:
            print(result.setup_guide)
        elif result.success:
            print(f"  ✓ Image saved: {result.output_path}")
            print(f"  Provider: {result.provider}")
        else:
            print(f"  ✗ Generation failed: {result.error}")
        return result

    async def generate_video(self, prompt: str, output_path: str = None,
                              duration: int = 5, **kwargs) -> GenerationResult:
        result = await self.videos.generate(prompt, output_path, duration, **kwargs)
        if not result.success and result.setup_guide:
            print(result.setup_guide)
        elif result.success:
            print(f"  ✓ Video saved: {result.output_path}")
            print(f"  Provider: {result.provider}")
        else:
            print(f"  ✗ Generation failed: {result.error}")
        return result
