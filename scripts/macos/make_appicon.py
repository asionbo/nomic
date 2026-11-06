#!/usr/bin/env python3
"""Generate the macOS app icon (.iconset PNGs) with pure-stdlib rendering.

Usage: python3 make_appicon.py <output.iconset>

Renders the same motif as the tray icon (src/icon.rs) — a signal ring with
a dot on it — in white on a black rounded square, and writes the 10 PNGs
that `iconutil -c icns` expects. No image libraries: pixels are drawn with
signed-distance functions and anti-aliased analytically.
"""
import math
import os
import struct
import sys
import zlib

SIZES = [
    ("icon_16x16.png", 16),
    ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32),
    ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128),
    ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256),
    ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512),
    ("icon_512x512@2x.png", 1024),
]

BG = (0.06, 0.06, 0.07)  # near-black, keeps pure highlights crisp
FG = (1.0, 1.0, 1.0)     # white ring + dot


def sd_rounded_rect(px: float, py: float, half: float, radius: float) -> float:
    qx = abs(px) - (half - radius)
    qy = abs(py) - (half - radius)
    ax, ay = max(qx, 0.0), max(qy, 0.0)
    return math.hypot(ax, ay) + min(max(qx, qy), 0.0) - radius


def render(size: int) -> bytes:
    """Return size*size*4 RGBA bytes for one icon size."""
    rgba = bytearray(size * size * 4)
    half = size / 2.0
    box_half = half - 0.5
    radius = size * 0.225          # macOS-style corner radius
    ring_r = size * 0.27           # ring radius
    ring_w = size * 0.075          # ring thickness
    dot_r = size * 0.088
    ring_cy = -size * 0.03         # ring sits slightly above center
    # Signal dot sits on the ring, lower-right.
    dot_cx = ring_r * math.cos(-math.pi / 4)
    dot_cy = ring_cy - ring_r * math.sin(-math.pi / 4)
    gap = dot_r + size * 0.030     # hole carved around the dot so it floats off the ring

    for y in range(size):
        py = y + 0.5 - half
        row = y * size * 4
        for x in range(size):
            px = x + 0.5 - half
            d_box = sd_rounded_rect(px, py, box_half, radius)
            a_box = min(1.0, max(0.0, 0.5 - d_box))  # corner anti-aliasing
            if a_box <= 0.0:
                continue
            # Ring, with a circular hole carved around the dot, plus the solid dot.
            d_ring = abs(math.hypot(px, py - ring_cy) - ring_r) - ring_w / 2.0
            d_dot = math.hypot(px - dot_cx, py - dot_cy)
            ring_cov = min(1.0, max(0.0, 0.5 - d_ring))
            hole_cov = min(1.0, max(0.0, gap - d_dot + 0.5))
            dot_cov = min(1.0, max(0.0, dot_r - d_dot + 0.5))
            w = max(ring_cov * (1.0 - hole_cov), dot_cov)
            i = row + x * 4
            rgba[i] = int(min(255.0, max(0.0, BG[0] * 255.0 + (FG[0] * 255.0 - BG[0] * 255.0) * w)))
            rgba[i + 1] = int(min(255.0, max(0.0, BG[1] * 255.0 + (FG[1] * 255.0 - BG[1] * 255.0) * w)))
            rgba[i + 2] = int(min(255.0, max(0.0, BG[2] * 255.0 + (FG[2] * 255.0 - BG[2] * 255.0) * w)))
            rgba[i + 3] = int(a_box * 255.0)
    return bytes(rgba)


def write_png(path: str, size: int, rgba: bytes) -> None:
    stride = size * 4
    raw = b"".join(b"\x00" + rgba[y * stride:(y + 1) * stride] for y in range(size))

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (struct.pack(">I", len(data)) + tag + data
                + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF))

    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n"
                + chunk(b"IHDR", ihdr)
                + chunk(b"IDAT", zlib.compress(raw, 9))
                + chunk(b"IEND", b""))


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit(f"usage: {sys.argv[0]} <output.iconset>")
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    for name, size in SIZES:
        write_png(os.path.join(out, name), size, render(size))
        print(f"wrote {out}/{name}")


if __name__ == "__main__":
    main()
