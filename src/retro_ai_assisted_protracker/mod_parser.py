"""Load a 4-channel ProTracker .mod file from disk into a ModFile."""

from __future__ import annotations

from pathlib import Path

from retro_ai_assisted_protracker.mod_model import (
    BYTES_PER_PATTERN,
    HEADER_SIZE,
    MAX_SAMPLES,
    ORDER_TABLE_SIZE,
    SUPPORTED_TAGS,
    ModFile,
    Pattern,
    SampleHeader,
)


class ModParseError(ValueError):
    """Raised when a file cannot be parsed as a valid .mod file."""


def load(path: str | Path) -> ModFile:
    """Parse a ProTracker .mod file and return a ModFile.

    Raises ModParseError if the format tag is unsupported or the file is truncated.
    """
    data = Path(path).read_bytes()
    _validate_header(data)

    mod = ModFile()

    # Song title (bytes 0..19)
    mod.title = data[:20].rstrip(b"\x00").decode("latin-1", errors="replace")

    # 31 sample headers (bytes 20..949)
    for i in range(MAX_SAMPLES):
        offset = 20 + i * 30
        mod.samples[i] = SampleHeader.from_bytes(data[offset : offset + 30])

    # Song length (byte 950)
    mod.song_length = data[950]
    if not (1 <= mod.song_length <= 128):
        raise ModParseError(
            f"Invalid song length {mod.song_length} at byte 950; expected 1..128."
        )

    # Restart byte (byte 951)
    mod.restart_byte = data[951]

    # Pattern order table (bytes 952..1079)
    mod.order_table = list(data[952:1080])

    # Format tag (bytes 1080..1083)
    mod.format_tag = data[1080:1084]
    if mod.format_tag not in SUPPORTED_TAGS:
        raise ModParseError(
            f"Unsupported format tag {mod.format_tag!r} at byte offset 1080. "
            f"Supported: {sorted(t.decode() for t in SUPPORTED_TAGS)}."
        )

    # Number of patterns
    n_patterns = max(mod.order_table) + 1
    required_size = HEADER_SIZE + n_patterns * BYTES_PER_PATTERN
    if len(data) < required_size:
        raise ModParseError(
            f"File is truncated: need {required_size} bytes for {n_patterns} patterns "
            f"but only {len(data)} bytes available."
        )

    # Patterns
    mod.patterns = []
    for i in range(n_patterns):
        start = HEADER_SIZE + i * BYTES_PER_PATTERN
        mod.patterns.append(Pattern.from_bytes(data[start : start + BYTES_PER_PATTERN]))

    # Sample PCM data
    sample_offset = HEADER_SIZE + n_patterns * BYTES_PER_PATTERN
    mod.sample_data = []
    for i in range(MAX_SAMPLES):
        byte_len = mod.samples[i].byte_length
        if len(data) < sample_offset + byte_len:
            raise ModParseError(
                f"File truncated reading sample {i + 1} PCM data at byte offset {sample_offset}; "
                f"need {byte_len} bytes but only {len(data) - sample_offset} remain."
            )
        mod.sample_data.append(data[sample_offset : sample_offset + byte_len])
        sample_offset += byte_len

    return mod


def _validate_header(data: bytes) -> None:
    if len(data) < HEADER_SIZE:
        raise ModParseError(
            f"File is too short: {len(data)} bytes; a valid .mod header requires {HEADER_SIZE} bytes."
        )
    tag = data[1080:1084]
    if tag not in SUPPORTED_TAGS:
        raise ModParseError(
            f"Unsupported format tag {tag!r} at byte offset 1080. "
            f"Supported: {sorted(t.decode() for t in SUPPORTED_TAGS)}."
        )
