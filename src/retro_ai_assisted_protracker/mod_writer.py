"""Write a ModFile back to a .mod file on disk."""

from __future__ import annotations

from pathlib import Path

from retro_ai_assisted_protracker.mod_model import (
    MAX_SAMPLES,
    ORDER_TABLE_SIZE,
    ModFile,
)


def save(mod: ModFile, path: str | Path) -> None:
    """Serialise a ModFile and write it to *path*, creating or overwriting the file."""
    Path(path).write_bytes(_serialise(mod))


def _serialise(mod: ModFile) -> bytes:
    out = bytearray()

    # Song title (20 bytes)
    title_b = mod.title.encode("latin-1", errors="replace")[:20].ljust(20, b"\x00")
    out += title_b

    # 31 sample headers (30 bytes each = 930 bytes)
    for i in range(MAX_SAMPLES):
        out += mod.samples[i].to_bytes()

    # Song length (1 byte)
    out += bytes([mod.song_length & 0xFF])

    # Restart byte (1 byte)
    out += bytes([mod.restart_byte & 0xFF])

    # Pattern order table (128 bytes)
    order = list(mod.order_table)[:ORDER_TABLE_SIZE]
    order += [0] * (ORDER_TABLE_SIZE - len(order))
    out += bytes(order)

    # Format tag (4 bytes)
    out += mod.format_tag[:4].ljust(4, b"\x00")

    # Pattern data
    for pattern in mod.patterns:
        out += pattern.to_bytes()

    # Sample PCM data
    for i in range(MAX_SAMPLES):
        pcm = mod.sample_data[i] if i < len(mod.sample_data) else b""
        # Requirement 3.3: first 2 bytes of non-looped samples must be 0x00
        if pcm and not mod.samples[i].has_loop and len(pcm) >= 2:
            pcm = b"\x00\x00" + pcm[2:]
        out += pcm

    return bytes(out)
