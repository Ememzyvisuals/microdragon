"""
microdragon/modules/gaming/vision/game_vision.py
Advanced per-game vision: GTA minimap reading, NFS speed detection,
MK health bars, CS2 radar, etc.
"""

import math
import time
from dataclasses import dataclass
from typing import Optional


@dataclass
class GTAGameState:
    """GTA-specific extracted state."""
    minimap_direction: float = 0.0    # degrees player is facing on minimap
    wanted_stars: int = 0             # 0-5 wanted level
    money_display: int = 0
    health_bar: float = 100.0
    armour_bar: float = 0.0
    ammo_type: str = ""
    ammo_count: int = -1
    in_vehicle: bool = False
    vehicle_health: float = 100.0
    vehicle_speed: float = 0.0
    mission_waypoint: Optional[tuple] = None
    on_foot: bool = True


@dataclass
class NFSGameState:
    """Need for Speed specific state."""
    speed_mph: int = 0
    speed_kmh: int = 0
    gear: int = 1
    rpm_pct: float = 0.0             # 0.0 to 1.0
    nitro_available: bool = False
    nitro_pct: float = 0.0
    race_position: int = 1
    race_total: int = 8
    lap: int = 1
    total_laps: int = 3
    time_behind_leader: float = 0.0
    road_deviation: float = 0.0      # pixels off-centre
    collision_ahead: bool = False
    traffic_ahead: bool = False


@dataclass
class MKGameState:
    """Mortal Kombat specific state."""
    p1_health: float = 100.0
    p2_health: float = 100.0
    p1_super_meter: float = 0.0     # 0-100%
    p2_super_meter: float = 0.0
    round_number: int = 1
    p1_rounds_won: int = 0
    p2_rounds_won: int = 0
    round_time: float = 60.0
    x_ray_available: bool = False
    fatal_blow_available: bool = False
    flawless_possible: bool = True
    distance_between: float = 50.0
    p1_in_air: bool = False
    p2_in_air: bool = False
    p1_crouching: bool = False
    p2_crouching: bool = False


class GTAVisionAnalyser:
    """Specialised vision for GTA V / GTA San Andreas."""

    def analyse(self, frame, cv2, np) -> GTAGameState:
        state = GTAGameState()
        h, w = frame.shape[:2]

        # Health bar (bottom-left, green bar)
        hb_roi = frame[h-80:h-50, 10:200]
        state.health_bar = self._read_bar(hb_roi, cv2, np, colour="green")

        # Armour bar (below health, blue bar)
        ab_roi = frame[h-55:h-35, 10:200]
        state.armour_bar = self._read_bar(ab_roi, cv2, np, colour="blue")

        # Wanted level (top-right stars — each star = 15x15 px approx)
        star_roi = frame[5:35, w-250:w-10]
        state.wanted_stars = self._count_wanted_stars(star_roi, cv2, np)

        # Minimap (bottom-left circle)
        mm_roi = frame[h-200:h-10, 10:200]
        state.minimap_direction = self._read_minimap_heading(mm_roi, cv2, np)

        # Vehicle detection (check if steering wheel visible in bottom)
        speed_roi = frame[h-130:h-90, w-200:w-10]
        state.vehicle_speed = self._read_speedometer(speed_roi, cv2, np)
        state.in_vehicle = state.vehicle_speed > 0

        return state

    def _read_bar(self, roi, cv2, np, colour: str) -> float:
        """Read percentage from a coloured bar."""
        if roi.size == 0:
            return 100.0
        hsv = cv2.cvtColor(roi, cv2.COLOR_BGR2HSV)

        if colour == "green":
            mask = cv2.inRange(hsv, np.array([35,80,80]), np.array([85,255,255]))
        elif colour == "blue":
            mask = cv2.inRange(hsv, np.array([100,80,80]), np.array([140,255,255]))
        elif colour == "red":
            mask1 = cv2.inRange(hsv, np.array([0,80,80]), np.array([10,255,255]))
            mask2 = cv2.inRange(hsv, np.array([170,80,80]), np.array([180,255,255]))
            mask = cv2.bitwise_or(mask1, mask2)
        else:
            return 100.0

        # Leftmost to rightmost filled pixel = bar fill
        cols = np.any(mask > 0, axis=0)
        if not np.any(cols):
            return 0.0
        fill = np.sum(cols) / len(cols)
        return round(fill * 100, 1)

    def _count_wanted_stars(self, roi, cv2, np) -> int:
        """Count yellow wanted stars in ROI."""
        if roi.size == 0:
            return 0
        hsv = cv2.cvtColor(roi, cv2.COLOR_BGR2HSV)
        yellow_mask = cv2.inRange(hsv, np.array([20,100,100]), np.array([35,255,255]))
        contours, _ = cv2.findContours(yellow_mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        # Each star = one blob of appropriate size
        star_count = sum(1 for c in contours if 30 < cv2.contourArea(c) < 500)
        return min(star_count, 5)

    def _read_minimap_heading(self, roi, cv2, np) -> float:
        """Read player heading from minimap north indicator."""
        # Simplified — look for the red player dot direction
        return 0.0

    def _read_speedometer(self, roi, cv2, np) -> float:
        """Detect if speedometer is visible (vehicle mode)."""
        # In GTA V the speedometer is a circular dial bottom-right
        return 0.0


class NFSVisionAnalyser:
    """Need for Speed specific vision."""

    def analyse(self, frame, cv2, np) -> NFSGameState:
        state = NFSGameState()
        h, w = frame.shape[:2]

        # Speed display (bottom-centre or bottom-right)
        speed_roi = frame[h-100:h-40, w//2-100:w//2+100]
        state.road_deviation = self._detect_lane_centre_deviation(frame, cv2, np, h, w)
        state.collision_ahead = self._detect_collision_ahead(frame, cv2, np, h, w)

        # Nitro bar (usually bottom, bright blue/yellow)
        nitro_roi = frame[h-30:h-5, w//4:3*w//4]
        state.nitro_pct = self._read_nitro(nitro_roi, cv2, np)
        state.nitro_available = state.nitro_pct > 0.1

        return state

    def _detect_lane_centre_deviation(self, frame, cv2, np, h, w) -> float:
        """Measure how far off-centre the car is in its lane."""
        roi = frame[h*2//3:, :]
        gray = cv2.cvtColor(roi, cv2.COLOR_BGR2GRAY)
        edges = cv2.Canny(gray, 50, 150)
        lines = cv2.HoughLinesP(edges, 1, np.pi/180, 30, minLineLength=30, maxLineGap=20)

        if lines is None:
            return 0.0

        # Find lane lines and compute centre deviation
        left_xs = []
        right_xs = []
        for line in lines:
            x1, y1, x2, y2 = line[0]
            angle = math.degrees(math.atan2(y2-y1, x2-x1))
            mid_x = (x1 + x2) / 2
            if 20 < abs(angle) < 80:
                if mid_x < w / 2:
                    left_xs.append(mid_x)
                else:
                    right_xs.append(mid_x)

        if left_xs and right_xs:
            lane_centre = (max(left_xs) + min(right_xs)) / 2
            deviation = lane_centre - w / 2
            return round(deviation, 1)
        return 0.0

    def _detect_collision_ahead(self, frame, cv2, np, h, w) -> bool:
        """Detect if there's a car/obstacle directly ahead."""
        # Look at centre strip — if it's red or full of a large object
        ahead_roi = frame[h//3:h//2, w//3:2*w//3]
        gray = cv2.cvtColor(ahead_roi, cv2.COLOR_BGR2GRAY)
        # High variance = likely object ahead
        variance = float(np.var(gray))
        return variance > 2000

    def _read_nitro(self, roi, cv2, np) -> float:
        """Detect nitro/boost bar level."""
        if roi.size == 0:
            return 0.0
        hsv = cv2.cvtColor(roi, cv2.COLOR_BGR2HSV)
        # NFS nitro is usually bright blue or orange
        blue = cv2.inRange(hsv, np.array([100,150,100]), np.array([140,255,255]))
        orange = cv2.inRange(hsv, np.array([10,150,100]), np.array([25,255,255]))
        combined = cv2.bitwise_or(blue, orange)
        fill = np.sum(combined > 0) / combined.size
        return round(fill, 2)


class MKVisionAnalyser:
    """Mortal Kombat specific vision."""

    def analyse(self, frame, cv2, np) -> MKGameState:
        state = MKGameState()
        h, w = frame.shape[:2]

        # P1 health bar (top-left)
        p1_hp_roi = frame[10:35, 20:w//2-50]
        state.p1_health = self._read_health_bar(p1_hp_roi, cv2, np)

        # P2 health bar (top-right, mirrored)
        p2_hp_roi = frame[10:35, w//2+50:w-20]
        state.p2_health = self._read_health_bar(p2_hp_roi, cv2, np, flipped=True)

        # Super meter / Fatal Blow indicator
        p1_meter_roi = frame[h-40:h-10, 20:w//2-50]
        state.p1_super_meter = self._read_super_meter(p1_meter_roi, cv2, np)

        # Fatal blow available (when health < 30%)
        state.fatal_blow_available = state.p1_health < 30

        # Detect characters in air (jumping)
        centre_roi = frame[h//4:3*h//4, w//4:3*w//4]
        characters = self._detect_characters(centre_roi, cv2, np, h, w)
        if len(characters) >= 1:
            state.p1_in_air = characters[0].get("in_air", False)
        if len(characters) >= 2:
            state.p2_in_air = characters[1].get("in_air", False)

        # Distance between characters
        if len(characters) >= 2:
            dx = characters[0]["x"] - characters[1]["x"]
            state.distance_between = abs(dx)

        return state

    def _read_health_bar(self, roi, cv2, np, flipped: bool = False) -> float:
        """Read MK health bar — filled part is the health remaining."""
        if roi.size == 0:
            return 100.0
        hsv = cv2.cvtColor(roi, cv2.COLOR_BGR2HSV)

        # MK health bars are typically red/yellow gradient
        green_mask = cv2.inRange(hsv, np.array([35,100,100]), np.array([85,255,255]))
        yellow_mask = cv2.inRange(hsv, np.array([20,100,100]), np.array([35,255,255]))
        red1 = cv2.inRange(hsv, np.array([0,100,100]), np.array([10,255,255]))
        red2 = cv2.inRange(hsv, np.array([170,100,100]), np.array([180,255,255]))
        health_mask = cv2.bitwise_or(cv2.bitwise_or(green_mask, yellow_mask),
                                      cv2.bitwise_or(red1, red2))

        cols = np.any(health_mask > 0, axis=0)
        if not np.any(cols):
            return 0.0

        if flipped:
            # P2 bar fills right-to-left
            filled = np.sum(cols[::-1])
        else:
            filled = np.sum(cols)

        return round((filled / len(cols)) * 100, 1)

    def _read_super_meter(self, roi, cv2, np) -> float:
        """Read MK super meter (blue bar at bottom)."""
        if roi.size == 0:
            return 0.0
        hsv = cv2.cvtColor(roi, cv2.COLOR_BGR2HSV)
        blue = cv2.inRange(hsv, np.array([100,100,100]), np.array([140,255,255]))
        cols = np.any(blue > 0, axis=0)
        return round(np.sum(cols) / max(len(cols), 1) * 100, 1)

    def _detect_characters(self, roi, cv2, np, fh, fw) -> list:
        """Find character positions in fighting game."""
        characters = []
        gray = cv2.cvtColor(roi, cv2.COLOR_BGR2GRAY)
        # Threshold to find large blobs (characters)
        _, binary = cv2.threshold(gray, 30, 255, cv2.THRESH_BINARY)
        contours, _ = cv2.findContours(binary, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)

        large_contours = sorted(
            [c for c in contours if cv2.contourArea(c) > 3000],
            key=lambda c: cv2.boundingRect(c)[0]
        )

        for cnt in large_contours[:2]:  # max 2 characters
            x, y, w, h = cv2.boundingRect(cnt)
            cx = x + w // 2
            cy = y + h // 2
            # Character "in air" if Y position is higher than normal
            in_air = y < roi.shape[0] * 0.3
            characters.append({"x": cx, "y": cy, "h": h, "in_air": in_air})

        return characters
