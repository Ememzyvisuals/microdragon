"""
microdragon/modules/gaming/src/cli_commands.py
Gaming commands exposed to MICRODRAGON CLI.
Usage from MICRODRAGON interactive mode:
  microdragon game play "GTA V"
  microdragon game play "Need for Speed Heat" --duration 600
  microdragon game play "Mortal Kombat 11" --character scorpion
  microdragon game status
  microdragon game stop
"""

import asyncio
import sys
import os
from pathlib import Path

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent))
from engine import MicrodragonGameEngine, GameGenre


# ─── Global session ───────────────────────────────────────────────────────────

_active_engine: MicrodragonGameEngine = None


async def cmd_play(game_name: str, duration: int = 300,
                   character: str = "", window_title: str = "") -> str:
    """Start MICRODRAGON playing a game."""
    global _active_engine

    if _active_engine and _active_engine.running:
        return "A game session is already running. Use 'microdragon game stop' first."

    _active_engine = MicrodragonGameEngine()

    print(f"""
╔══════════════════════════════════════════════════════════╗
║  ⬡ MICRODRAGON GAME ENGINE                                      ║
╠══════════════════════════════════════════════════════════╣
║  Game:     {game_name:<44} ║
║  Duration: {duration}s{' '*max(0,43-len(str(duration)))}║
║  Mode:     AI-controlled (80%+ accuracy target)          ║
╠══════════════════════════════════════════════════════════╣
║  MAKE SURE:                                              ║
║  1. {game_name[:50]:<50} ║
║     is open and in focus (windowed or borderless)        ║
║  2. You are on the gameplay screen (not main menu)       ║
║  3. Python deps are installed:                           ║
║     pip install mss pynput opencv-python-headless numpy  ║
╚══════════════════════════════════════════════════════════╝

  Starting in 5 seconds... Switch to your game now!
""")

    await asyncio.sleep(5)

    report = await _active_engine.play(
        game_name,
        duration_seconds=duration,
        window_title=window_title or game_name
    )

    return format_report(report)


def cmd_stop() -> str:
    """Stop current game session."""
    global _active_engine
    if _active_engine and _active_engine.running:
        _active_engine.stop()
        return "Game session stopped."
    return "No active game session."


def cmd_pause() -> str:
    """Pause/resume current game session."""
    global _active_engine
    if _active_engine and _active_engine.running:
        return _active_engine.pause()
    return "No active game session."


def cmd_status() -> str:
    """Get current game session status."""
    global _active_engine
    if _active_engine:
        return _active_engine.get_status()
    return "No game session running."


def cmd_list_games() -> str:
    """List games MICRODRAGON can play."""
    return """
MICRODRAGON Game Engine — Supported Games
════════════════════════════════════════════════════════

OPEN WORLD (Strategy: Drive + Combat + Explore)
  • GTA V / Grand Theft Auto V
  • GTA San Andreas / GTA Vice City
  • Red Dead Redemption 2
  • Saints Row series

RACING (Strategy: Lane keeping PID + Racing line)
  • Need for Speed (Heat, Unbound, Most Wanted)
  • Forza Horizon / Forza Motorsport
  • Gran Turismo 7
  • Assetto Corsa
  • Mario Kart (emulator)
  • Burnout Paradise
  • F1 series

FIGHTING (Strategy: Frame-perfect combos + block/punish)
  • Mortal Kombat 11 / MK1
  • Street Fighter 6 / SF5
  • Tekken 8 / Tekken 7
  • Injustice 2
  • Soul Calibur 6
  • Guilty Gear Strive
  • DNF Duel

SHOOTER (Strategy: AI aim + strafe + tactical movement)
  • Call of Duty (Modern Warfare, Warzone)
  • Counter-Strike 2 (CS2)
  • Valorant
  • Halo Infinite
  • Battlefield series
  • Apex Legends
  • Overwatch 2

Accuracy target: 80%+ on core gameplay tasks

Usage:
  microdragon game play "GTA V"
  microdragon game play "Need for Speed Heat" --duration 600
  microdragon game play "Mortal Kombat 11" --character scorpion
  microdragon game stop
  microdragon game status
"""


def format_report(report: dict) -> str:
    """Format game session report."""
    highlights = report.get("session_highlights", [])
    hl_text = ""
    for h in highlights:
        hl_text += f"  [{h.get('time', '?')}s] {h.get('reasoning', '')} (HP:{h.get('health', '?'):.0f}%)\n"

    return f"""
╔══════════════════════════════════════════════════════════╗
║  ⬡ MICRODRAGON GAME SESSION REPORT                             ║
╚══════════════════════════════════════════════════════════╝

  Game:            {report.get('game', 'Unknown')}
  Genre:           {report.get('genre', 'Unknown')}
  Duration:        {report.get('duration_seconds', 0)}s
  Frames analysed: {report.get('frames_analysed', 0):,}
  Actions taken:   {report.get('actions_taken', 0):,}
  Actions/second:  {report.get('actions_per_second', 0)}
  Deaths:          {report.get('deaths', 0)}
  Average FPS:     {report.get('avg_fps', 0)}

  ┌─────────────────────────────────┐
  │  ACCURACY:  {report.get('estimated_accuracy', 'N/A'):<22} │
  │  CONFIDENCE: {report.get('avg_confidence', 'N/A'):<21} │
  └─────────────────────────────────┘

  Session Highlights:
{hl_text or '  (No highlights recorded)'}
"""


if __name__ == "__main__":
    import sys
    game = " ".join(sys.argv[1:]) if len(sys.argv) > 1 else "GTA V"
    asyncio.run(cmd_play(game))
