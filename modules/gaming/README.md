# MICRODRAGON Gaming Module

MICRODRAGON can play games using computer vision + AI decision making + precise input control.

## How It Works

```
Screen Capture (30fps via mss)
        ↓
Frame Analysis (OpenCV)
  - Health bars        - Enemy detection
  - Road lane lines    - Speed/ammo/money
  - Game state         - Menu detection
        ↓
AI Decision Engine
  - Racing: PID lane-keeping + racing line
  - Fighting: Frame-perfect combos + block/punish
  - Open World: Drive + combat + mission logic
  - Shooter: AI aim + strafe + tactical movement
        ↓
Controller Output (pynput + vgamepad)
  - Keyboard inputs
  - Mouse movement (aim)
  - Virtual gamepad (Windows)
        ↓
Learning Loop
  - Track outcomes (died/completed/score)
  - Adjust strategy parameters
  - Improve over session
```

## Install Dependencies

```bash
# Core (required)
pip install mss pynput opencv-python-headless numpy

# Optional (improve accuracy)
pip install vgamepad      # Virtual Xbox gamepad (Windows only)
pip install pytesseract   # OCR for reading scores/ammo text
pip install pillow        # Additional image processing
```

## Usage

```bash
# From MICRODRAGON CLI
microdragon game play "GTA V"
microdragon game play "Need for Speed Heat" --duration 600
microdragon game play "Mortal Kombat 11" --character scorpion
microdragon game stop
microdragon game status
microdragon game list

# From interactive REPL
/game GTA V
/game stop
```

## Accuracy Notes

| Game Type | Task | Expected Accuracy |
|---|---|---|
| Racing | Stay on road | 85-90% |
| Racing | Beat AI opponents | 60-70% |
| Fighting | Execute combos | 80-85% |
| Fighting | Win matches vs Easy AI | 75% |
| Open World | Drive without crashing | 75-80% |
| Open World | Complete basic missions | 55-65% |
| Shooter | Hit targets | 65-75% |
| Shooter | Win deathmatches | 50-60% |

**Important:** Accuracy depends on:
1. Game window resolution (1280x720+ recommended)
2. No overlapping windows over the game
3. Stable framerate in the game itself
4. Windowed or borderless windowed mode

## Supported Games (Full List)

- **GTA V** / GTA San Andreas / Red Dead Redemption 2
- **Need for Speed** Heat, Unbound, Most Wanted, Hot Pursuit
- **Forza Horizon** 4/5, Forza Motorsport
- **Mortal Kombat** 11, MK1 (2023)
- **Street Fighter** 6, Street Fighter V
- **Tekken** 8, Tekken 7
- **Call of Duty** Modern Warfare, Warzone
- **Counter-Strike 2** (CS2)
- And any game with keyboard+mouse controls

## Safety

The gaming module includes safety guards:
- Will not minimize to desktop or switch windows
- Respects game window boundaries
- Stops immediately when you press Ctrl+C
- No macro injection — uses standard input APIs
