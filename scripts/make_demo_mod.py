"""Generate data/demo.mod – a minimal but real 4-channel M.K. ProTracker file for testing."""

import math
import struct
from pathlib import Path

PATTERN_ROWS = 64
PERIODS = [
    856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453,
    428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226,
    214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113,
]


def note_cell(sample: int, period: int, cmd: int = 0, arg: int = 0) -> bytes:
    b0 = (sample & 0xF0) | ((period >> 8) & 0x0F)
    b1 = period & 0xFF
    b2 = ((sample & 0x0F) << 4) | (cmd & 0x0F)
    b3 = arg & 0xFF
    return bytes([b0, b1, b2, b3])


def build() -> bytes:
    out = bytearray()

    # Song title (20 bytes)
    out += b"Demo Tune\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"

    # Sample 1: Kick Drum  (256 words = 512 bytes, no loop)
    out += b"Kick Drum\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"
    out += struct.pack(">H", 256)
    out += bytes([0, 64])
    out += struct.pack(">HH", 0, 1)

    # Sample 2: Bass Synth (128 words = 256 bytes, full loop)
    out += b"Bass Synth\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"
    out += struct.pack(">H", 128)
    out += bytes([0, 48])
    out += struct.pack(">HH", 0, 128)

    # Samples 3-31: empty
    for _ in range(29):
        out += b"\x00" * 22
        out += struct.pack(">H", 0)
        out += bytes([0, 0])
        out += struct.pack(">HH", 0, 1)

    # Song length=1, restart=0x7F, order table
    out += bytes([1, 0x7F])
    out += bytes([0] * 128)

    # Format tag
    out += b"M.K."

    # Pattern 0
    kick_rows = {0, 16, 32, 48}
    bass_melody = {
        0: PERIODS[12], 4: PERIODS[14], 8: PERIODS[16], 12: PERIODS[19],
        16: PERIODS[12], 20: PERIODS[21], 24: PERIODS[19], 28: PERIODS[17],
        32: PERIODS[12], 36: PERIODS[14], 40: PERIODS[16], 44: PERIODS[19],
        48: PERIODS[23], 52: PERIODS[21], 56: PERIODS[19], 60: PERIODS[16],
    }
    pattern = bytearray()
    for r in range(PATTERN_ROWS):
        # ch0: kick on beats
        pattern += note_cell(1, PERIODS[0], 0xC, 0x40) if r in kick_rows else note_cell(0, 0)
        # ch1: bass melody
        p = bass_melody.get(r, 0)
        pattern += note_cell(2, p) if p else note_cell(0, 0)
        # ch2, ch3: silent
        pattern += note_cell(0, 0)
        pattern += note_cell(0, 0)
    out += bytes(pattern)

    # Kick PCM: exponential decay × sine, 512 bytes, signed 8-bit, first 2 bytes = 0x00
    kick_raw = [int(64 * math.exp(-i / 80) * math.sin(i * 0.6)) for i in range(512)]
    kick_pcm = bytes([b & 0xFF for b in kick_raw])
    kick_pcm = b"\x00\x00" + kick_pcm[2:]
    out += kick_pcm

    # Bass PCM: simple sine loop, 256 bytes
    bass_pcm = bytes([int(40 * math.sin(2 * math.pi * i / 32)) & 0xFF for i in range(256)])
    out += bass_pcm

    return bytes(out)


if __name__ == "__main__":
    dest = Path(__file__).parent.parent / "data" / "demo.mod"
    dest.write_bytes(build())
    print(f"Written {dest.stat().st_size} bytes to {dest}")
