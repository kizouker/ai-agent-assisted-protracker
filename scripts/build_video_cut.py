#!/usr/bin/env python3
"""Build the hackathon video cut (docs/video-script.md): cards + VHS clips + captions + optional macOS TTS.

Adapted from adaptive-IVR-agent/data/video/cut/build_cut.py.
If a VHS clip under data/video/term/ does not exist yet, automatically falls back to a styled card
so you can build and preview the full video at any stage of the project.

Run from repo root:
  uv run --no-project --with pillow python scripts/build_video_cut.py --tts
"""
import subprocess
import sys
import textwrap
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
VID = ROOT / "data" / "video"
OUT = VID / "cut"
TMP = OUT / "build"
W, H, FPS = 1920, 1080, 30

# Retro Amiga / ProTracker dark palette
BG = (12, 14, 22)
FG = (235, 238, 245)
DIM = (140, 150, 170)
ACCENT = (0, 210, 220)
AMIGA_AMBER = (255, 175, 65)

SANS = "/System/Library/Fonts/SFNS.ttf"
MONO = "/System/Library/Fonts/Menlo.ttc"

FIT = "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2:color=0x0c0e16"


def font(path: str, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(path, size)


def card(path: Path, lines: list[tuple[int, tuple[int, int, int], str]]) -> None:
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text(
        (160, 100),
        "AI-AGENT-ASSISTED (AAA) PROTRACKER  ·  KIROWEEN 2026 (RESURRECTION)",
        font=font(MONO, 28),
        fill=DIM,
    )
    y = 220
    for size, color, text in lines:
        use_mono = text.startswith("[") or "|" in text or "0xy" in text
        f = font(MONO if use_mono else SANS, size)
        wrap_width = 58 if size < 52 else 32
        for chunk in textwrap.wrap(text, wrap_width):
            d.text((160, y), chunk, font=f, fill=color)
            y += int(size * 1.35)
        y += 22
    img.save(path)


def caption(path: Path, text: str) -> None:
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    if text:
        d = ImageDraw.Draw(img)
        f = font(SANS, 42)
        rows = textwrap.wrap(text, 64)
        line_h = 56
        box_h = line_h * len(rows) + 38
        top = H - 64 - box_h
        d.rounded_rectangle((120, top, W - 120, top + box_h), radius=18, fill=(0, 0, 0, 185))
        for i, row in enumerate(rows):
            tw = d.textlength(row, font=f)
            d.text(((W - tw) / 2, top + 18 + i * line_h), row, font=f, fill=FG)
    img.save(path)


# (id, source_rel_to_data_video, in_s, out_s, speed, fit, dur_s, say_text)
# First 30 seconds (A1 + A2 + A3): Amiga 500, Paula 8364, MOD format history & Demoscene.
SEGMENTS = [
    (
        "A1",
        None,
        0,
        0,
        1,
        None,
        10,
        "In 1987, the Commodore Amiga 500 gave bedroom musicians a secret weapon: "
        "the MOS 8364 Paula sound chip—four hardware DMA channels playing raw eight-bit samples.",
    ),
    (
        "A2",
        None,
        0,
        0,
        1,
        None,
        10,
        "Karsten Obarski's Ultimate Soundtracker and ProTracker created the MOD format: "
        "thirty-one samples, sixty-four hexadecimal rows per pattern, all packed in one tiny file.",
    ),
    (
        "A3",
        None,
        0,
        0,
        1,
        None,
        10,
        "Across the European demoscene, composers squeezed full symphonies out of four channels "
        "using fast zero-x-y arpeggios, sample offsets, and hardware period tables.",
    ),
    (
        "B1",
        None,
        0,
        0,
        1,
        None,
        12,
        "Built from scratch for Kiroween 2026 in the Resurrection category, "
        "AI-Agent-Assisted ProTracker brings the classic Amiga MOD format into the AI agent era.",
    ),
    (
        "B2",
        "term/v1-inspect.mp4",
        0.0,
        20.0,
        1,
        FIT,
        20,
        "First, we specified the binary ProTracker format byte for byte in Kiro. "
        "One command inspects any MOD file's thirty-one samples and sixty-four-row patterns.",
    ),
    (
        "B3",
        "term/v2-compose.mp4",
        0.0,
        30.0,
        1,
        FIT,
        30,
        "Tell the agent in plain English: add a syncopated bassline on channel two that locks into "
        "the kick and snare on channel one, and minor arpeggios on channel three.",
    ),
    (
        "C1",
        "term/v3-pt2-clone.mp4",
        0.0,
        23.0,
        1,
        FIT,
        23,
        "Open the generated MOD file in ProTracker 2 on macOS—or on a real Amiga 500. "
        "Every note uses exact Amiga Paula periods and lossless eight-bit PCM samples.",
    ),
    (
        "D1",
        "term/v4-tests.mp4",
        0.0,
        15.0,
        1,
        FIT,
        15,
        "Every Kiro EARS requirement is verified: byte-identical round-trip parsing, "
        "Paula hardware bounds, and two-byte sample silence.",
    ),
    (
        "E1",
        None,
        0,
        0,
        1,
        None,
        10,
        "Four channels. Eight-bit samples. Sixty-four rows. "
        "AI-Agent-Assisted ProTracker—resurrected with Kiro.",
    ),
]

CARDS = {
    "A1": [
        (82, AMIGA_AMBER, "1987 · Commodore Amiga 500"),
        (46, FG, "The MOS 8364 'Paula' Audio Chip"),
        (36, ACCENT, "4 Hardware DMA Voices (Left · Right · Right · Left)"),
        (36, FG, "8-bit signed PCM samples  ·  6-bit volume (0..64)  ·  12-bit hardware periods"),
    ],
    "A2": [
        (82, AMIGA_AMBER, "The Birth of .MOD (1987–1990)"),
        (46, FG, "Ultimate Soundtracker → NoiseTracker → ProTracker"),
        (36, ACCENT, "[0..1083 Header | M.K. | 64-Row Patterns | 8-Bit PCM Samples]"),
        (36, FG, "Up to 31 instruments and 64 patterns of 4-channel hexadecimal music in a single file."),
    ],
    "A3": [
        (82, AMIGA_AMBER, "The Demoscene & Tracker Tricks"),
        (46, FG, "Making 4 Channels Sound Like 12"),
        (36, ACCENT, "C-2 01 C40  |  E-2 02 037 (Minor Chord Arpeggio)  |  G-2 03 908 (Offset)"),
        (36, FG, "Demoscene legends shared .MOD files with full source visibility—every sample and effect open to learn from."),
    ],
    "B1": [
        (82, ACCENT, "AI-Agent-Assisted (AAA) ProTracker"),
        (44, FG, "Resurrecting 1980s Amiga Tracker Music with Kiro"),
        (36, AMIGA_AMBER, "Prompt in natural language → Agent writes 64-row patterns → Tweak in ProTracker 2"),
        (36, DIM, "Built from scratch Nov 2026 for Kiroween  ·  100% Real Amiga Compatible"),
    ],
    "B2": [
        (72, ACCENT, "Step 1 · Byte-for-Byte MOD Inspection"),
        (40, FG, "Kiro Spec-Driven Binary Parser (M.K. / 31 Samples / 64 Rows)"),
        (34, AMIGA_AMBER, "[VHS Placeholder: data/video/term/v1-inspect.mp4]"),
        (34, FG, "00 | C-2 01 C40 | --- 00 000 | --- 00 000 | --- 00 F06"),
    ],
    "B3": [
        (72, ACCENT, "Step 2 · Co-Composing with the MOD-Agent"),
        (40, FG, '"Add a syncopated bassline on channel 2 that locks into the drums on channel 1"'),
        (34, AMIGA_AMBER, "[VHS Placeholder: data/video/term/v2-compose.mp4]"),
        (34, FG, "00 | C-2 01 C40 | C-1 02 C38 | A-2 03 037 | --- 00 000"),
    ],
    "C1": [
        (72, ACCENT, "Step 3 · Round-Trip in ProTracker 2 (pt2-clone)"),
        (40, FG, "Open in ProTracker 2 on macOS or transfer to a real Amiga 500"),
        (34, AMIGA_AMBER, "[Clip Placeholder: data/video/term/v3-pt2-clone.mp4]"),
        (34, FG, "Lossless round-trip: untouched channels, patterns, and 8-bit PCM samples stay byte-identical."),
    ],
    "D1": [
        (72, ACCENT, "Step 4 · Verified Against Kiro EARS Requirements"),
        (40, FG, "Round-trip SHA-256 identity  ·  Amiga period table (856..113)  ·  Paula 2-byte loop silence"),
        (34, AMIGA_AMBER, "[VHS Placeholder: data/video/term/v4-tests.mp4]"),
    ],
    "E1": [
        (86, FG, "AI-Agent-Assisted ProTracker"),
        (42, ACCENT, "github.com/kizouker/ai-agent-assisted-protracker"),
        (38, AMIGA_AMBER, "Kiroween 2026  ·  Category: Resurrection"),
        (38, FG, "4 Channels. 8-Bit Samples. 64 Rows. Resurrected with Kiro."),
    ],
}


def run(cmd: list[str]) -> None:
    subprocess.run(cmd, check=True)


def render(seg: tuple) -> Path:
    sid, src, t_in, t_out, speed, fit, dur, text = seg
    cap = TMP / f"{sid}-cap.png"
    caption(cap, text)
    out = TMP / f"{sid}.mp4"
    clip_path = (VID / src) if src else None
    if clip_path is None or not clip_path.exists():
        png = TMP / f"{sid}-card.png"
        card(png, CARDS[sid])
        inputs = ["-loop", "1", "-t", str(dur), "-i", str(png)]
        vf = "[0:v]format=rgb24"
    else:
        inputs = ["-i", str(clip_path)]
        vf = (
            f"[0:v]trim={t_in}:{t_out},setpts=(PTS-STARTPTS)/{speed},{fit},"
            f"tpad=stop_mode=clone:stop_duration=30"
        )
    graph = f"{vf},fps={FPS}[v];[v][1:v]overlay=0:0,format=yuv420p,trim=duration={dur}[o]"
    run(
        [
            "ffmpeg",
            "-v",
            "error",
            "-y",
            *inputs,
            "-i",
            str(cap),
            "-filter_complex",
            graph,
            "-map",
            "[o]",
            "-an",
            "-c:v",
            "libx264",
            "-crf",
            "18",
            "-r",
            str(FPS),
            str(out),
        ]
    )
    return out


def build_bed(total_dur: float) -> Path | None:
    """Build bed.wav from ProTracker 2 recording (if present) for intro, C1 demo, and outro."""
    pt2_clip = VID / "term/v3-pt2-clone.mp4"
    out = OUT / "bed.wav"
    if not pt2_clip.exists():
        out.unlink(missing_ok=True)
        return None
    bed_map = {
        "A1": (0.0, 0.15),
        "A2": (10.0, 0.15),
        "A3": (20.0, 0.15),
        "C1": (0.0, 0.28),
        "E1": (18.0, 0.22),
    }
    inputs: list[str] = []
    chains: list[str] = []
    t = 0.0
    n = 0
    for sid, *_, dur, _text in SEGMENTS:
        if sid in bed_map:
            start, gain = bed_map[sid]
            inputs += ["-i", str(pt2_clip)]
            ms = int(t * 1000)
            chains.append(
                f"[{n}:a]atrim={start}:{start + dur},asetpts=PTS-STARTPTS,"
                f"afade=t=in:d=0.4,afade=t=out:st={max(0.0, dur - 0.8)}:d=0.8,volume={gain},"
                f"aresample=48000,aformat=channel_layouts=stereo,adelay={ms}|{ms}[b{n}]"
            )
            n += 1
        t += dur
    labels = "".join(f"[b{i}]" for i in range(n))
    graph = ";".join(chains) + (
        f";{labels}amix=inputs={n}:normalize=0,aresample=48000,apad,atrim=0:{total_dur}[a]"
    )
    run(
        [
            "ffmpeg",
            "-v",
            "error",
            "-y",
            *inputs,
            "-filter_complex",
            graph,
            "-map",
            "[a]",
            "-ar",
            "48000",
            str(out),
        ]
    )
    return out


def synthesize_segment_tts(sid: str, text: str, tts_dir: Path) -> Path:
    """Generate natural English neural TTS via edge-tts, with English macOS `say` fallback."""
    mp3 = tts_dir / f"{sid}.mp3"
    if mp3.exists() and mp3.stat().st_size > 0:
        return mp3
    try:
        run(
            [
                "uv",
                "run",
                "--no-project",
                "--with",
                "edge-tts",
                "edge-tts",
                "--voice",
                "en-US-ChristopherNeural",
                "--rate=+12%",
                "--text",
                text,
                "--write-media",
                str(mp3),
            ]
        )
        return mp3
    except subprocess.CalledProcessError:
        aiff = tts_dir / f"{sid}.aiff"
        run(["say", "-v", "Samantha", "-r", "185", "-o", str(aiff), text])
        return aiff


def build_tts_mux(silent_mp4: Path, total_dur: float, bed_wav: Path | None) -> Path:
    """Generate English narration per segment and mux (+ optional MOD bed) into a preview MP4."""
    tts_dir = TMP / "tts"
    tts_dir.mkdir(parents=True, exist_ok=True)
    inputs: list[str] = []
    chains: list[str] = []
    t = 0.0
    for idx, (sid, *_, dur, text) in enumerate(SEGMENTS):
        audio_file = synthesize_segment_tts(sid, text, tts_dir)
        inputs += ["-i", str(audio_file)]
        ms = int(t * 1000)
        chains.append(
            f"[{idx}:a]aresample=48000,aformat=channel_layouts=stereo,"
            f"atrim=0:{dur},asetpts=PTS-STARTPTS,adelay={ms}|{ms}[a{idx}]"
        )
        t += dur
    labels = "".join(f"[a{i}]" for i in range(len(chains)))
    graph = ";".join(chains) + (
        f";{labels}amix=inputs={len(chains)}:normalize=0,"
        f"loudnorm=I=-16:TP=-1.5:LRA=11,aresample=48000,apad,atrim=0:{total_dur}[aout]"
    )
    narration_wav = OUT / "narration.wav"
    run(
        [
            "ffmpeg",
            "-v",
            "error",
            "-y",
            *inputs,
            "-filter_complex",
            graph,
            "-map",
            "[aout]",
            "-ar",
            "48000",
            str(narration_wav),
        ]
    )
    preview_mp4 = OUT / "AAA-PROTRACKER-PREVIEW.mp4"
    if bed_wav and bed_wav.exists():
        run(
            [
                "ffmpeg",
                "-v",
                "error",
                "-y",
                "-i",
                str(silent_mp4),
                "-i",
                str(narration_wav),
                "-i",
                str(bed_wav),
                "-filter_complex",
                "[1:a][2:a]amix=inputs=2:normalize=0,aresample=48000[mix]",
                "-map",
                "0:v",
                "-map",
                "[mix]",
                "-c:v",
                "copy",
                "-c:a",
                "aac",
                "-b:a",
                "192k",
                "-ar",
                "48000",
                "-shortest",
                str(preview_mp4),
            ]
        )
    else:
        run(
            [
                "ffmpeg",
                "-v",
                "error",
                "-y",
                "-i",
                str(silent_mp4),
                "-i",
                str(narration_wav),
                "-c:v",
                "copy",
                "-c:a",
                "aac",
                "-b:a",
                "192k",
                "-ar",
                "48000",
                "-shortest",
                str(preview_mp4),
            ]
        )
    return preview_mp4


def timecode(s: float) -> str:
    return f"{int(s // 60)}:{s % 60:04.1f}"


def main() -> None:
    TMP.mkdir(parents=True, exist_ok=True)
    parts = [render(s) for s in SEGMENTS]
    lst = TMP / "list.txt"
    lst.write_text("".join(f"file '{p}'\n" for p in parts))
    silent_mp4 = OUT / "silent.mp4"
    run(
        [
            "ffmpeg",
            "-v",
            "error",
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            str(lst),
            "-c",
            "copy",
            str(silent_mp4),
        ]
    )
    rows, t = [], 0.0
    for sid, src, *_, dur, text in SEGMENTS:
        rows.append(f"| {timecode(t)} | {timecode(t + dur)} | {sid} | {src or 'card'} | {text} |")
        t += dur
    (OUT / "vo-sheet.md").write_text(
        "# Voiceover sheet for silent.mp4 (generated by build_video_cut.py)\n\n"
        "Read each line while its segment plays. Captions on screen = these lines.\n\n"
        "| From | To | Seg | Picture | Say |\n|---|---|---|---|---|\n"
        + "\n".join(rows)
        + f"\n\nTotal: {timecode(t)} (hard stop 3:00).\n"
    )
    bed_wav = build_bed(t)
    print(f"Built {silent_mp4} ({t:.1f} s) and {OUT / 'vo-sheet.md'}")
    if "--tts" in sys.argv:
        preview = build_tts_mux(silent_mp4, t, bed_wav)
        print(f"Built narrated preview: {preview}")


if __name__ == "__main__":
    main()
