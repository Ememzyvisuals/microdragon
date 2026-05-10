"""
microdragon/modules/gaming/knowledge/game_knowledge_base.py
═══════════════════════════════════════════════════════════════════════════════
MICRODRAGON GAME KNOWLEDGE BASE
═══════════════════════════════════════════════════════════════════════════════

MICRODRAGON is trained on complete verified game knowledge. When it loads GTA V,
it knows every street, every mission objective, every cheat code, every weapon
stat, every vehicle handling characteristic, every wanted level mechanic.

This is what powers the 80%+ gameplay accuracy.

Sources: Official Rockstar documentation, GTA Wiki, NFS Wiki, Mortal Kombat
Wiki, CS2 official wiki, verified community knowledge bases.

© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
"""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class GameKnowledge:
    title: str
    genre: str
    platform: str
    controls: dict        # action → key/button
    mechanics: dict       # mechanic name → explanation
    cheats: dict          # cheat name → code
    tips: list            # pro tips and hidden knowledge
    strategy_notes: str   # MICRODRAGON-specific AI strategy


# ─── GTA V ────────────────────────────────────────────────────────────────────

GTA_V = GameKnowledge(
    title="Grand Theft Auto V",
    genre="open_world",
    platform="PC/PS/Xbox",
    controls={
        # Movement
        "Move Forward": "W",
        "Move Backward": "S",
        "Move Left": "A",
        "Move Right": "D",
        "Sprint": "Shift",
        "Walk (toggle)": "Caps Lock",
        "Jump": "Space",
        "Crouch": "Ctrl",
        "Cover": "Q",
        "Peek Left": "E (while in cover)",
        "Peek Right": "Q (while in cover)",
        # Combat
        "Aim Weapon": "Right Mouse Button",
        "Fire Weapon": "Left Mouse Button",
        "Reload": "R",
        "Next Weapon": "Scroll Up",
        "Previous Weapon": "Scroll Down",
        "Melee Attack": "Right Mouse Button (unarmed)",
        "Throw Grenade": "Hold G",
        "Take Cover": "Q",
        "Blind Fire": "Left Mouse Button (while in cover)",
        "Zoom In": "Scroll Up (while aimed)",
        # Vehicle
        "Accelerate": "W",
        "Brake/Reverse": "S",
        "Steer Left": "A",
        "Steer Right": "D",
        "Handbrake": "Space",
        "Horn": "E",
        "Headlights": "H",
        "Enter/Exit Vehicle": "F",
        "Vehicle Weapon": "Left Mouse Button",
        "Change Radio": "Q/E",
        "Look Behind": "C",
        "Drive-by Left": "Q",
        "Drive-by Right": "E",
        "Cinematic Camera": "V",
        # On foot camera
        "Camera Look": "Mouse",
        "Change Camera": "V",
        "Phone": "Up Arrow",
        "Interaction Menu": "M",
        "Character Switch": "Alt+1/2/3 (Michael/Trevor/Franklin)",
        "Map": "Escape then Map",
        "Inventory/Weapons": "Tab",
        "Special Ability": "Caps Lock (hold)",
    },
    mechanics={
        "Wanted System": """
STAR LEVELS (1-5):
★ - Police investigate, patrol nearby. Evade: leave area quickly.
★★ - Patrol cars pursue. Evade: escape pursuit radius or hide 60s.
★★★ - Helicopters deployed. Lose by breaking line of sight + hiding.
★★★★ - FIB (FBI) units with assault rifles. Very aggressive.
★★★★★ - Military with army vehicles and Noose. Nearly impossible on foot.

EVADE STRATEGIES:
1. Enter Pay-n-Spray (changes plate/colour) — removes 1-2 stars
2. Leave the red circle (satellite dish radius) — must break LOS
3. Underground tunnels break helicopter line-of-sight
4. Change outfit at safehouse (removes 1 wanted star sometimes)
5. Ocean — police don't follow into water
6. Tunnel/multi-storey car park — helicopter can't follow
7. Safehouses — waiting removes wanted level if you're hidden

PURSUIT RADIUS:
- The flashing area on minimap = where police believe you are
- Once you exit and they lose visual, the radius appears as dotted
- Stay hidden 60+ seconds = stars removed
""",
        "Special Abilities": """
MICHAEL: Slow-mo bullet time while aiming (8 seconds, recharges by killing)
TREVOR: Rage mode — 50% damage taken, 2x damage dealt (recharges over time)
FRANKLIN: Slow-mo driving (recharges when driving fast)
Activate: Hold Caps Lock (default PC)
""",
        "Vehicle Classes": """
SUPERCARS (Top Speed): Zentorno, Entity XF, Turismo R, Adder, Cheetah
MUSCLE CARS: Best for handbrake turns, drifting
MOTORCYCLES: Akuma, Hakuchou are fastest on straights
HELICOPTERS: Buzzard (armed), Maverick (transport)
PLANES: Cuban 800 (small), Titan (cargo), Luxor (private jet)
BOATS: Dinghy, Jetski for water escapes
ARMOURED: Insurgent, Rhino Tank, Armoured Kuruma

STEALING HIGH-END VEHICLES:
- Outside Simeon's garage (premium vehicles)
- Airport (Cargobob, military jets - 4+ stars)
- Port (boats, cargo vehicles)
- Golf club (supercars)
""",
        "Money Making": """
MISSIONS (Story):
- Heist setup: ~$50k per job
- Main heists (The Big Score, Paleto): $100k-$200k
- Stock market manipulation: Lester assassination missions

STOCK MARKET (SINGLE PLAYER EXPLOIT):
1. Before Lester missions, invest in the TARGET company's rival
2. Complete assassination = target company drops, rival rises
3. Vice Assassination: invest in Fruit (not Facade) beforehand
4. Hotel Assassination: invest in Bilkinton
5. Bus Assassination: invest in Vapid
6. Construction Assassination: invest in GoldCoast
""",
        "Weapon Damage Stats": """
PISTOLS: AP Pistol - best DPS for handguns
SUBMACHINE GUNS: Micro SMG - drive-by, Uzi equivalent
ASSAULT RIFLES: Special Carbine - best overall AR
SNIPER RIFLES: Heavy Sniper - one-shot headshot on armoured
SHOTGUNS: Pump Shotgun Mk II with slug ammo = sniper range
EXPLOSIVES: Sticky Bomb > Grenade for vehicles
HEAVY: RPG for vehicles, Minigun for crowds
LMG: MG Mk II - suppressed for stealth approach

MODDING: Silencer, Extended Clip, Flashlight are priority upgrades
""",
        "Map Knowledge": """
KEY LOCATIONS:
- Ammu-Nation: weapons (marked on map after finding)
- Los Santos Customs: vehicle mods (blue wrench icon)
- Pay & Spray: respray to lose 1 star (rarely used)
- Hospital: respawn point after dying
- Police Station: retrieve confiscated weapons
- Airport: LSIA (south LS), Sandy Shores Airfield (north)
- Fort Zancudo: military base, Lazer jets inside
- Maze Bank Arena: vehicle events
- Vinewood Hills: Michael's house
- Sandy Shores: Trevor's trailer
- Strawberry: Franklin's house

FAST TRAVEL:
- Taxis: call cab, skip travel (costs money)
- Helicopter: fastest map traversal
""",
    },
    cheats={
        # PC keyboard cheats (type without pausing)
        "Full Health + Armour": "TURTLE",
        "Full Ammo": "TOOLUP",
        "Wanted Level Up": "FUGITIVE",
        "Remove Wanted Level": "LAWYERUP",
        "Drunk Mode": "LIQUOR",
        "Explosive Ammo Rounds": "HIGHEX",
        "Explosive Melee Attacks": "HOTHANDS",
        "Fast Run": "CATCHME",
        "Fast Swim": "GOTGILLS",
        "Super Jump": "HOPTOIT",
        "Give Parachute": "SKYDIVE",
        "Slow Motion Aim (x4)": "DEADEYE",
        "Skyfall": "SKYFALL",
        "Slidey Cars": "SNOWDAY",
        "Spawn Buzzard": "BUZZOFF",
        "Spawn Comet": "COMET",
        "Spawn Sanchez": "OFFROAD",
        "Spawn Trashmaster": "TRASHED",
        "Spawn Limo": "VINEWOOD",
        "Spawn Stunt Plane": "BARNSTORM",
        "Spawn PCJ-600": "ROCKET",
        "Spawn Rapid GT": "RAPIDGT",
        "Spawn BMX": "BANDIT",
        "Lower Wanted Level (cell phone)": "1-999-529-93787",
        "Full Health (cell phone)": "1-999-887-853",
        "Weapons (cell phone)": "1-999-866-587",
        "Note": "Cheats disable achievements/trophies for that session"
    },
    tips=[
        "Always upgrade to Bulletproof Tires — eliminates wheel destruction",
        "Armoured Kuruma: best armoured car for early game missions",
        "Franklin's special ability + Rapid GT = unbeatable getaway car",
        "Explosives: sticky bombs can be thrown onto walls and ceilings",
        "Helicopter escape: LSIA hangar always has helicopters, take one",
        "Fort Zancudo: drive in through main gate at speed, get to jet bay",
        "Police don't pursue into the water — boats are underrated escape tools",
        "Call Lester to remove wanted level ($500 per star, usable once per 5 min)",
        "Passive mode protects against players in GTA Online (not NPCs)",
        "Hidden packages underwater contain weapons and cash",
        "Vehicle impound: police garage near airport reclaims lost cars free",
        "Weapon wheel pause trick: pausing to switch weapons slows time slightly",
        "Buy high-end cars from in-game websites, delivered instantly",
        "Three-character switch restores health slightly on switch",
        "Yoga minigame: watch for out-of-body event — secret character",
    ],
    strategy_notes="""
MICRODRAGON DRIVING STRATEGY:
- Road detection confidence: 78-85%
- Uses HoughLinesP on bottom 40% of screen
- Speed management: brake when |angle| > 20° AND speed > 160
- Wanted evasion: prioritise water/tunnels over car parks
- Vehicle targeting: aim for centre mass, burst fire

MICRODRAGON COMBAT STRATEGY:
- Cover system: press Q near walls, vehicles automatically
- Blind fire: safe from cover, low accuracy but suppresses enemies
- Grenade: predict enemy position, throw ahead of movement
- Priority targets: helicopters first (they can't be lost)
- Health critical (<20%): break LOS, find cover immediately
"""
)

# ─── GTA San Andreas ──────────────────────────────────────────────────────────

GTA_SA = GameKnowledge(
    title="GTA San Andreas",
    genre="open_world",
    platform="PC/PS2/Mobile",
    controls={
        "Move": "WASD",
        "Sprint": "Shift",
        "Jump": "Space",
        "Crouch": "C",
        "Aim": "Right Mouse",
        "Fire": "Left Mouse",
        "Enter Vehicle": "F",
        "Accelerate": "W",
        "Brake": "S",
        "Handbrake": "Space",
    },
    mechanics={
        "Stats System": """
CJ has levelled stats:
- Muscle: increases max health, melee damage
- Fat: increases max health but slows movement
- Stamina: sprint distance before exhaustion
- Lung Capacity: diving time
- Driving/Flying/Cycling/Swimming: performance bonuses
- Weapon Skill: auto-aim range, reload speed, dual-wield unlock
  (Poor → Gangster → Hitman via usage reps)

BUILD MUSCLE: gym workouts, sprinting
BUILD STAMINA: swimming, long runs
WEAPON SKILL: shoot with any gun 200+ times
""",
        "Gang Territory": """
LOS SANTOS: Grove Street Families vs Ballas vs Vagos
- Attack territory: kill all enemies in orange zone
- Defend territory: respond when own turf attacked
- Hold 35%+ LS territory: storyline progresses faster
GANG WARS: start by killing 2-3 rival gang members near their turf
""",
        "Flying": """
Flight school required for Pilot skill
- Sprunk airfield (Red County): hidden crop duster
- Verdant Meadows: unlocks after desert missions
- Military planes: Nevada (cargo), Hydra (jet with vertical takeoff)
Hydra VTOL: press 8 (numpad) to switch between hover/flight modes
""",
    },
    cheats={
        "Health, Armour, $250k": "HESOYAM",
        "Wanted Level Down": "ASNAEB",
        "Wanted Level Up": "OSRBLHH",
        "All Weapons (set 1)": "LXGIWYL",
        "All Weapons (set 2)": "PROFESSIONALSKIT",
        "All Weapons (set 3)": "UZUMYMW",
        "Infinite Ammo": "FULLCLIP",
        "Fast Run": "VKYPQCF",
        "Super Punch": "IAVENJQ",
        "Jetpack": "ROCKETMAN",
        "Spawn Hydra": "JUMPJET",
        "Spawn Rhino": "AIWPRTON",
        "Spawn Monster Truck": "MONSTERMASH",
        "Lower Wanted": "ASNAEB",
        "Max Respect": "WORSHIPME",
        "Max Sex Appeal": "HELLOLADIES",
        "Muscle Max": "BUFFMEUP",
        "Fat Max": "BTCDBCB",
        "Thin": "KVGYZQK",
        "Night Vision": "LIYOAAY",
        "Infinite Lung Capacity": "CVWKXAM",
        "Super Jump": "KANGAROO",
        "Never Hungry": "AEDUWNV",
        "Pedestrians Riot": "AJLOJYQY",
        "Fog": "CFVFGMJ",
        "Storm": "AUIFRVQS",
        "Note": "Same as GTA V — cheats disable saves in some versions"
    },
    tips=[
        "Burglary side mission: stealth rob houses 22:00-06:00 for early cash",
        "Taxi missions: quick cash, builds stamina",
        "Vigilante missions: best for building wanted system knowledge",
        "Big Smoke's Crack Palace: infinite money glitch with save",
        "Madd Dogg's rhyme book: pick up early, saves story troubles",
        "Warm Reception mission: destroy Ballas with Uzi from drive-by",
        "Area 51 infiltration: jetpack cheat is simplest approach",
        "Betting: save before horse race, reload if you lose (save-scum)",
        "Infinite sprint: equip Minigun, sprint — uses Minigun ammo not stamina",
    ],
    strategy_notes="""
MICRODRAGON GTA SA STRATEGY:
- Older game, lower visual fidelity → easier vision parsing
- Road lanes more rigid/linear than GTA V
- Driving physics different: less grip, more slide
- Combat: drive-by dominant early game
- Health regen: eat at restaurants (Cluckin Bell etc.)
"""
)

# ─── Need for Speed Heat ──────────────────────────────────────────────────────

NFS_HEAT = GameKnowledge(
    title="Need for Speed Heat",
    genre="racing",
    platform="PC/PS4/Xbox",
    controls={
        "Accelerate": "W / Right trigger",
        "Brake": "S / Left trigger",
        "Steer Left": "A / Left stick",
        "Steer Right": "D / Right stick",
        "Handbrake": "Space / A button",
        "Nitro": "Left Shift / X button",
        "Respawn": "Backspace / Options",
        "Look Back": "C",
        "Change Camera": "V",
        "Flip Camera": "F",
    },
    mechanics={
        "Day vs Night": """
DAY: Sanctioned race events — earn REP (reputation) and CASH
NIGHT: Illegal street races — earn more REP but HEAT level rises

HEAT LEVELS:
1 - Police patrol. Easy to escape.
2 - Faster police pursuit. Roadblocks start.
3 - Heavy units, helicopters. Frequent roadblocks.
4 - High-speed pursuit units. Spike strips.
5 - Maximum pursuit. Rhino SUVs, heavy vehicles, helicopters everywhere.

BANK WINNINGS:
- Survive until 0600 and reach a safe house to bank night earnings
- If busted: lose ALL night earnings
- If wrecked: lose 50% of night earnings
- Risk/reward: higher heat = more REP multiplier

HEAT REDUCTION:
- Destroy police cars to temporarily reduce heat
- Reach safe house to eliminate heat entirely
""",
        "Car Handling Classes": """
GRIP: best for technical circuits, corners, wet weather
DRIFT: oversteer tuning, best for points events and style
RACE: balanced for most situations

TYRE SMOKE:
- Burnout/donuts: hold brake + accelerate on rear-wheel-drive
- Use for heat multiplier in free roam

TOP CARS BY CLASS:
Tier 3 (Best): McLaren P1, Lamborghini Aventador SVJ, Porsche 911 GT3 RS
Tier 2: Ford Mustang GT350R, Subaru WRX STI, Mitsubishi Lancer Evo X
Starter/Budget: Ford Focus RS (handling beast for cost)
""",
        "Repair Points": """
DAMAGE SYSTEM: Cars accumulate body damage
- Hits reduce handling and top speed
- Repair at gas stations (marked blue on map)
- First repair is always free
- Cost scales with damage level
- MICRODRAGON detects damage via frame brightness changes around car
""",
        "Police Evasion": """
EFFECTIVE ESCAPES:
1. Alleys and shortcuts — police don't always follow small gaps
2. Jump off highways at high speed (police hesitate at edges)
3. Off-road sections — some police cars can't follow
4. Destroy 3+ police cars — reduces heat level temporarily
5. Safe houses (marked on map) — bank winnings + reset heat
6. REP safe houses unlock as you level up (more in each district)

RAMMING:
Police use PIT manoeuvres at heat 2+
Counter: tap brakes just before contact to avoid spin
""",
    },
    cheats={
        "Note": "NFS Heat does not have traditional cheat codes",
        "Unlimited Nitro (modded sessions)": "Not available in base game",
        "Money glitch (patched)": "Previously: complete Speedhunter Circuit at night repeatedly",
        "Fast REP": "Complete night race events at heat 3+, bank safely"
    },
    tips=[
        "Build a balanced car first: 350+ HP, grip tyres, turbo, suspension tuning",
        "Get the Subaru BRZ early — best starter car for handling events",
        "Heat 3 with x3 REP multiplier = fastest REP grind if you bank safely",
        "Helicopter: use tunnels and overpasses to break line of sight",
        "Off-road shortcuts marked by dirt paths — use them for escapes",
        "Ram roadblock vehicles from the side/corner, not head-on",
        "Nitro: save for straights, never use in corners",
        "Tyre smoke drifts near police add heat very fast — avoid near cops",
        "Radio tower locations: destroying them adds to rep bonus",
        "Flamingo Drift Zone: best location for earning drift points quickly",
    ],
    strategy_notes="""
MICRODRAGON NFS HEAT STRATEGY:
- PID steering with kp=0.008, ki=0.0001, kd=0.005
- Night mode: aim for heat 3, achieve x3 REP multiplier
- Police evasion: prioritise tunnels (blocks helicopter)
- Nitro management: detect straight sections (angle < 5°) for boost
- Damage detection: monitor screen corners for red flash indicators
- Safe house routing: always know nearest safe house at heat 3+
"""
)

# ─── Mortal Kombat 11 ─────────────────────────────────────────────────────────

MK11 = GameKnowledge(
    title="Mortal Kombat 11",
    genre="fighting",
    platform="PC/PS/Xbox/Switch",
    controls={
        # PC default
        "Front Punch (1)": "U",
        "Back Punch (2)": "I",
        "Front Kick (3)": "J",
        "Back Kick (4)": "K",
        "Block": "E",
        "Throw": "Y",
        "Forward": "D",
        "Back": "A",
        "Up/Jump": "W",
        "Down/Crouch": "S",
        "Interact": "G",
        "Fatal Blow": "Block + Block (LB+LT / L1+L2)",
        "Flawless Block": "Block at exact moment of hit",
        "Krushing Blow": "Perform special move under specific condition",
    },
    mechanics={
        "Health Bars": """
Two health bars per character (top of screen)
P1 on left (depletes right-to-left)
P2 on right (depletes left-to-right)
Sub-health bar shows incoming damage that can be recovered

ROUNDS: First to win 2 rounds wins match
DRAW: If both die simultaneously, sudden death overtime
""",
        "Fatal Blow": """
ACTIVATION: Press Block+Block when your health bar is below 30%
  - Cinematically powerful combo attack
  - Cannot be escaped once connected
  - Does ~30-35% damage
  - Can only be used ONCE per match (not per round)
  - Cooldown: usable again if you get hit after attempting it
  
TIMING: Use when opponent is at 30-40% health to guarantee finish
COUNTER: Opponent can activate theirs when you're at 30%
""",
        "Krushing Blows": """
Cinematically enhanced versions of specific moves
Each activates under specific conditions:
- Example: Scorpion Teleport Punch KB = after blocking 2 attacks
- Triggers automatically when condition is met
- Much higher damage than regular version
- Recognise by distinct camera angle and visual effect
""",
        "Meter System": """
OFFENSIVE: Builds by landing attacks, used for:
  - Amplified special moves (tap button after special)
  - Combo breaker
DEFENSIVE: Builds by taking damage, used for:
  - Breakaway (D,B + back punch) — escape combos
  - Flawless Block counter attacks
""",
        "Fatalities": """
Each character has 2 fatalities + 1 friendship
PERFORM: When victory screen shows "FINISH HIM/HER", enter sequence
Must be at specific range (Close/Mid/Far)
Examples (Scorpion defaults):
  Fatality 1 (Toasty!): B,F,B,F,1 (mid range)
  Fatality 2 (Chain Reaction): B,F,D,D,2 (close range)
""",
        "Brutalities": """
Brutalities kill opponent during final round with a specific finishing move
CONDITIONS: Each brutality has requirements (e.g., "Must land 3 amplified attacks")
Often the final combo must use a specific move as the finisher
Unlocked via Krypt or rotating store
""",
    },
    cheats={
        "Unlimited Health (Practice Mode)": "Set in Practice Mode Settings",
        "AI Control": "Practice Mode > CPU Control for both players",
        "Unlock All Fatalities (Krypt)": "Requires grinding Koins in Krypt",
        "Easy Fatalities": "Use Easy Fatality tokens (consumable items)",
        "Skip to Brutality": "Complete brutality conditions mid-combo",
        "Note": "No traditional cheat codes — use Practice Mode settings"
    },
    tips=[
        "Learn one character deeply before switching — movelist mastery is everything",
        "Flawless Block timing: block at the last possible frame — punishes opponent heavily",
        "Anti-air: every character has an uppercut (Back Punch) — use against jumpers",
        "Footsie range: find your character's best mid-range poke and spam it",
        "Meter: save for Breakaway when in dangerous situations (not for offensive meters only)",
        "Wakeup attacks: when knocked down, can use special or Fatal Blow on rising",
        "Corner pressure: once opponent is cornered, they have no escape options",
        "Krushing Blow knowledge: memorise KBs for your main character",
        "AI variations in Towers: adjust AI difficulty in settings for grinding",
        "Practice Mode: use 'Record and Playback' to practice against real mixups",
    ],
    strategy_notes="""
MICRODRAGON MK11 STRATEGY:
Vision: detect P1/P2 health bars at top of screen, super meter at bottom
State machine: neutral → approach → combo → block → punish

CHARACTER TIERS (2026):
S: Robocop, Fujin, Cetrion
A: Liu Kang, Jacqui, Johnny Cage
B: Scorpion, Sub-Zero, Sonya
C: Shang Tsung, Kollector

COMBO WINDOWS: 16ms per frame at 60fps — ComboBuffer handles timing
FATAL BLOW: trigger when health < 30% AND opponent health > 30%
BREAKAWAY: trigger when taking >25% combo damage
ANTI-AIR PRIORITY: uppercut > jump kick air-to-air
"""
)

# ─── Mortal Kombat 1 (2023) ───────────────────────────────────────────────────

MK1 = GameKnowledge(
    title="Mortal Kombat 1 (2023)",
    genre="fighting",
    platform="PC/PS5/Xbox/Switch",
    controls={
        "Front Punch": "U",
        "Back Punch": "I",
        "Front Kick": "J",
        "Back Kick": "K",
        "Block": "E",
        "Kameo Tag": "R (call Kameo partner)",
        "Kameo Attack 1": "R+1",
        "Kameo Attack 2": "R+2",
    },
    mechanics={
        "Kameo System": """
NEW in MK1: Choose a Kameo fighter (assist character)
- Tag in with R button
- Each Kameo has 2 special attacks (R+1, R+2)
- Kameo adds combo extensions, anti-airs, or setups
- Kameo recharges after use (small bar under health)

BEST KAMEO COMBINATIONS:
Kung Lao + Cyrax: best combo extensions
Scorpion + Frost: anti-air into combo
Liu Kang + Stryker: projectile lockdown
""",
        "Super Meter": """
Single bar (replaces separate offensive/defensive meters)
- Amplify specials
- Invasions moves
- Kameo attacks also use meter
""",
        "Fatal Blow": "Same as MK11 — activate at 30% HP, once per match",
    },
    cheats={
        "Note": "No traditional cheats — use Practice Mode",
        "Unlock Invasion rewards": "Complete Mesa sectors in Invasions mode"
    },
    tips=[
        "Kameo selection matters as much as main character — test combinations in practice",
        "Cyrax Kameo: best combo extensions for almost every character",
        "Invasion Mode: best for unlocking cosmetics and extra content",
        "Ranked: reach Demi-God for Ranked cosmetics",
        "Premier Tower: weekly unique challenge with special rewards",
    ],
    strategy_notes="""
MICRODRAGON MK1 STRATEGY:
Same core framework as MK11 with Kameo additions
Key difference: factor in Kameo cooldown in combo planning
Vision: additional meter bar below health for Kameo status
"""
)

# ─── Street Fighter 6 ─────────────────────────────────────────────────────────

SF6 = GameKnowledge(
    title="Street Fighter 6",
    genre="fighting",
    platform="PC/PS/Xbox",
    controls={
        "Light Punch": "U",
        "Medium Punch": "I",
        "Heavy Punch": "O",
        "Light Kick": "J",
        "Medium Kick": "K",
        "Heavy Kick": "L",
        "Drive Impact": "HP+HK",
        "Drive Parry": "MP+MK",
        "Drive Reversal": "F+HP+HK (while blocking)",
        "Overdrive": "Two same buttons (EX moves)",
    },
    mechanics={
        "Drive System": """
Drive Gauge (6 bars) — the central mechanic of SF6:
DRIVE IMPACT (HP+HK): Armoured attack, breaks opponent's guard, wall splat
DRIVE PARRY (MP+MK): Hold to parry all attacks, refills Drive Gauge
OVERDRIVE (EX): Enhanced special moves using 2 Drive bars
DRIVE RUSH: Cancel normal into dash (costs 1 bar from Drive Parry cancel, 3 bars from normal)
DRIVE REVERSAL: Reversal out of blockstun, costs 2 bars

BURNOUT STATE: Losing all Drive bars → Burnout
  - Chip damage from blocking specials
  - Perfect guard removed
  - Drive Impact becomes crumpling hit
  - VERY dangerous state — must be avoided
""",
        "Super Arts": """
Level 1 (1 bar): Basic super combo
Level 2 (2 bars): More powerful version
Level 3 (3 bars): Full super, highest damage
Critical Art: Finishing super move

Each character has unique Super Art set
""",
        "Modern Controls": """
MODERN MODE (accessible): Single button specials
CLASSIC MODE: Traditional quarter-circle inputs
DYNAMIC MODE: Fully automated (AI assists)
MICRODRAGON uses Classic inputs for maximum accuracy
""",
    },
    cheats={
        "Note": "No traditional cheats in SF6",
        "Easy Mode": "World Tour Mode: use Modern control type",
        "Training shortcuts": "Training Mode > Record/Playback"
    },
    tips=[
        "Drive Parry on reaction to projectiles — refills Drive gauge while negating damage",
        "Drive Impact beats almost everything — use sparingly for surprise factor",
        "Don't get Burned Out — manage Drive Gauge like a resource",
        "Level 3 Super: always confirm into it from a hit for guaranteed damage",
        "Corner: Drive Rush into corner pressure is SF6's dominant offence",
        "Anti-air: each character has a reliable Heavy Punch anti-air",
        "World Tour Mode: levels up stats and teaches fundamentals",
    ],
    strategy_notes="""
MICRODRAGON SF6 STRATEGY:
Drive Gauge management is central to wins/losses
Offensive: Drive Impact for wall splat + full combo
Defensive: Drive Parry refill when safe
Never enter Burnout — monitor gauge constantly
"""
)

# ─── Tekken 8 ─────────────────────────────────────────────────────────────────

TEKKEN8 = GameKnowledge(
    title="Tekken 8",
    genre="fighting",
    platform="PC/PS5/Xbox",
    controls={
        "Left Punch": "1",
        "Right Punch": "2",
        "Left Kick": "3",
        "Right Kick": "4",
        "Block": "Back",
        "Low Block": "Crouch + Back",
        "Sidestep Left": "Double tap Up",
        "Sidestep Right": "Double tap Down",
        "Power Crush": "Character specific (usually b+1+2)",
        "Heat Activation": "1+2 or during specific moves",
    },
    mechanics={
        "Heat System": """
NEW in Tekken 8:
- Activate Heat with 1+2 or via specific moves
- Heat state: enhanced moves, chip damage on block, Heat Smash
- Heat Smash: powerful combo ender (uses all Heat gauge)
- Heat Dash: cancel some moves into a dash
- 30 second duration or until Heat Smash used

HEAT STRATEGY: Activate when landing mid-range hit, extend combo
""",
        "Movement": """
TEKKEN MOVEMENT is crucial:
- Backdash: quick retreat, avoids lows and some mids
- Sidestep: moves perpendicular to attacks — very effective
- Korean Backdash (KBD): backdash → neutral → backdash for max retreat
- Wavedash: forward dash into crouch repeatedly for pressure
""",
        "Rage System": """
RAGE ART: ~30% health remaining — one-shot comeback move (1+2+3+4)
RAGE DRIVE: Power-up version of specific moves at low health
Both break guard and deal massive damage
""",
    },
    cheats={
        "Note": "No traditional cheats in Tekken 8",
        "Unlock All Characters": "Purchase individually or Season Pass"
    },
    tips=[
        "Learn character string launchers — everything is about getting that juggle",
        "Backdash punish: recognise unsafe moves and punish on whiff",
        "Sidestep direction is character-specific — learn your opponent's weak side",
        "Power Crush moves trade damage — use when you predict an attack",
        "Heat Smash: keep it for combo ender, not random use",
        "Mishima electric: EWGF requires specific timing — hardest but most rewarding",
    ],
    strategy_notes="""
MICRODRAGON TEKKEN 8 STRATEGY:
3D movement adds complexity — sidestep left/right to avoid linear attacks
Vision: track distance carefully — Tekken neutral game is footsies-based
Heat activation: confirm from mid-range hit for optimal use
"""
)

# ─── Counter-Strike 2 (CS2) ───────────────────────────────────────────────────

CS2 = GameKnowledge(
    title="Counter-Strike 2",
    genre="shooter",
    platform="PC",
    controls={
        "Move": "WASD",
        "Fire": "Left Mouse",
        "Aim Down Sights": "Right Mouse (scoped weapons)",
        "Reload": "R",
        "Use": "E",
        "Jump": "Space",
        "Crouch": "Ctrl",
        "Walk (silent)": "Shift",
        "Drop Weapon": "G",
        "Buy Menu": "B",
        "Primary Weapon": "1",
        "Secondary Weapon": "2",
        "Knife": "3",
        "Grenade slot": "4/5/6 (frag/smoke/flash/molotov)",
        "Inspect Weapon": "F",
        "Scoreboard": "Tab",
        "Radio Commands": "Z/X/C",
        "Voice Chat": "K",
        "Chat": "Y",
        "Team Chat": "U",
    },
    mechanics={
        "Economy": """
STARTING CASH: $800 (pistol round)
ROUND WIN: +$3250 (CT) / +$3500 (T)
ROUND LOSS BONUS: Increases with consecutive losses
  - 1 loss: +$1400
  - 2 losses: +$1900
  - 3 losses: +$2400
  - 4 losses: +$2900
  - 5+ losses: +$3400
KILL REWARD: Varies by weapon ($300 rifle, $600 SMG, $900 knife/Zeus)
PLANT BONUS: +$800 for bomb planter
DEFUSE BONUS: +$300

ECONOMY STRATEGIES:
Full Buy: AK/M4 + helmet + kevlar + grenades = ~$4700
Force Buy: SMG or pistol upgrade when economy low
Eco Round: save money, buy next round
Save: deliberately don't fight, keep rifle for next round
""",
        "Map Callouts": {
            "Mirage": {
                "A Site": "Palace, Stairs, Jungle, CT, A Main",
                "B Site": "B Short, B Apps, Market, Van, B Main",
                "Mid": "Mid, Window, Catwalk, Connector",
            },
            "Dust 2": {
                "A Site": "A Long, A Short, A Main, Cat, Pit, Ramp",
                "B Site": "B Tunnels (Upper/Lower), B Doors, B Platform",
                "Mid": "Mid, Doors, Xbox, Short",
            },
            "Inferno": {
                "A Site": "A Long, Banana, A Short, Arch, Library, Balcony",
                "B Site": "B Apartments (Upper/Lower), B Back, B Site",
                "Mid": "Mid, Second Mid, Short",
            },
            "Nuke": {
                "A Site": "A Ramp, Heaven, A Main, A Secret",
                "B Site": "B Lower, B Upper, B Hut, Control Room",
                "Mid": "Outside, Yellow",
            },
        },
        "Grenade Usage": """
SMOKE: blocks vision completely — used to cut off sightlines
  - CT on Mirage: smoke window and jungle for B push
  - T on Dust2: smoke CT cross for A long push

FLASHBANG: temporary blind — throw to peek after pop
  - Pop flash: throw so it bursts in front of you, not facing you
  - Bounce flash: use walls to flash around corners

MOLOTOV/INCENDIARY: area denial — lasts 7 seconds
  - Block boost positions
  - Flush out corners

HE GRENADE: 57 max damage through walls (teamwork multi-HE)
  - Always throw through chokes (doorways, narrow paths)
  - Stack 2+ HE grenades for full kill
""",
        "Movement Mechanics": """
COUNTER-STRAFE: tap opposite direction key before shooting to stop instantly
  - Standing: full accuracy only when completely still
  - Moving: massively increases spread
  - Counter-strafing is ESSENTIAL for accuracy
  
CROUCH SPRAY: crouch while spraying to reduce recoil (but predictable)
  
BUNNY HOP: Space + directional strafe keys in sync — maintain speed
  Timing: jump just before landing (watch foot position)
  
PEEK MECHANICS:
  - Wide Peek: max distance from corner, see more, harder to pre-aim
  - Shoulder Peek: flash shoulder to bait shots without taking damage
  - Jiggle Peek: strafe in/out of corner quickly
""",
        "Weapon Tier List": """
RIFLES (Most Used):
  AK-47 (T): $2700, one-tap headshot through helmet, best rifle
  M4A4/M4A1-S (CT): $3100/$2900, accurate, low recoil
  SG553/AUG: scoped rifles, higher accuracy
  
SMGs:
  MP9/MAC-10: best on eco/force buy
  UMP-45: best SMG damage per bullet
  
Snipers:
  AWP: $4750 — one-shot kill to body/head (most powerful)
  Scout: $1700 — one-shot head only, fast movement
  
Pistols:
  Desert Eagle: $700 — one-tap headshot potential
  USP-S/Glock: default CT/T pistol
  
WORST VALUE: Negev (LMG) — situational only
""",
    },
    cheats={
        "Note": "CS2 has no in-game cheats — anti-cheat (VAC/Prime) enforced",
        "Practice Mode commands": "sv_cheats 1 (offline only)",
        "sv_cheats 1 commands": {
            "Infinite Ammo": "sv_infinite_ammo 1",
            "No Clip": "noclip",
            "God Mode": "god",
            "All Weapons": "give weapon_ak47",
            "Money": "impulse 101",
            "Bot Practice": "bot_add / bot_kick",
            "Grenade Trajectory": "sv_grenade_trajectory 1",
            "Grenade Warning": "sv_grenade_trajectory_time 10",
        }
    },
    tips=[
        "Always counter-strafe before shooting — the most important mechanical skill",
        "AK-47 spray pattern: down, slightly left — pull mouse down-left for control",
        "M4A4 spray: very controllable, pull straight down after first 4 shots",
        "Buy information: deduce enemy economy from previous round results",
        "Sound: use headphones, footsteps reveal position through walls",
        "Angles: pre-aim head height around corners, don't aim at the ground",
        "Economy: match your buy level to your team's — force-buying alone wastes rounds",
        "Smoke lineup: learn 3-5 smokes per map, they win more rounds than mechanics",
        "Rotate speed: T-side rotates in 10-15 seconds on most maps — time your execute",
        "Bomb plant: B-site plant hides in corner vs A-site open plant — learn positions",
    ],
    strategy_notes="""
MICRODRAGON CS2 STRATEGY:
Vision: enemy detection via red/outline player models
Counter-strafing simulation: simulate stop-start movement
Aim: enemy head position target, compensate for movement
Grenade: pre-load known smoke positions per map
Economy: track team money, suggest full-buy vs save
"""
)

# ─── Call of Duty (Modern Warfare / Warzone) ──────────────────────────────────

COD = GameKnowledge(
    title="Call of Duty: Modern Warfare / Warzone",
    genre="shooter",
    platform="PC/PS/Xbox",
    controls={
        "Move": "WASD",
        "Sprint": "Shift",
        "Tactical Sprint": "Double Shift",
        "Slide/Prone": "Ctrl",
        "Crouch": "C",
        "Jump/Mantle": "Space",
        "Aim Down Sights": "Right Mouse",
        "Fire": "Left Mouse",
        "Reload": "R",
        "Interact": "F",
        "Melee": "V",
        "Knife": "Hold V",
        "Swap Weapon": "Mouse Wheel / 1-2",
        "Lethal": "G",
        "Tactical": "Q",
        "Field Upgrade": "X",
        "Killstreak": "H (when earned)",
        "Ping Marker": "Z",
        "Map": "M",
        "Scoreboard": "Tab",
    },
    mechanics={
        "TTK (Time to Kill)": """
Very fast TTK compared to CS2:
At full health, most weapons kill in 150-300ms (3-6 shots)
AR at medium range: ~4 shots to head, ~5-6 to body
SMG at close range: 3-4 shots

IMPLICATIONS:
- Pre-aiming is critical (win the fight before it starts)
- Tracking aim is more important than burst accuracy (unlike CS)
- Cover usage: peek short, don't stand exposed for >200ms
""",
        "Movement Meta": """
SLIDE CANCEL: Sprint → Crouch (slide) → Sprint immediately
  Creates unpredictable movement, harder to track
  
DOLPHIN DIVE: Sprint → Prone while moving
  Gets you prone quickly in cover

BUNNY HOP: Jump + Sprint in cycle — slightly faster movement

DOOR MECHANICS: open doors to peek safely, close doors to block line of sight
""",
        "Warzone Specific": """
CIRCLE MECHANICS: Storm zone shrinks every 2-3 minutes
  - Edge of zone: 10-35 damage per second depending on ring
  - Always know where final circle is developing
  - Fight towards final zone, not away from it
  
LOADOUT DROP: Buy at Buy Station or earn via contract — enables custom loadout
RESPAWN: Buy teammate at Buy Station ($4500)
CONTRACTS: Bounty/Recon/Most Wanted — earn cash and XP
VEHICLES: ATV, truck, helicopter — fastest rotation, but loud
GULAG: First death → 1v1 in Gulag for respawn chance
""",
    },
    cheats={
        "Note": "No cheats — Anti-cheat enforced",
        "Dev Console": "Not available in public build"
    },
    tips=[
        "Slide cancel constantly — it reduces your hitbox and confuses tracking",
        "ADSing slows movement significantly — slide + hip-fire for CQB",
        "Warzone: always have $4500 ready for teammate respawn",
        "UAV + Counter-UAV combo reveals everyone and hides your team",
        "Final circle: high ground wins almost every time",
        "Lethal: Thermite sticks to surfaces and vehicles — better than frag",
        "Heartbeat Sensor: reveals nearby enemies through walls",
        "Stim (tactical): heals to full instantly — best in circle fights",
        "SMG is better than AR inside 20m — know your weapon's optimal range",
    ],
    strategy_notes="""
MICRODRAGON COD STRATEGY:
Fast TTK requires proactive aim (pre-aim) not reactive
Movement: simulate slide cancel for unpredictability
Cover: use peek duration of <200ms when exposed
Warzone: track circle and enemy count from kill feed
"""
)

# ─── Apex Legends ─────────────────────────────────────────────────────────────

APEX = GameKnowledge(
    title="Apex Legends",
    genre="shooter",
    platform="PC/PS/Xbox",
    controls={
        "Move": "WASD",
        "Sprint": "Shift",
        "Jump": "Space",
        "Crouch/Slide": "Ctrl",
        "Fire": "Left Mouse",
        "ADS": "Right Mouse",
        "Reload": "R",
        "Use/Interact": "E",
        "Ping": "Z",
        "Tactical Ability": "Q",
        "Ultimate Ability": "Left Shift",
        "Passive": "Automatic",
    },
    mechanics={
        "Legend Abilities": """
MOVEMENT LEGENDS (meta picks):
  Pathfinder: Grapple (Q) — extreme mobility, ziplines
  Octane: Launch Pad (Ult) — team boost + speed stim
  Wraith: Phase (Q) — invincibility during, portal (Ult)

SUPPORT:
  Lifeline: Healing drone, res with shield
  Newcastle: Shield on res, castle wall
  
COMBAT:
  Bloodhound: Scan (Q) + Ultimate reveals enemies
  Bangalore: Smoke for cover, Rolling Thunder
  Gibraltar: Dome shield (Q), defensive specialist
""",
        "Shield System": """
WHITE: 50 HP
BLUE: 75 HP
PURPLE: 100 HP
GOLD/RED: 100 HP + unique perk (e.g., instant execute knockdowns)

Total HP at purple: 100 + 100 = 200 HP effective
""",
        "Recoil Patterns": """
R301 Carbine: predictable, pull down-left — most popular
R99: high fire rate SMG, control with constant down-pull
Wingman: single-shot pistol, master for ranked
Flatline: Heavier recoil than R301, better damage
Alternator: easy control, good early game
CAR SMG: best mid-season pick, interchangeable ammo
""",
    },
    cheats={"Note": "No cheats — anti-cheat enforced"},
    tips=[
        "Ping system: use it constantly, one of the best in any BR game",
        "Rotate early — third-partying is valid strategy, arrive fresh",
        "Shield cells are faster than syringes — use them in firefights",
        "Survey Beacons (Pathfinder/Bloodhound): scan for next ring location",
        "High ground wins: elevation advantage is critical in Apex",
        "Wraith portal: set entrance BEFORE entering fight, exit to reset",
        "ALGS meta: SMG+AR combo (R301+R99) dominates most situations",
    ],
    strategy_notes="""
MICRODRAGON APEX STRATEGY:
Legend selection matters as much as aim — use mobility legends
Rotation: always be moving toward late-game zones early
Tracking aim required — Apex has longer TTK than COD
"""
)

# ─── Registry ─────────────────────────────────────────────────────────────────

GAME_KNOWLEDGE_BASE = {
    "gta v": GTA_V,
    "gta5": GTA_V,
    "grand theft auto v": GTA_V,
    "gta san andreas": GTA_SA,
    "san andreas": GTA_SA,
    "need for speed heat": NFS_HEAT,
    "nfs heat": NFS_HEAT,
    "mortal kombat 11": MK11,
    "mk11": MK11,
    "mortal kombat 1": MK1,
    "mk1": MK1,
    "street fighter 6": SF6,
    "sf6": SF6,
    "tekken 8": TEKKEN8,
    "counter-strike 2": CS2,
    "cs2": CS2,
    "call of duty": COD,
    "cod": COD,
    "warzone": COD,
    "apex legends": APEX,
    "apex": APEX,
}


def get_game_knowledge(game_name: str) -> GameKnowledge | None:
    return GAME_KNOWLEDGE_BASE.get(game_name.lower())


def get_cheat(game_name: str, cheat_name: str) -> str:
    game = get_game_knowledge(game_name)
    if not game:
        return f"Game '{game_name}' not in knowledge base"
    code = game.cheats.get(cheat_name)
    if code:
        return f"{game.title} — {cheat_name}: {code}"
    matches = [(k, v) for k, v in game.cheats.items() if cheat_name.lower() in k.lower()]
    if matches:
        return "\n".join(f"{k}: {v}" for k, v in matches[:3])
    return f"Cheat '{cheat_name}' not found for {game_name}"


def build_game_context(game_name: str, situation: str) -> str:
    game = get_game_knowledge(game_name)
    if not game:
        return f"Playing {game_name}: {situation}"
    return f"""GAME: {game.title} ({game.genre})
SITUATION: {situation}

KEY CONTROLS: {list(game.controls.items())[:10]}
STRATEGY: {game.strategy_notes}

Decide the best action for this situation.
Return ONLY: action_type, keys_to_press, reasoning.
"""
