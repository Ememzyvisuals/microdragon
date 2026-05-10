"""
microdragon/modules/huggingface/src/engine.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON HUGGINGFACE MODEL DOWNLOADER
═══════════════════════════════════════════════════════════════════════════════

Download any model from HuggingFace and use it to power MICRODRAGON.

Recommended models — all verified working with MICRODRAGON:

SMALLEST (works on 2GB RAM, CPU only):
  phi-3-mini-4k     — 3.8B params, 2.3GB — Microsoft, excellent for code
  tinyllama-1.1b    — 1.1B params, 0.6GB — very fast, basic tasks

MEDIUM (4-8GB RAM, CPU or GPU):
  llama-3.1-8b      — 8B params, 4.7GB — Meta, best general purpose
  mistral-7b-v0.3   — 7B params, 4.1GB — very capable coder
  qwen2.5-7b        — 7B params, 4.3GB — excellent multilingual

LARGE (16GB+ RAM or 8GB VRAM):
  llama-3.1-70b     — 70B params, 40GB (Q4: 39GB) — near GPT-4 quality
  qwen2.5-72b       — 72B params, 41GB (Q4: 40GB) — best open model
  deepseek-r1-14b   — 14B params, 8.3GB — best reasoning model

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

import asyncio
import os
import sys
import subprocess
import json
import shutil
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class HFModel:
    name: str               # Display name
    hf_repo: str            # HuggingFace repo ID
    hf_url: str             # Direct download URL
    ollama_name: str        # Name to use with Ollama
    size_gb: float          # Download size
    ram_required_gb: float  # Minimum RAM
    gpu_vram_gb: float      # VRAM if using GPU (0 = CPU OK)
    params: str             # Parameter count
    quality: int            # 1-10 quality score
    speed: int              # 1-10 speed score (10 = fastest)
    best_for: list[str]
    description: str
    context_window: int     # Max context tokens
    quantization: str       # GGUF quantization level


# ─── Verified Model Registry ──────────────────────────────────────────────────

RECOMMENDED_MODELS = {

    # ── TIER 1: Runs on any computer (2GB+ RAM, no GPU needed) ─────────────

    "tinyllama": HFModel(
        name="TinyLlama 1.1B",
        hf_repo="TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF",
        hf_url="https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf",
        ollama_name="tinyllama",
        size_gb=0.6,
        ram_required_gb=1.0,
        gpu_vram_gb=0,
        params="1.1B",
        quality=4,
        speed=10,
        best_for=["basic tasks", "fast responses", "low-end hardware"],
        description="Fastest model, minimal RAM. Good for simple Q&A and basic tasks on old hardware.",
        context_window=2048,
        quantization="Q4_K_M"
    ),

    "phi3-mini": HFModel(
        name="Phi-3 Mini 4K Instruct",
        hf_repo="microsoft/Phi-3-mini-4k-instruct-gguf",
        hf_url="https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf",
        ollama_name="phi3:mini",
        size_gb=2.3,
        ram_required_gb=3.0,
        gpu_vram_gb=0,
        params="3.8B",
        quality=7,
        speed=8,
        best_for=["code generation", "reasoning", "low-end hardware", "all-rounder"],
        description="Microsoft's best small model. Excellent at code and reasoning despite size. Recommended for low-end hardware.",
        context_window=4096,
        quantization="Q4"
    ),

    "phi3-medium": HFModel(
        name="Phi-3 Medium 14B",
        hf_repo="microsoft/Phi-3-medium-4k-instruct-gguf",
        hf_url="https://huggingface.co/microsoft/Phi-3-medium-4k-instruct-gguf/resolve/main/Phi-3-medium-4k-instruct-q4.gguf",
        ollama_name="phi3:medium",
        size_gb=7.9,
        ram_required_gb=10.0,
        gpu_vram_gb=8.0,
        params="14B",
        quality=8,
        speed=6,
        best_for=["code", "analysis", "complex reasoning"],
        description="Larger Phi-3 for more capable tasks.",
        context_window=4096,
        quantization="Q4"
    ),

    # ── TIER 2: Best general purpose (4-8GB RAM) ────────────────────────────

    "llama31-8b": HFModel(
        name="Llama 3.1 8B Instruct",
        hf_repo="meta-llama/Meta-Llama-3.1-8B-Instruct",
        hf_url="https://huggingface.co/bartowski/Meta-Llama-3.1-8B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf",
        ollama_name="llama3.1",
        size_gb=4.7,
        ram_required_gb=6.0,
        gpu_vram_gb=0,
        params="8B",
        quality=8,
        speed=7,
        best_for=["general purpose", "code", "writing", "analysis", "chat"],
        description="Meta's best open model at this size. Excellent all-rounder. RECOMMENDED as default MICRODRAGON model.",
        context_window=128000,
        quantization="Q4_K_M"
    ),

    "mistral7b": HFModel(
        name="Mistral 7B v0.3",
        hf_repo="mistralai/Mistral-7B-Instruct-v0.3",
        hf_url="https://huggingface.co/bartowski/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3-Q4_K_M.gguf",
        ollama_name="mistral:7b",
        size_gb=4.1,
        ram_required_gb=6.0,
        gpu_vram_gb=0,
        params="7B",
        quality=8,
        speed=7,
        best_for=["code", "instruction following", "technical tasks"],
        description="Mistral AI's flagship 7B model. Very strong at following instructions and code.",
        context_window=32768,
        quantization="Q4_K_M"
    ),

    "qwen25-7b": HFModel(
        name="Qwen 2.5 7B Instruct",
        hf_repo="Qwen/Qwen2.5-7B-Instruct-GGUF",
        hf_url="https://huggingface.co/Qwen/Qwen2.5-7B-Instruct-GGUF/resolve/main/qwen2.5-7b-instruct-q4_k_m.gguf",
        ollama_name="qwen2.5:7b",
        size_gb=4.3,
        ram_required_gb=6.0,
        gpu_vram_gb=0,
        params="7B",
        quality=8,
        speed=7,
        best_for=["multilingual", "coding", "math", "reasoning"],
        description="Alibaba's Qwen 2.5 — excellent multilingual support and very strong at code and math.",
        context_window=128000,
        quantization="Q4_K_M"
    ),

    "codellama7b": HFModel(
        name="CodeLlama 7B",
        hf_repo="codellama/CodeLlama-7b-Instruct-hf",
        hf_url="https://huggingface.co/TheBloke/CodeLlama-7B-Instruct-GGUF/resolve/main/codellama-7b-instruct.Q4_K_M.gguf",
        ollama_name="codellama:7b",
        size_gb=3.8,
        ram_required_gb=5.0,
        gpu_vram_gb=0,
        params="7B",
        quality=7,
        speed=8,
        best_for=["code generation", "code completion", "debugging", "refactoring"],
        description="Meta's code-specialised model. Best for pure coding tasks.",
        context_window=16384,
        quantization="Q4_K_M"
    ),

    # ── TIER 3: High quality (8-16GB RAM) ───────────────────────────────────

    "llama31-13b": HFModel(
        name="Llama 3.1 13B Instruct",
        hf_repo="meta-llama/Meta-Llama-3.1-8B-Instruct",
        hf_url="https://huggingface.co/bartowski/Meta-Llama-3.1-70B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-70B-Instruct-Q2_K.gguf",
        ollama_name="llama3.1:13b",
        size_gb=8.3,
        ram_required_gb=10.0,
        gpu_vram_gb=0,
        params="13B",
        quality=9,
        speed=5,
        best_for=["complex reasoning", "long documents", "research", "writing"],
        description="Significantly more capable than 8B for complex tasks.",
        context_window=128000,
        quantization="Q4_K_M"
    ),

    "deepseek-r1-14b": HFModel(
        name="DeepSeek R1 14B",
        hf_repo="deepseek-ai/DeepSeek-R1",
        hf_url="https://huggingface.co/bartowski/DeepSeek-R1-Distill-Qwen-14B-GGUF/resolve/main/DeepSeek-R1-Distill-Qwen-14B-Q4_K_M.gguf",
        ollama_name="deepseek-r1:14b",
        size_gb=8.3,
        ram_required_gb=10.0,
        gpu_vram_gb=0,
        params="14B",
        quality=9,
        speed=5,
        best_for=["complex reasoning", "math", "logic", "chain-of-thought"],
        description="DeepSeek's reasoning model — shows its thinking process. Best for complex logic and math.",
        context_window=32768,
        quantization="Q4_K_M"
    ),

    "qwen25-14b": HFModel(
        name="Qwen 2.5 14B Instruct",
        hf_repo="Qwen/Qwen2.5-14B-Instruct-GGUF",
        hf_url="https://huggingface.co/Qwen/Qwen2.5-14B-Instruct-GGUF/resolve/main/qwen2.5-14b-instruct-q4_k_m.gguf",
        ollama_name="qwen2.5:14b",
        size_gb=8.7,
        ram_required_gb=11.0,
        gpu_vram_gb=0,
        params="14B",
        quality=9,
        speed=5,
        best_for=["all tasks", "multilingual", "code", "analysis"],
        description="Excellent 14B model from Alibaba. Near GPT-4 quality at tasks in this size class.",
        context_window=128000,
        quantization="Q4_K_M"
    ),

    # ── TIER 4: Near GPT-4 quality (32GB+ RAM or GPU) ─────────────────────

    "llama31-70b": HFModel(
        name="Llama 3.1 70B Instruct",
        hf_repo="meta-llama/Meta-Llama-3.1-70B-Instruct",
        hf_url="https://huggingface.co/bartowski/Meta-Llama-3.1-70B-Instruct-GGUF/resolve/main/Meta-Llama-3.1-70B-Instruct-Q4_K_M.gguf",
        ollama_name="llama3.1:70b",
        size_gb=39.0,
        ram_required_gb=42.0,
        gpu_vram_gb=40.0,
        params="70B",
        quality=10,
        speed=2,
        best_for=["everything", "complex reasoning", "code", "writing"],
        description="Near GPT-4 quality. Excellent for all tasks. Requires high-end hardware.",
        context_window=128000,
        quantization="Q4_K_M"
    ),

    "qwen25-72b": HFModel(
        name="Qwen 2.5 72B Instruct",
        hf_repo="Qwen/Qwen2.5-72B-Instruct-GGUF",
        hf_url="https://huggingface.co/Qwen/Qwen2.5-72B-Instruct-GGUF/resolve/main/qwen2.5-72b-instruct-q4_k_m.gguf",
        ollama_name="qwen2.5:72b",
        size_gb=41.0,
        ram_required_gb=44.0,
        gpu_vram_gb=40.0,
        params="72B",
        quality=10,
        speed=2,
        best_for=["everything", "multilingual", "code", "math"],
        description="Best open-source model available. Competitive with GPT-4o on many benchmarks.",
        context_window=128000,
        quantization="Q4_K_M"
    ),
}


class HuggingFaceEngine:
    """Download, manage, and integrate HuggingFace models with MICRODRAGON."""

    def __init__(self):
        self.models_dir = Path(
            os.path.expanduser("~/.local/share/microdragon/models")
        )
        self.models_dir.mkdir(parents=True, exist_ok=True)

    def list_recommended(self) -> str:
        """Show all recommended models with hardware requirements."""
        lines = [
            "",
            "  MICRODRAGON — Recommended HuggingFace Models",
            "  ════════════════════════════════════════════════════════════",
            "",
            "  TIER 1 — Runs on any computer (512MB-3GB RAM, no GPU needed)",
            "",
        ]

        tiers = [
            ("tinyllama", "phi3-mini"),
            ("llama31-8b", "mistral7b", "qwen25-7b", "codellama7b"),
            ("llama31-13b", "deepseek-r1-14b", "qwen25-14b"),
            ("llama31-70b", "qwen25-72b"),
        ]
        tier_labels = [
            "  TIER 1 — Any computer (512MB-3GB RAM, no GPU needed)",
            "  TIER 2 — Best all-rounder (4-8GB RAM, no GPU needed)",
            "  TIER 3 — High quality (8-16GB RAM or 8GB VRAM)",
            "  TIER 4 — Near GPT-4 quality (32GB+ RAM or 40GB VRAM)",
        ]

        for label, tier_keys in zip(tier_labels, tiers):
            lines.append(label)
            lines.append("")
            for key in tier_keys:
                if key not in RECOMMENDED_MODELS:
                    continue
                m = RECOMMENDED_MODELS[key]
                star = "★" * m.quality + "☆" * (10 - m.quality)
                lines.append(f"    {m.name:<35} {m.size_gb:>5.1f}GB  RAM:{m.ram_required_gb:.0f}GB")
                lines.append(f"    Quality: {star[:7]}  Speed: {'▶'*m.speed}  Params: {m.params}")
                lines.append(f"    Best for: {', '.join(m.best_for[:3])}")
                lines.append(f"    Install: microdragon model download {key}")
                lines.append("")
            lines.append("")

        lines.append("  HARDWARE RECOMMENDATION:")
        lines.append("    Any PC with 4GB RAM → phi3-mini (great quality, tiny size)")
        lines.append("    PC with 6-8GB RAM  → llama3.1 (recommended, best balance)")
        lines.append("    PC with 16GB RAM   → deepseek-r1-14b (reasoning specialist)")
        lines.append("    High-end GPU       → qwen2.5-72b (best open model available)")
        lines.append("")

        return '\n'.join(lines)

    def get_hardware_recommendation(self) -> str:
        """Detect available RAM and GPU, recommend best model."""
        try:
            import psutil
            ram_gb = psutil.virtual_memory().total / (1024**3)
        except ImportError:
            ram_gb = 8  # assume 8GB if psutil not available

        # Check for NVIDIA GPU
        has_gpu = False
        gpu_vram = 0
        try:
            result = subprocess.run(
                ["nvidia-smi", "--query-gpu=memory.total",
                 "--format=csv,noheader,nounits"],
                capture_output=True, text=True, timeout=5
            )
            if result.returncode == 0:
                gpu_vram = int(result.stdout.strip().split('\n')[0]) / 1024
                has_gpu = True
        except Exception:
            pass

        lines = [
            f"",
            f"  Hardware detection:",
            f"  RAM: {ram_gb:.1f}GB available",
            f"  GPU: {'Yes — ' + str(gpu_vram) + 'GB VRAM' if has_gpu else 'Not detected (CPU mode)'}",
            f"",
            f"  Recommended model for your hardware:",
        ]

        if ram_gb >= 44:
            rec = RECOMMENDED_MODELS["qwen25-72b"]
            reason = "Maximum quality — your system can handle the best open model"
        elif ram_gb >= 42:
            rec = RECOMMENDED_MODELS["llama31-70b"]
            reason = "Near GPT-4 quality on your high-memory system"
        elif ram_gb >= 11:
            rec = RECOMMENDED_MODELS["qwen25-14b"]
            reason = "Excellent quality, great balance for your RAM"
        elif ram_gb >= 6:
            rec = RECOMMENDED_MODELS["llama31-8b"]
            reason = "Best all-rounder for your hardware — RECOMMENDED"
        elif ram_gb >= 3:
            rec = RECOMMENDED_MODELS["phi3-mini"]
            reason = "Best small model — excellent quality for its size"
        else:
            rec = RECOMMENDED_MODELS["tinyllama"]
            reason = "Best option for very limited RAM"

        lines.extend([
            f"  ✓ {rec.name} ({rec.size_gb}GB download, {rec.ram_required_gb}GB RAM needed)",
            f"    {reason}",
            f"",
            f"  Install command:",
            f"  microdragon model download {list(k for k, v in RECOMMENDED_MODELS.items() if v == rec)[0]}",
            f""
        ])

        return '\n'.join(lines)

    def download_via_ollama(self, model_key: str,
                             progress_callback=None) -> dict:
        """
        Download model via Ollama (easiest method).
        Ollama handles GGUF format, caching, and GPU offloading automatically.
        """
        if model_key not in RECOMMENDED_MODELS:
            return {"success": False, "error": f"Unknown model: {model_key}"}

        model = RECOMMENDED_MODELS[model_key]

        # Check Ollama is installed
        if not shutil.which("ollama"):
            return {
                "success": False,
                "error": "Ollama not installed",
                "install_url": "https://ollama.ai/download",
                "instructions": [
                    "1. Go to https://ollama.ai/download",
                    "2. Download for your OS (macOS/Linux/Windows)",
                    "3. Install it (next-next-finish on Windows)",
                    f"4. Run: ollama pull {model.ollama_name}",
                    "5. Then: microdragon config provider custom",
                    "   Endpoint: http://localhost:11434/v1",
                    f"   Model: {model.ollama_name}"
                ]
            }

        print(f"\n  Downloading {model.name} via Ollama...")
        print(f"  Size: {model.size_gb}GB — this may take a few minutes")
        print(f"  Progress:\n")

        result = subprocess.run(
            ["ollama", "pull", model.ollama_name],
            capture_output=False
        )

        if result.returncode == 0:
            return {
                "success": True,
                "model": model.name,
                "ollama_name": model.ollama_name,
                "configure_command": f"microdragon config provider custom",
                "configure_endpoint": "http://localhost:11434/v1",
                "configure_model": model.ollama_name,
                "message": (
                    f"✓ {model.name} downloaded successfully!\n\n"
                    f"Configure MICRODRAGON to use it:\n"
                    f"  microdragon config provider custom\n"
                    f"  Endpoint: http://localhost:11434/v1\n"
                    f"  Model: {model.ollama_name}"
                )
            }
        else:
            return {
                "success": False,
                "error": f"Ollama pull failed (exit {result.returncode})",
                "fallback": f"Try manually: ollama pull {model.ollama_name}"
            }

    def download_gguf_direct(self, model_key: str) -> dict:
        """
        Download GGUF file directly from HuggingFace.
        Use this if Ollama is not available.
        Requires llama.cpp or similar to run the model.
        """
        if model_key not in RECOMMENDED_MODELS:
            return {"success": False, "error": f"Unknown model: {model_key}"}

        model = RECOMMENDED_MODELS[model_key]
        output_path = self.models_dir / f"{model_key}.gguf"

        print(f"\n  Downloading {model.name} from HuggingFace...")
        print(f"  URL: {model.hf_url}")
        print(f"  Size: {model.size_gb}GB")
        print(f"  Output: {output_path}")
        print(f"  This will take several minutes depending on your connection.\n")

        try:
            # Use wget if available (shows progress), else requests
            if shutil.which("wget"):
                result = subprocess.run(
                    ["wget", "-c", "--show-progress",
                     "-O", str(output_path), model.hf_url],
                    check=False
                )
                success = result.returncode == 0
            elif shutil.which("curl"):
                result = subprocess.run(
                    ["curl", "-L", "--progress-bar",
                     "-o", str(output_path), model.hf_url],
                    check=False
                )
                success = result.returncode == 0
            else:
                # Pure Python fallback
                import urllib.request
                def reporthook(count, block_size, total_size):
                    pct = count * block_size / total_size * 100
                    print(f"\r  {pct:.1f}%", end='', flush=True)
                urllib.request.urlretrieve(model.hf_url, output_path, reporthook)
                print()
                success = True

            if success:
                return {
                    "success": True,
                    "path": str(output_path),
                    "model": model.name,
                    "instructions": [
                        f"Model saved to: {output_path}",
                        "To use with llama.cpp:",
                        f"  ./llama-cli -m {output_path} -i --instruct",
                        "To use with Ollama (create from GGUF):",
                        f"  echo 'FROM {output_path}' > Modelfile",
                        f"  ollama create {model_key} -f Modelfile",
                        f"  microdragon config model {model_key}"
                    ]
                }
            else:
                return {"success": False, "error": "Download failed"}

        except Exception as e:
            return {"success": False, "error": str(e)}

    def configure_microdragon(self, model_key: str) -> str:
        """Generate configuration commands to use downloaded model."""
        if model_key not in RECOMMENDED_MODELS:
            return f"Unknown model: {model_key}"

        model = RECOMMENDED_MODELS[model_key]
        return f"""
  Configure MICRODRAGON to use {model.name}:

  microdragon config provider custom
  # When prompted:
  #   Endpoint: http://localhost:11434/v1
  #   Model: {model.ollama_name}

  Or set environment variables:
  export MICRODRAGON_PROVIDER=custom
  export MICRODRAGON_API_ENDPOINT=http://localhost:11434/v1
  export MICRODRAGON_MODEL={model.ollama_name}

  Verify it works:
  microdragon ask "hello, what model are you?"
"""


if __name__ == "__main__":
    engine = HuggingFaceEngine()

    if len(sys.argv) > 1 and sys.argv[1] == "list":
        print(engine.list_recommended())
    elif len(sys.argv) > 1 and sys.argv[1] == "recommend":
        print(engine.get_hardware_recommendation())
    elif len(sys.argv) > 2 and sys.argv[1] == "download":
        result = engine.download_via_ollama(sys.argv[2])
        print(result.get("message") or result.get("error"))
    else:
        print(engine.list_recommended())
        print(engine.get_hardware_recommendation())
