# Video Script & Prompt – AI-Agent-Assisted (AAA) ProTracker

Bygger på samma pipeline som i `adaptive-IVR-agent`:
- **Skript som bygger videon i `adaptive-IVR-agent`:** `data/video/cut/build_cut.py` (använder `Pillow` för titelkort + inbrända undertexter och `ffmpeg` för att klippa ihop kort, `vhs`-klipp från `data/video/term/*.mp4`, bakgrundsljud `bed.wav` och röstspår via macOS `say` eller `record-vo.sh`).
- **Här i `ai-agent-assisted-protracker`:** `scripts/build_video_cut.py` och VHS-mall i `data/video/term/tapes/common.tape`.

---

## 1. Storyboard & Manus (med 30 sek Amiga 500, Paula, MOD & Demoscene-intro)

| Tid | Längd | Seg | Bild (`source` i `build_video_cut.py`) | Berättarröst / Inbränd undertext (`SAY`) | Ljudbädd (`BED`) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0:00–0:10` | `10s` | `A1` | Kort: **Amiga 500 & Paula 8364 (1987)** | *"In 1987, the Commodore Amiga 500 gave bedroom musicians a secret weapon: the MOS 8364 Paula sound chip—four hardware DMA channels playing raw eight-bit samples."* | Klassisk MOD-slinga (låg volym) |
| `0:10–0:20` | `10s` | `A2` | Kort: **The Birth of `.MOD` (1987–1990)** | *"Karsten Obarski's Ultimate Soundtracker and ProTracker created the MOD format: thirty-one samples, sixty-four hexadecimal rows per pattern, and song data packed together in one tiny file."* | Klassisk MOD-slinga |
| `0:20–0:30` | `10s` | `A3` | Kort: **The Demoscene & 4-Channel Magic** | *"Across the European demoscene, composers squeezed full symphonies out of four channels using fast zero-x-y arpeggios, sample offsets, and hardware period tables."* | Arpeggio-exempel (`037` / `047`) |
| `0:30–0:42` | `12s` | `B1` | Kort: **Kiroween Resurrection: AAA ProTracker** | *"Built from scratch for Kiroween 2026 in the Resurrection category, AI-Agent-Assisted ProTracker brings the classic Amiga MOD format into the AI agent era."* | Tyst / övertoning |
| `0:42–1:02` | `20s` | `B2` | VHS (`term/v1-inspect.mp4` — eller kort tills CLI är klart) | *"First, we specified the binary ProTracker format byte for byte in Kiro. One command inspects any MOD file's thirty-one samples and sixty-four-row patterns."* | Tangentbord / tyst |
| `1:02–1:32` | `30s` | `B3` | VHS (`term/v2-compose.mp4` — eller kort tills CLI är klart) | *"Tell the agent in plain English: add a syncopated bassline on channel two that locks into the kick and snare on channel one, and minor arpeggios on channel three."* | Kort uppspelning av genererad bas/arpeggio i paus |
| `1:32–1:55` | `23s` | `C1` | Skärmklipp / bild (`ProTracker 2.app` / `pt2-clone`) | *"Open the generated MOD file in ProTracker 2 on macOS—or on a real Amiga 500. Every note uses exact Amiga Paula periods and lossless eight-bit PCM samples."* | Uppspelning från `ProTracker 2.app` |
| `1:55–2:10` | `15s` | `D1` | VHS (`term/v3-tests.mp4` — `uv run pytest`) | *"Every Kiro EARS requirement is verified: byte-identical round-trip parsing, Paula hardware bounds, and two-byte sample header silence."* | Tyst |
| `2:10–2:20` | `10s` | `E1` | Slutkort: **AI-Agent-Assisted (AAA) ProTracker** | *"Four channels. Eight-bit samples. Sixty-four rows. Resurrected with Kiro."* | Slutackord (MOD) |

---

## 2. Färdig Prompt för Video-Agent / Subagent

Kopiera prompten nedan när du vill låta en agent eller subagent generera eller uppdatera hela videon:

Prompt (inte ett terminalkommando):
```text
Bygg och rendera vår hackathon-video för AI-Agent-Assisted (AAA) ProTracker i ~/kod/007_hackathons/002_kiroween_2026/ai-agent-assisted-protracker enligt mönstret från adaptive-IVR-agent:

1. Läs docs/video-script.md och scripts/build_video_cut.py.
2. Om våra CLI-kommandon är klara och vhs finns installerat:
   - Kör .tape-filerna i data/video/term/tapes/ i en ren temp-klon (/tmp/aaa-protracker) så att inga personliga sökvägar (/Users/raberg/...) syns i terminalen.
   - Spara utdata i data/video/term/ (v1-inspect.mp4, v2-compose.mp4, v3-tests.mp4).
   - Om något .mp4-klipp i data/video/term/ saknas ännu ska scripts/build_video_cut.py automatiskt falla tillbaka till ett snyggt infokort för det segmentet så att videon alltid går att bygga direkt.
3. Kör:
   uv run --no-project --with pillow python scripts/build_video_cut.py --tts
   Detta ska:
   - Rendera 1920x1080 (30 fps) kort och klipp med inbrända undertexter (SAY) till data/video/cut/silent.mp4 och data/video/cut/vo-sheet.md.
   - Generera engelsk AI-berättarröst med macOS `say` (rösten Karen eller Samantha om Karen Premium saknas) för varje segment, synkad till segmentets starttid, sampla om till 48000 Hz stereo och mixa ihop till data/video/cut/AAA-PROTRACKER-PREVIEW.mp4.
4. Verifiera med ffprobe att videon är 1920x1080, har 48 kHz ljudspår och är under 180 sekunder.
```
