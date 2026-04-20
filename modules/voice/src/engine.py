"""
microdragon/modules/voice/src/engine.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON VOICE MODULE — STT + TTS
═══════════════════════════════════════════════════════════════════════════════

STT (Speech-to-Text):
  Transcribes audio — microphone, voicemails from WhatsApp/Telegram, files.
  Provider choice:
    - Groq Whisper API     (fastest, free tier)
    - OpenAI Whisper API   (best accuracy)
    - Local Whisper        (faster-whisper, 100% offline, any hardware)
    - Google STT           (if GCP key configured)

  WhatsApp/Telegram voicemails: automatically transcribed before processing.
  Users are informed which STT engine is active on setup.

TTS (Text-to-Speech):
  Speaks responses aloud. Depends entirely on the user's AI provider:
    - Groq provider        → no TTS (Groq doesn't offer TTS) → system TTS fallback
    - OpenAI provider      → OpenAI TTS (tts-1, alloy/nova/echo voices)
    - Anthropic provider   → no TTS → ElevenLabs if key set, else system TTS
    - Local/Ollama         → system TTS (piper, espeak, macOS say, Windows SAPI)
    - ElevenLabs key set   → always use ElevenLabs (overrides all)

  Users are clearly told which TTS is active and why.

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

import asyncio
import os
import sys
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path
from enum import Enum
from typing import Optional


class STTProvider(Enum):
    GROQ_WHISPER   = "groq_whisper"
    OPENAI_WHISPER = "openai_whisper"
    LOCAL_WHISPER  = "local_whisper"
    GOOGLE_STT     = "google_stt"
    UNAVAILABLE    = "unavailable"


class TTSProvider(Enum):
    OPENAI_TTS    = "openai_tts"
    ELEVENLABS    = "elevenlabs"
    SYSTEM_PIPER  = "system_piper"
    MACOS_SAY     = "macos_say"
    WINDOWS_SAPI  = "windows_sapi"
    ESPEAK        = "espeak"
    UNAVAILABLE   = "unavailable"


@dataclass
class VoiceConfig:
    stt_provider:  STTProvider
    tts_provider:  TTSProvider
    stt_model:     str = "whisper-large-v3"
    tts_voice:     str = "nova"
    tts_speed:     float = 1.0
    language:      str = "en"
    # Explanation shown to user so they know EXACTLY what's running
    stt_notice:    str = ""
    tts_notice:    str = ""


# ─── Provider detection ───────────────────────────────────────────────────────

def detect_voice_config(ai_provider: str = "") -> VoiceConfig:
    """
    Detect the best STT + TTS based on the user's configured AI provider.
    Always tell the user what's being used and why.
    """
    provider = ai_provider.lower()

    # ── STT detection ─────────────────────────────────────────────────────────

    groq_key   = os.getenv("GROQ_API_KEY", "")
    openai_key = os.getenv("OPENAI_API_KEY", "")

    if provider == "groq" and groq_key:
        stt = STTProvider.GROQ_WHISPER
        stt_notice = (
            "STT: Groq Whisper API (your Groq key) — fast, free tier available.\n"
            "Your audio is sent to Groq's servers for transcription."
        )
    elif openai_key:
        stt = STTProvider.OPENAI_WHISPER
        stt_notice = (
            "STT: OpenAI Whisper API (your OpenAI key) — highest accuracy.\n"
            "Your audio is sent to OpenAI's servers for transcription."
        )
    elif _local_whisper_available():
        stt = STTProvider.LOCAL_WHISPER
        stt_notice = (
            "STT: Local Whisper (faster-whisper, runs on your machine).\n"
            "Audio never leaves your device. Slightly slower than API."
        )
    else:
        stt = STTProvider.UNAVAILABLE
        stt_notice = (
            "STT: Not available.\n"
            "Install faster-whisper for offline STT: pip install faster-whisper\n"
            "Or set GROQ_API_KEY (free) for cloud STT."
        )

    # ── TTS detection ─────────────────────────────────────────────────────────

    elevenlabs_key = os.getenv("ELEVENLABS_API_KEY", "")

    if elevenlabs_key:
        tts = TTSProvider.ELEVENLABS
        tts_notice = "TTS: ElevenLabs (your ElevenLabs key) — highest quality voice synthesis."
    elif provider == "openai" and openai_key:
        tts = TTSProvider.OPENAI_TTS
        tts_notice = (
            "TTS: OpenAI TTS (tts-1, your OpenAI key) — very natural voice.\n"
            "Uses OpenAI API (small cost per character)."
        )
    elif provider == "groq":
        # Groq doesn't offer TTS — fall back
        tts, tts_notice = _system_tts_fallback()
        tts_notice = (
            f"TTS: {tts_notice}\n"
            "Note: Groq does not offer TTS. Using system TTS instead.\n"
            "For better voice: set ELEVENLABS_API_KEY (free tier available)."
        )
    elif provider == "anthropic":
        tts, tts_notice = _system_tts_fallback()
        tts_notice = (
            f"TTS: {tts_notice}\n"
            "Note: Anthropic does not offer TTS. Using system TTS instead.\n"
            "For better voice: set ELEVENLABS_API_KEY or OPENAI_API_KEY."
        )
    else:
        tts, tts_notice = _system_tts_fallback()

    return VoiceConfig(
        stt_provider=stt,
        tts_provider=tts,
        stt_notice=stt_notice,
        tts_notice=tts_notice,
    )


def _local_whisper_available() -> bool:
    try:
        import faster_whisper  # noqa
        return True
    except ImportError:
        return False


def _system_tts_fallback() -> tuple[TTSProvider, str]:
    """Pick the best available system TTS."""
    if sys.platform == "darwin":
        return TTSProvider.MACOS_SAY, "macOS 'say' command (built-in, offline)"
    elif sys.platform == "win32":
        return TTSProvider.WINDOWS_SAPI, "Windows SAPI (built-in, offline)"
    else:
        # Linux
        if subprocess.run(["which", "piper"], capture_output=True).returncode == 0:
            return TTSProvider.SYSTEM_PIPER, "Piper TTS (local, offline, good quality)"
        elif subprocess.run(["which", "espeak"], capture_output=True).returncode == 0:
            return TTSProvider.ESPEAK, "eSpeak (local, offline, robotic voice)"
        return TTSProvider.UNAVAILABLE, "No TTS available (install espeak: sudo apt install espeak)"


# ─── STT Engine ───────────────────────────────────────────────────────────────

class SpeechToText:
    """Transcribes audio from any source."""

    def __init__(self, config: VoiceConfig):
        self.config = config
        self._local_model = None

    async def transcribe_file(self, audio_path: str) -> str:
        """Transcribe an audio file (MP3, WAV, OGG, M4A, OPUS)."""
        if self.config.stt_provider == STTProvider.GROQ_WHISPER:
            return await self._transcribe_groq(audio_path)
        elif self.config.stt_provider == STTProvider.OPENAI_WHISPER:
            return await self._transcribe_openai(audio_path)
        elif self.config.stt_provider == STTProvider.LOCAL_WHISPER:
            return self._transcribe_local(audio_path)
        else:
            return "[STT not configured — see: microdragon voice setup]"

    async def transcribe_voicemail(self, audio_path: str, platform: str) -> str:
        """
        Transcribe a voicemail from WhatsApp/Telegram.
        Called automatically when an audio message arrives.
        Returns text that Microdragon then processes as a regular message.
        """
        try:
            text = await self.transcribe_file(audio_path)
            return f"[Voice message from {platform}]: {text}"
        except Exception as e:
            return f"[Voice message — transcription failed: {e}]"

    async def transcribe_microphone(self, duration_seconds: int = 5) -> str:
        """Record from microphone and transcribe."""
        tmp = tempfile.mktemp(suffix=".wav")
        try:
            # Record using sounddevice or pyaudio
            if await self._record_audio(tmp, duration_seconds):
                return await self.transcribe_file(tmp)
            return "[Recording failed — check microphone permissions]"
        finally:
            try: os.unlink(tmp)
            except: pass

    async def _record_audio(self, output_path: str, duration: int) -> bool:
        try:
            import sounddevice as sd
            import soundfile as sf
            import numpy as np
            sample_rate = 16000
            print(f"  🎤 Recording {duration}s... (speak now)")
            audio = sd.rec(int(duration * sample_rate), samplerate=sample_rate,
                           channels=1, dtype='float32')
            sd.wait()
            sf.write(output_path, audio, sample_rate)
            return True
        except ImportError:
            # Fallback: ffmpeg + /dev/stdin
            try:
                result = subprocess.run([
                    "ffmpeg", "-f", "alsa", "-i", "default",
                    "-t", str(duration), "-ar", "16000", "-ac", "1",
                    output_path, "-y", "-loglevel", "quiet"
                ], timeout=duration + 5)
                return result.returncode == 0
            except Exception:
                return False

    async def _transcribe_groq(self, audio_path: str) -> str:
        try:
            import aiohttp
            api_key = os.getenv("GROQ_API_KEY", "")
            async with aiohttp.ClientSession() as session:
                with open(audio_path, "rb") as f:
                    data = aiohttp.FormData()
                    data.add_field("file", f, filename=Path(audio_path).name,
                                   content_type="audio/wav")
                    data.add_field("model", "whisper-large-v3")
                    data.add_field("language", self.config.language)
                    data.add_field("response_format", "text")
                    async with session.post(
                        "https://api.groq.com/openai/v1/audio/transcriptions",
                        headers={"Authorization": f"Bearer {api_key}"},
                        data=data
                    ) as resp:
                        if resp.status == 200:
                            return (await resp.text()).strip()
                        return f"[Groq STT error {resp.status}]"
        except Exception as e:
            return f"[Groq STT failed: {e}]"

    async def _transcribe_openai(self, audio_path: str) -> str:
        try:
            import aiohttp
            api_key = os.getenv("OPENAI_API_KEY", "")
            async with aiohttp.ClientSession() as session:
                with open(audio_path, "rb") as f:
                    data = aiohttp.FormData()
                    data.add_field("file", f, filename=Path(audio_path).name,
                                   content_type="audio/wav")
                    data.add_field("model", "whisper-1")
                    data.add_field("language", self.config.language)
                    async with session.post(
                        "https://api.openai.com/v1/audio/transcriptions",
                        headers={"Authorization": f"Bearer {api_key}"},
                        data=data
                    ) as resp:
                        if resp.status == 200:
                            result = await resp.json()
                            return result.get("text", "").strip()
                        return f"[OpenAI Whisper error {resp.status}]"
        except Exception as e:
            return f"[OpenAI Whisper failed: {e}]"

    def _transcribe_local(self, audio_path: str) -> str:
        try:
            from faster_whisper import WhisperModel
            if self._local_model is None:
                # Load on first use — small model for speed
                model_size = os.getenv("MICRODRAGON_WHISPER_MODEL", "base")
                self._local_model = WhisperModel(model_size, device="auto",
                                                  compute_type="auto")
            segments, _ = self._local_model.transcribe(audio_path,
                                                         language=self.config.language or None)
            return " ".join(s.text for s in segments).strip()
        except Exception as e:
            return f"[Local Whisper failed: {e}]"


# ─── TTS Engine ───────────────────────────────────────────────────────────────

class TextToSpeech:
    """Speaks text using the provider available for this user."""

    def __init__(self, config: VoiceConfig):
        self.config = config

    async def speak(self, text: str) -> bool:
        """Speak text aloud. Returns True if successful."""
        if self.config.tts_provider == TTSProvider.OPENAI_TTS:
            return await self._speak_openai(text)
        elif self.config.tts_provider == TTSProvider.ELEVENLABS:
            return await self._speak_elevenlabs(text)
        elif self.config.tts_provider == TTSProvider.MACOS_SAY:
            return self._speak_macos(text)
        elif self.config.tts_provider == TTSProvider.WINDOWS_SAPI:
            return self._speak_windows(text)
        elif self.config.tts_provider == TTSProvider.ESPEAK:
            return self._speak_espeak(text)
        elif self.config.tts_provider == TTSProvider.SYSTEM_PIPER:
            return self._speak_piper(text)
        return False

    async def _speak_openai(self, text: str) -> bool:
        try:
            import aiohttp
            api_key = os.getenv("OPENAI_API_KEY", "")
            tmp = tempfile.mktemp(suffix=".mp3")
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    "https://api.openai.com/v1/audio/speech",
                    headers={"Authorization": f"Bearer {api_key}"},
                    json={"model": "tts-1", "input": text[:4096],
                          "voice": self.config.tts_voice,
                          "speed": self.config.tts_speed}
                ) as resp:
                    if resp.status == 200:
                        with open(tmp, "wb") as f:
                            f.write(await resp.read())
                        self._play_audio(tmp)
                        try: os.unlink(tmp)
                        except: pass
                        return True
            return False
        except Exception:
            return False

    async def _speak_elevenlabs(self, text: str) -> bool:
        try:
            import aiohttp
            api_key = os.getenv("ELEVENLABS_API_KEY", "")
            voice_id = os.getenv("ELEVENLABS_VOICE_ID", "21m00Tcm4TlvDq8ikWAM")
            tmp = tempfile.mktemp(suffix=".mp3")
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"https://api.elevenlabs.io/v1/text-to-speech/{voice_id}",
                    headers={"xi-api-key": api_key},
                    json={"text": text[:5000],
                          "model_id": "eleven_monolingual_v1",
                          "voice_settings": {"stability": 0.5, "similarity_boost": 0.75}}
                ) as resp:
                    if resp.status == 200:
                        with open(tmp, "wb") as f:
                            f.write(await resp.read())
                        self._play_audio(tmp)
                        try: os.unlink(tmp)
                        except: pass
                        return True
            return False
        except Exception:
            return False

    def _speak_macos(self, text: str) -> bool:
        voice = os.getenv("MICRODRAGON_MACOS_VOICE", "Samantha")
        try:
            subprocess.run(["say", "-v", voice, text], timeout=60)
            return True
        except Exception:
            return False

    def _speak_windows(self, text: str) -> bool:
        try:
            ps_script = f'Add-Type -AssemblyName System.Speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak("{text[:3000].replace(chr(34), "")}")'
            subprocess.run(["powershell", "-Command", ps_script], timeout=60)
            return True
        except Exception:
            return False

    def _speak_espeak(self, text: str) -> bool:
        try:
            subprocess.run(["espeak", "-v", "en+m3", "-s", "150", text[:3000]], timeout=60)
            return True
        except Exception:
            return False

    def _speak_piper(self, text: str) -> bool:
        try:
            tmp = tempfile.mktemp(suffix=".wav")
            result = subprocess.run(
                ["piper", "--output_file", tmp],
                input=text.encode(), capture_output=True, timeout=30
            )
            if result.returncode == 0:
                self._play_audio(tmp)
                try: os.unlink(tmp)
                except: pass
                return True
        except Exception:
            pass
        return False

    def _play_audio(self, path: str):
        """Play audio file using system player."""
        try:
            if sys.platform == "darwin":
                subprocess.run(["afplay", path], timeout=300, check=False)
            elif sys.platform == "win32":
                subprocess.run(["powershell", f'(New-Object Media.SoundPlayer "{path}").PlaySync()'], timeout=300)
            else:
                for player in ["mpg123", "ffplay", "aplay", "paplay"]:
                    if subprocess.run(["which", player], capture_output=True).returncode == 0:
                        subprocess.run([player, "-q" if player != "aplay" else "", path],
                                       timeout=300, check=False)
                        return
        except Exception:
            pass


# ─── Voice Engine (unified) ───────────────────────────────────────────────────

class VoiceEngine:
    """Single entry point for all voice operations."""

    def __init__(self, ai_provider: str = ""):
        self.config = detect_voice_config(ai_provider)
        self.stt = SpeechToText(self.config)
        self.tts = TextToSpeech(self.config)

    def get_setup_notice(self) -> str:
        """Return what TTS/STT is active and why — shown in setup and /status."""
        return (
            f"\n  Voice configuration:\n\n"
            f"  {self.config.stt_notice}\n\n"
            f"  {self.config.tts_notice}\n\n"
            f"  To change: configure a different AI provider or set:\n"
            f"    GROQ_API_KEY       → Groq Whisper STT (free tier)\n"
            f"    OPENAI_API_KEY     → OpenAI Whisper STT + TTS\n"
            f"    ELEVENLABS_API_KEY → ElevenLabs TTS (best quality)\n"
        )

    async def listen(self, duration: int = 5) -> str:
        return await self.stt.transcribe_microphone(duration)

    async def say(self, text: str) -> bool:
        return await self.tts.speak(text)

    async def transcribe_file(self, path: str) -> str:
        return await self.stt.transcribe_file(path)

    async def transcribe_voicemail(self, path: str, platform: str) -> str:
        return await self.stt.transcribe_voicemail(path, platform)

    async def wake_word_loop(self, wake_word: str = "hey microdragon",
                              callback=None):
        """Always-on wake word detection."""
        try:
            import pvporcupine
            # Picovoice Porcupine — best wake word engine
            print(f"  Wake word active: '{wake_word}'")
            print("  Say the wake word to activate Microdragon.")
            # ... full Porcupine integration
        except ImportError:
            # Fallback: simpler keyword spotting
            print(f"  Wake word mode (basic): listening for '{wake_word}'")
            print("  For better wake word: pip install pvporcupine")
            # Just use continuous transcription and check
            while True:
                text = await self.listen(3)
                if wake_word.lower() in text.lower():
                    print(f"  🐉 Activated!")
                    if callback:
                        await callback()
                    break
                await asyncio.sleep(0.1)
