"""Data model for 4-channel ProTracker .mod files (31 samples, M.K. family)."""

from __future__ import annotations

import struct
from dataclasses import dataclass, field

SUPPORTED_TAGS: frozenset[bytes] = frozenset(
    [b"M.K.", b"M!K!", b"FLT4", b"4CHN"]
)

PATTERN_ROWS = 64
CHANNELS = 4
BYTES_PER_CELL = 4
BYTES_PER_ROW = CHANNELS * BYTES_PER_CELL
BYTES_PER_PATTERN = PATTERN_ROWS * BYTES_PER_ROW  # 1024

MAX_SAMPLES = 31
ORDER_TABLE_SIZE = 128
HEADER_SIZE = 1084  # bytes 0..1083

# Amiga period table, finetune 0 — 3 octaves (C-1 .. B-3), index 0=C-1 .. 35=B-3
NOTE_NAMES = ["C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-"]
PERIOD_TABLE: dict[str, int] = {}
_PERIODS = [
    # octave 1
    856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453,
    # octave 2
    428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226,
    # octave 3
    214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113,
]
for _i, _p in enumerate(_PERIODS):
    _note = NOTE_NAMES[_i % 12] + str((_i // 12) + 1)  # e.g. "C-1", "C#2"
    PERIOD_TABLE[_note] = _p

PERIOD_TO_NOTE: dict[int, str] = {v: k for k, v in PERIOD_TABLE.items()}


@dataclass
class SampleHeader:
    """30-byte sample header (1-indexed slot 1..31)."""

    name: str = ""  # up to 22 chars
    length_words: int = 0  # byte length = words * 2
    finetune: int = 0  # 0x0..0xF, lower 4 bits; 8..15 = -8..-1
    volume: int = 0  # 0..64
    repeat_offset_words: int = 0
    repeat_length_words: int = 1  # ProTracker writes 1 for no-loop

    @property
    def byte_length(self) -> int:
        return self.length_words * 2

    @property
    def has_loop(self) -> bool:
        return self.repeat_length_words > 1

    def to_bytes(self) -> bytes:
        name_b = self.name.encode("latin-1", errors="replace")[:22].ljust(22, b"\x00")
        return (
            name_b
            + struct.pack(">H", self.length_words)
            + bytes([self.finetune & 0x0F, self.volume & 0xFF])
            + struct.pack(">HH", self.repeat_offset_words, self.repeat_length_words)
        )

    @classmethod
    def from_bytes(cls, data: bytes) -> "SampleHeader":
        assert len(data) == 30
        name = data[:22].rstrip(b"\x00").decode("latin-1", errors="replace")
        (length_words,) = struct.unpack_from(">H", data, 22)
        finetune = data[24] & 0x0F
        volume = data[25]
        (repeat_offset_words,) = struct.unpack_from(">H", data, 26)
        (repeat_length_words,) = struct.unpack_from(">H", data, 28)
        return cls(
            name=name,
            length_words=length_words,
            finetune=finetune,
            volume=volume,
            repeat_offset_words=repeat_offset_words,
            repeat_length_words=repeat_length_words,
        )


@dataclass
class NoteCell:
    """4-byte cell: sample (0..31), period (0..4095), effect cmd (0..15), effect arg (0..255)."""

    sample: int = 0
    period: int = 0
    effect_cmd: int = 0
    effect_arg: int = 0

    def note_name(self) -> str:
        """Return ProTracker text notation, e.g. 'C-2 01 C40' or '--- 00 000'."""
        if self.period == 0:
            note = "---"
        else:
            note = PERIOD_TO_NOTE.get(self.period, f"?{self.period:03d}")
        return f"{note} {self.sample:02X} {self.effect_cmd:X}{self.effect_arg:02X}"

    def to_bytes(self) -> bytes:
        b0 = (self.sample & 0xF0) | ((self.period >> 8) & 0x0F)
        b1 = self.period & 0xFF
        b2 = ((self.sample & 0x0F) << 4) | (self.effect_cmd & 0x0F)
        b3 = self.effect_arg & 0xFF
        return bytes([b0, b1, b2, b3])

    @classmethod
    def from_bytes(cls, data: bytes) -> "NoteCell":
        assert len(data) == 4
        b0, b1, b2, b3 = data
        sample = (b0 & 0xF0) | ((b2 >> 4) & 0x0F)
        period = ((b0 & 0x0F) << 8) | b1
        effect_cmd = b2 & 0x0F
        effect_arg = b3
        return cls(sample=sample, period=period, effect_cmd=effect_cmd, effect_arg=effect_arg)


@dataclass
class Pattern:
    """64 rows × 4 channels of NoteCells."""

    rows: list[list[NoteCell]] = field(
        default_factory=lambda: [[NoteCell() for _ in range(CHANNELS)] for _ in range(PATTERN_ROWS)]
    )

    def to_bytes(self) -> bytes:
        out = bytearray()
        for row in self.rows:
            for cell in row:
                out += cell.to_bytes()
        return bytes(out)

    @classmethod
    def from_bytes(cls, data: bytes) -> "Pattern":
        assert len(data) == BYTES_PER_PATTERN
        rows = []
        for r in range(PATTERN_ROWS):
            row = []
            for c in range(CHANNELS):
                offset = (r * CHANNELS + c) * BYTES_PER_CELL
                row.append(NoteCell.from_bytes(data[offset : offset + BYTES_PER_CELL]))
            rows.append(row)
        return cls(rows=rows)


@dataclass
class ModFile:
    """Full in-memory representation of a 4-channel ProTracker .mod file."""

    title: str = ""
    samples: list[SampleHeader] = field(default_factory=lambda: [SampleHeader() for _ in range(MAX_SAMPLES)])
    song_length: int = 1  # 1..128
    restart_byte: int = 0x7F
    order_table: list[int] = field(default_factory=lambda: [0] * ORDER_TABLE_SIZE)
    format_tag: bytes = b"M.K."
    patterns: list[Pattern] = field(default_factory=list)
    sample_data: list[bytes] = field(default_factory=lambda: [b""] * MAX_SAMPLES)

    @property
    def pattern_count(self) -> int:
        return max(self.order_table) + 1 if self.order_table else 1
