"""Tests for Ticket 1 – binary MOD parser and serialiser.

Covers acceptance criteria 1.1, 1.2, 1.3 from spec.md.
"""

from __future__ import annotations

import hashlib
import struct
import tempfile
from pathlib import Path

import pytest

from retro_ai_assisted_protracker.mod_model import (
    BYTES_PER_PATTERN,
    HEADER_SIZE,
    MAX_SAMPLES,
    ORDER_TABLE_SIZE,
    PATTERN_ROWS,
    NoteCell,
    Pattern,
    SampleHeader,
)
from retro_ai_assisted_protracker.mod_parser import ModParseError, load
from retro_ai_assisted_protracker.mod_writer import save, _serialise


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _build_minimal_mod(
    title: str = "Test Song",
    n_patterns: int = 1,
    tag: bytes = b"M.K.",
    song_length: int = 1,
) -> bytes:
    """Build a syntactically valid minimal .mod file in memory."""
    out = bytearray()

    # Song title
    out += title.encode("latin-1", errors="replace")[:20].ljust(20, b"\x00")

    # 31 sample headers — all empty (length 0, repeat_length 1)
    for _ in range(MAX_SAMPLES):
        out += b"\x00" * 22  # name
        out += struct.pack(">H", 0)   # length words
        out += bytes([0, 0])          # finetune, volume
        out += struct.pack(">HH", 0, 1)  # repeat offset, repeat length (1 = no loop)

    # Song length
    out += bytes([song_length])
    # Restart byte
    out += bytes([0x7F])

    # Pattern order table: first entry is 0, rest 0
    order = [0] * ORDER_TABLE_SIZE
    for i in range(song_length):
        order[i] = i % n_patterns
    out += bytes(order)

    # Format tag
    out += tag

    # n_patterns × 1024 bytes of empty pattern data
    out += b"\x00" * (n_patterns * BYTES_PER_PATTERN)

    # No sample PCM data (all lengths are 0)
    return bytes(out)


# ---------------------------------------------------------------------------
# Acceptance criterion 1.1 – parser reads all fields
# ---------------------------------------------------------------------------

class TestParser:
    def test_reads_title(self) -> None:
        raw = _build_minimal_mod(title="Haunted Tune")
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(raw)
            path = f.name
        mod = load(path)
        assert mod.title == "Haunted Tune"

    def test_reads_31_sample_headers(self) -> None:
        raw = _build_minimal_mod()
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(raw)
            path = f.name
        mod = load(path)
        assert len(mod.samples) == MAX_SAMPLES

    def test_reads_song_length(self) -> None:
        raw = _build_minimal_mod(song_length=3, n_patterns=3)
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(raw)
            path = f.name
        mod = load(path)
        assert mod.song_length == 3

    def test_reads_order_table(self) -> None:
        raw = _build_minimal_mod(song_length=1)
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(raw)
            path = f.name
        mod = load(path)
        assert len(mod.order_table) == ORDER_TABLE_SIZE
        assert mod.order_table[0] == 0

    def test_reads_patterns(self) -> None:
        raw = _build_minimal_mod(n_patterns=2, song_length=2)
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(raw)
            path = f.name
        mod = load(path)
        assert len(mod.patterns) == 2

    def test_reads_pattern_rows(self) -> None:
        raw = _build_minimal_mod()
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(raw)
            path = f.name
        mod = load(path)
        assert len(mod.patterns[0].rows) == PATTERN_ROWS
        assert len(mod.patterns[0].rows[0]) == 4

    def test_reads_sample_pcm(self) -> None:
        """Samples with length 0 produce empty PCM bytes."""
        raw = _build_minimal_mod()
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(raw)
            path = f.name
        mod = load(path)
        assert len(mod.sample_data) == MAX_SAMPLES
        assert mod.sample_data[0] == b""

    def test_accepts_all_supported_tags(self) -> None:
        for tag in [b"M.K.", b"M!K!", b"FLT4", b"4CHN"]:
            raw = _build_minimal_mod(tag=tag)
            with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
                f.write(raw)
                path = f.name
            mod = load(path)
            assert mod.format_tag == tag


# ---------------------------------------------------------------------------
# Acceptance criterion 1.2 – byte-identical round-trip (synthetic)
# ---------------------------------------------------------------------------

class TestRoundTrip:
    def test_synthetic_roundtrip_is_byte_identical(self) -> None:
        raw = _build_minimal_mod(title="Round Trip", n_patterns=2, song_length=2)
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(raw)
            path = f.name
        mod = load(path)
        result = _serialise(mod)
        assert hashlib.sha256(result).hexdigest() == hashlib.sha256(raw).hexdigest(), (
            "Round-trip produced different bytes."
        )

    def test_save_then_load_matches(self) -> None:
        raw = _build_minimal_mod(title="Save Load")
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(raw)
            path = f.name
        mod1 = load(path)
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f2:
            out_path = f2.name
        save(mod1, out_path)
        mod2 = load(out_path)
        assert mod1.title == mod2.title
        assert mod1.song_length == mod2.song_length
        assert mod1.order_table == mod2.order_table

    @pytest.mark.parametrize(
        "mod_file",
        sorted(Path("/Users/raberg/kod/007_hackathons/002_kiroween_2026/ai-agent-assisted-protracker/data").glob("**/*.mod")),
    )
    def test_real_file_roundtrip_byte_identical(self, mod_file: Path) -> None:
        """Round-trip a real .mod file from data/. Skipped when no files exist."""
        raw = mod_file.read_bytes()
        mod = load(mod_file)
        result = _serialise(mod)
        assert hashlib.sha256(result).hexdigest() == hashlib.sha256(raw).hexdigest(), (
            f"Round-trip mismatch for {mod_file.name}"
        )


# ---------------------------------------------------------------------------
# Acceptance criterion 1.3 – validation errors
# ---------------------------------------------------------------------------

class TestValidation:
    def test_rejects_too_short_file(self) -> None:
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(b"\x00" * 100)
            path = f.name
        with pytest.raises(ModParseError, match="too short"):
            load(path)

    def test_rejects_bad_format_tag(self) -> None:
        raw = bytearray(_build_minimal_mod())
        raw[1080:1084] = b"XXXX"
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(raw)
            path = f.name
        with pytest.raises(ModParseError, match="Unsupported format tag"):
            load(path)

    def test_rejects_truncated_pattern_data(self) -> None:
        raw = _build_minimal_mod(n_patterns=2, song_length=2)
        # Chop off the second pattern
        truncated = raw[: HEADER_SIZE + BYTES_PER_PATTERN]
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(truncated)
            path = f.name
        with pytest.raises(ModParseError, match="truncated"):
            load(path)

    def test_error_message_includes_byte_offset_for_bad_tag(self) -> None:
        raw = bytearray(_build_minimal_mod())
        raw[1080:1084] = b"????"
        with tempfile.NamedTemporaryFile(suffix=".mod", delete=False) as f:
            f.write(raw)
            path = f.name
        with pytest.raises(ModParseError, match="1080"):
            load(path)


# ---------------------------------------------------------------------------
# NoteCell encoding/decoding
# ---------------------------------------------------------------------------

class TestNoteCell:
    def test_encode_decode_roundtrip(self) -> None:
        cell = NoteCell(sample=0x1A, period=428, effect_cmd=0xC, effect_arg=0x40)
        assert NoteCell.from_bytes(cell.to_bytes()) == cell

    def test_empty_cell_encodes_to_four_zero_bytes(self) -> None:
        assert NoteCell().to_bytes() == b"\x00\x00\x00\x00"

    def test_note_name_known_period(self) -> None:
        cell = NoteCell(sample=1, period=428, effect_cmd=0, effect_arg=0)
        assert cell.note_name().startswith("C-2")

    def test_note_name_zero_period(self) -> None:
        cell = NoteCell(sample=0, period=0, effect_cmd=0, effect_arg=0)
        assert cell.note_name().startswith("---")


# ---------------------------------------------------------------------------
# SampleHeader encoding/decoding
# ---------------------------------------------------------------------------

class TestSampleHeader:
    def test_encode_decode_roundtrip(self) -> None:
        sh = SampleHeader(
            name="Kick Drum",
            length_words=256,
            finetune=2,
            volume=50,
            repeat_offset_words=0,
            repeat_length_words=1,
        )
        assert SampleHeader.from_bytes(sh.to_bytes()) == sh

    def test_has_loop_false_when_repeat_length_1(self) -> None:
        sh = SampleHeader(repeat_length_words=1)
        assert not sh.has_loop

    def test_has_loop_true_when_repeat_length_gt_1(self) -> None:
        sh = SampleHeader(repeat_length_words=4)
        assert sh.has_loop

    def test_byte_length_is_words_times_2(self) -> None:
        sh = SampleHeader(length_words=100)
        assert sh.byte_length == 200
