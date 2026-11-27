#!/usr/bin/env python3
"""Generate Solace app icons (PNG, ICNS, ICO) and menu bar Sun/Moon tray icons."""

import math
import os
import struct
import subprocess
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICONS_DIR = os.path.join(ROOT, "src-tauri", "icons")
os.makedirs(ICONS_DIR, exist_ok=True)


def write_png(path: str, width: int, height: int, rgba: bytes) -> None:
    def chunk(tag: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(tag + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    raw = bytearray()
    stride = width * 4
    for y in range(height):
        raw.append(0)
        raw.extend(rgba[y * stride : (y + 1) * stride])

    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )
    with open(path, "wb") as f:
        f.write(png)


def clamp(v: float, lo: float = 0.0, hi: float = 1.0) -> float:
    return max(lo, min(hi, v))


def smoothstep(e0: float, e1: float, x: float) -> float:
    t = clamp((x - e0) / (e1 - e0))
    return t * t * (3.0 - 2.0 * t)


def render_app_icon(size: int) -> bytes:
    """Render a refined squircle app icon: warm amber sunrise top half, deep indigo twilight bottom half, glowing horizon sun/moon split."""
    buf = bytearray(size * size * 4)
    cx = (size - 1) / 2.0
    cy = (size - 1) / 2.0
    half = size * 0.43  # macOS standard icon inset padding

    for y in range(size):
        for x in range(size):
            nx = (x - cx) / half
            ny = (y - cy) / half

            # Superellipse squircle (n = 5)
            d = (abs(nx) ** 5 + abs(ny) ** 5) ** 0.2
            edge_aa = 1.0 - smoothstep(1.0 - 2.2 / size, 1.0 + 1.2 / size, d)
            if edge_aa <= 0.0:
                continue

            # Vertical sky gradient: warm golden horizon -> twilight indigo
            t = clamp((ny + 1.0) * 0.5)
            if t < 0.52:
                lt = t / 0.52
                # Golden sunrise sky
                r = 252 - lt * 12
                g = 204 - lt * 58
                b = 118 - lt * 32
            else:
                dt = (t - 0.52) / 0.48
                # Deep indigo dusk
                r = 38 - dt * 20
                g = 42 - dt * 20
                b = 84 - dt * 32

            # Soft horizon glow
            horizon_dist = abs(ny - 0.04)
            glow = math.exp(-horizon_dist * 6.5) * 0.35
            r = clamp(r + glow * 255, 0, 255)
            g = clamp(g + glow * 165, 0, 255)
            b = clamp(b + glow * 75, 0, 255)

            # Central celestial orb: upper half bright sun, lower half silver crescent reflection
            dist_c = math.hypot(nx, ny - 0.02)
            orb_r = 0.44
            orb_mask = 1.0 - smoothstep(orb_r - 2.2 / size, orb_r + 1.2 / size, dist_c)
            if orb_mask > 0.0:
                if ny < 0.04:
                    # Warm sun upper disc
                    or_r, or_g, or_b = 255.0, 248.0, 224.0
                else:
                    # Moonlit reflection / dark eclipse lower half with subtle crescent rim
                    crescent_d = math.hypot(nx - 0.14, ny - 0.14)
                    rim = smoothstep(0.28, 0.44, crescent_d)
                    or_r = 165.0 + rim * 70.0
                    or_g = 185.0 + rim * 55.0
                    or_b = 235.0 + rim * 20.0
                r = r * (1.0 - orb_mask) + or_r * orb_mask
                g = g * (1.0 - orb_mask) + or_g * orb_mask
                b = b * (1.0 - orb_mask) + or_b * orb_mask

            # Crisp horizon divider line
            line_a = 1.0 - smoothstep(0.0, 0.028, abs(ny - 0.04))
            if line_a > 0.0:
                r = r * (1.0 - line_a * 0.45) + 255.0 * (line_a * 0.45)
                g = g * (1.0 - line_a * 0.45) + 230.0 * (line_a * 0.45)
                b = b * (1.0 - line_a * 0.45) + 190.0 * (line_a * 0.45)

            # Subtle inner rim highlight
            rim_a = smoothstep(0.92, 0.99, d) * 0.18
            r = clamp(r + rim_a * 255, 0, 255)
            g = clamp(g + rim_a * 255, 0, 255)
            b = clamp(b + rim_a * 255, 0, 255)

            idx = (y * size + x) * 4
            buf[idx] = int(clamp(r, 0, 255))
            buf[idx + 1] = int(clamp(g, 0, 255))
            buf[idx + 2] = int(clamp(b, 0, 255))
            buf[idx + 3] = int(clamp(edge_aa * 255, 0, 255))
    return bytes(buf)


def render_tray_sun(size: int = 44) -> bytes:
    """Monochrome template sun icon for menu bar / tray."""
    buf = bytearray(size * size * 4)
    cx = (size - 1) / 2.0
    cy = (size - 1) / 2.0
    r_core = size * 0.21
    r_inner = size * 0.30
    r_outer = size * 0.43

    for y in range(size):
        for x in range(size):
            dx = x - cx
            dy = y - cy
            dist = math.hypot(dx, dy)
            alpha = 1.0 - smoothstep(r_core - 1.0, r_core + 0.8, dist)

            # 8 rays
            if r_inner - 1.0 <= dist <= r_outer + 1.0:
                angle = math.atan2(dy, dx)
                for k in range(8):
                    ray_a = k * math.pi / 4.0
                    # distance to line segment from r_inner to r_outer
                    rx, ry = math.cos(ray_a), math.sin(ray_a)
                    proj = dx * rx + dy * ry
                    perp = abs(-dx * ry + dy * rx)
                    clamped_proj = clamp(proj, r_inner, r_outer)
                    d_seg = math.hypot(dx - clamped_proj * rx, dy - clamped_proj * ry)
                    ray_alpha = 1.0 - smoothstep(size * 0.032, size * 0.055, d_seg)
                    alpha = max(alpha, ray_alpha)

            if alpha > 0.0:
                idx = (y * size + x) * 4
                buf[idx] = 255
                buf[idx + 1] = 255
                buf[idx + 2] = 255
                buf[idx + 3] = int(clamp(alpha * 255, 0, 255))
    return bytes(buf)


def render_tray_moon(size: int = 44) -> bytes:
    """Monochrome template crescent moon icon for menu bar / tray."""
    buf = bytearray(size * size * 4)
    cx = (size - 1) / 2.0
    cy = (size - 1) / 2.0
    r_outer = size * 0.36
    cut_cx = cx + size * 0.16
    cut_cy = cy - size * 0.11
    r_cut = size * 0.31

    for y in range(size):
        for x in range(size):
            d1 = math.hypot(x - cx, y - cy)
            d2 = math.hypot(x - cut_cx, y - cut_cy)
            a1 = 1.0 - smoothstep(r_outer - 1.0, r_outer + 0.8, d1)
            a2 = smoothstep(r_cut - 1.0, r_cut + 0.8, d2)
            alpha = a1 * a2
            if alpha > 0.0:
                idx = (y * size + x) * 4
                buf[idx] = 255
                buf[idx + 1] = 255
                buf[idx + 2] = 255
                buf[idx + 3] = int(clamp(alpha * 255, 0, 255))
    return bytes(buf)


def write_ico(path: str, png_map: dict[int, bytes]) -> None:
    sizes = sorted(png_map.keys())
    header = struct.pack("<HHH", 0, 1, len(sizes))
    entries = bytearray()
    offset = 6 + 16 * len(sizes)
    blobs = bytearray()
    for sz in sizes:
        data = png_map[sz]
        dim = 0 if sz >= 256 else sz
        entries.extend(struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(data), offset))
        blobs.extend(data)
        offset += len(data)
    with open(path, "wb") as f:
        f.write(header + bytes(entries) + bytes(blobs))


def main() -> None:
    # Standard Tauri PNG icons
    for name, sz in [("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256), ("icon.png", 512)]:
        write_png(os.path.join(ICONS_DIR, name), sz, sz, render_app_icon(sz))

    # Tray icons (Sun & Moon)
    write_png(os.path.join(ICONS_DIR, "tray-sun.png"), 44, 44, render_tray_sun(44))
    write_png(os.path.join(ICONS_DIR, "tray-moon.png"), 44, 44, render_tray_moon(44))

    # Windows .ico
    ico_pngs = {}
    for sz in (16, 32, 48, 256):
        tmp_path = os.path.join(ICONS_DIR, f"_tmp_{sz}.png")
        write_png(tmp_path, sz, sz, render_app_icon(sz))
        with open(tmp_path, "rb") as f:
            ico_pngs[sz] = f.read()
        os.remove(tmp_path)
    write_ico(os.path.join(ICONS_DIR, "icon.ico"), ico_pngs)

    # macOS .icns via iconutil if available
    iconset = os.path.join(ICONS_DIR, "Solace.iconset")
    os.makedirs(iconset, exist_ok=True)
    icns_sizes = [
        ("icon_16x16.png", 16),
        ("icon_16x16@2x.png", 32),
        ("icon_32x32.png", 32),
        ("icon_32x32@2x.png", 64),
        ("icon_128x128.png", 128),
        ("icon_128x128@2x.png", 256),
        ("icon_256x256.png", 256),
        ("icon_256x256@2x.png", 512),
        ("icon_512x512.png", 512),
    ]
    for fname, sz in icns_sizes:
        write_png(os.path.join(iconset, fname), sz, sz, render_app_icon(sz))

    icns_out = os.path.join(ICONS_DIR, "icon.icns")
    res = subprocess.run(["iconutil", "-c", "icns", iconset, "-o", icns_out], capture_output=True)
    if res.returncode != 0:
        print("Warning: iconutil failed:", res.stderr.decode())
    # Clean up iconset dir
    for fname, _ in icns_sizes:
        os.remove(os.path.join(iconset, fname))
    os.rmdir(iconset)
    print("Generated all Solace icons in", ICONS_DIR)


if __name__ == "__main__":
    main()
