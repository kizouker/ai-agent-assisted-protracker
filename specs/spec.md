# Requirements Document

## Introduction
AI-Assisted ProTracker (**MOD-Agent**) is a tool for the Kiroween 2026 hackathon (category **Resurrection**) that reads, inspects, and writes classic 4-channel Commodore Amiga ProTracker `.mod` files. The composer describes musical ideas in plain language (such as adding a bassline that fits the drums), the **MOD-Agent** writes notes and effects into the patterns, and the composer opens the resulting `.mod` file in a ProTracker clone (`pt2-clone`) or on a real Amiga to listen and edit by hand.

Source notes: [topic-kiroween-mod-agent.md](../docs/topic-kiroween-mod-agent.md) and byte specification [protracker-mod-format.md](../docs/protracker-mod-format.md).

## Glossary
- **MOD-Agent**: The system that reads, analyzes, edits, and writes ProTracker `.mod` files from user instructions.
- **Module (`.mod`)**: A standard 31-sample, 4-channel ProTracker music file (`M.K.` format tag) containing a 1084-byte header, 64-row patterns, and 8-bit signed PCM samples.
- **Pattern**: A block of 64 rows across 4 channels (`1024` bytes) where each cell holds a sample number, a 12-bit Amiga period, and a 12-bit effect command.
- **Sample**: An 8-bit signed PCM audio instrument (`1..31`) stored in the Module with length, finetune, default volume (`0..64`), and loop words.
- **Period**: A 12-bit hardware timer value from the ProTracker period table (`C-1` = `856` to `B-3` = `113`) that sets pitch on the Amiga Paula chip.

## Requirements

### Requirement 1 – Lossless Reading and Writing of ProTracker MOD Files
**User Story:** As a tracker musician, I want the MOD-Agent to read and write standard 4-channel ProTracker `.mod` files byte for byte without corrupting headers, patterns, or samples, so that I can move freely between the MOD-Agent, `pt2-clone`, and a real Amiga.
**Priority:** Must · **Source:** [topic-kiroween-mod-agent.md](../docs/topic-kiroween-mod-agent.md#L5) ("en AI-agent som läser och skriver ProTracker MOD-filer... Låten ska gå att spela på en riktig Amiga")

#### Acceptance Criteria
1. WHEN a valid 4-channel ProTracker Module (`M.K.`, `M!K!`, `FLT4`, or `4CHN`) is loaded THE MOD-Agent SHALL parse the song title, all 31 Sample headers, the song length, the 128-entry pattern order table, all Patterns, and all 8-bit signed PCM Sample data.
2. WHEN a loaded Module is written back without modifications THE MOD-Agent SHALL produce a byte-identical `.mod` file.
3. IF an input file does not have a supported 4-channel format tag or is truncated before all declared Patterns and Samples are read THEN THE MOD-Agent SHALL reject the file with a clear validation error stating the byte offset and reason.

---

### Requirement 2 – Readable Inspection of Song Structure, Samples, and Patterns
**User Story:** As a tracker musician, I want to inspect the samples, pattern order, and 64-row tracker grid of a `.mod` file in standard ProTracker notation (such as `C-2 01 C40`), so that both I and the AI prompt context can see what is already in the song.
**Priority:** Must · **Source:** [topic-kiroween-mod-agent.md](../docs/topic-kiroween-mod-agent.md#L5) ("läser och skriver ProTracker MOD-filer")

#### Acceptance Criteria
1. WHEN the user requests a summary of a Module THE MOD-Agent SHALL list the song title, song length, pattern count, pattern order table, and all non-empty Samples (`1..31`) with name, byte length, default volume, finetune, and loop status.
2. WHEN the user or prompt builder inspects a Pattern THE MOD-Agent SHALL format each of the 64 rows and 4 channels in standard ProTracker text notation (`Note-Octave SampleNumber EffectCommand`, e.g., `C-2 01 C40` or `--- 00 000`).
3. IF a Pattern cell contains a non-zero Period that does not match an exact Finetune-0 table entry THEN THE MOD-Agent SHALL display the nearest note name alongside the raw Period value so no information is hidden.

---

### Requirement 3 – Amiga Hardware and ProTracker Constraint Validation
**User Story:** As a tracker musician targeting real Amiga hardware, I want the MOD-Agent to enforce ProTracker and Paula chip limits on every generated note, sample, and effect, so that the output always plays cleanly on a real Amiga and in `pt2-clone`.
**Priority:** Must · **Source:** [topic-kiroween-mod-agent.md](../docs/topic-kiroween-mod-agent.md#L17) ("Låten ska gå att spela på en riktig Amiga, alltså fyra kanaler och 8-bitars samplingar")

#### Acceptance Criteria
1. WHEN the MOD-Agent writes notes into a Pattern THE MOD-Agent SHALL convert note names (`C-1` through `B-3`) to exact 12-bit Period values from the ProTracker period table (`856` down to `113`).
2. IF a requested note, sample index (`1..31`), volume (`0..64`), or row index (`0..63`) falls outside valid ProTracker bounds THEN THE MOD-Agent SHALL reject or clamp the edit and report which constraint was violated.
3. WHEN the MOD-Agent writes an unlooped Sample (`repeat_length <= 1` word) THE MOD-Agent SHALL ensure the first 2 bytes of the Sample PCM data are `0x00` so the Amiga Paula chip remains silent after one-shot playback.

---

### Requirement 4 – Co-Composing Patterns from Natural Language Prompts
**User Story:** As a composer, I want to tell the MOD-Agent in plain language what musical part to add or change (for example, *"add a bassline on channel 2 that locks into the drums on channel 1"*), so that I can quickly sketch ideas and then tweak them in `pt2-clone`.
**Priority:** Must · **Source:** [topic-kiroween-mod-agent.md](../docs/topic-kiroween-mod-agent.md#L34) ("Jag säger till exempel 'lägg en bas som passar trummorna', agenten skriver noterna i mönstren, och jag öppnar filen i en ProTracker-klon och ändrar själv")

#### Acceptance Criteria
1. WHEN the user gives a natural-language prompt targeting specific pattern(s) and channel(s) THE MOD-Agent SHALL provide the existing Pattern rows and available Sample list as context and write the generated notes, sample numbers, and effect commands (`0xy`–`Fxx`) into the target cells.
2. WHILE editing a target channel or row range THE MOD-Agent SHALL preserve all untouched channels, untouched patterns, and existing Sample audio data in the Module.
3. WHEN the MOD-Agent finishes applying a prompt edit THE MOD-Agent SHALL save the updated `.mod` file and output a summary of which patterns, channels, and rows were modified.

---

### Requirement 5 – Creating a New Module and Importing 8-Bit Samples (Draft)
**User Story:** As a composer starting a new track from scratch, I want to create a blank `.mod` file and load my own or AI-generated samples into slots `1..31`, so that I do not always have to start from an existing `.mod` file.
**Priority:** Should · **Source:** [topic-kiroween-mod-agent.md](../docs/topic-kiroween-mod-agent.md#L10) ("Ljuden kan komma från egna samplingar, eller från AI om man vill")

#### Acceptance Criteria
1. WHEN the user requests a new Module THE MOD-Agent SHALL initialize a valid 4-channel `M.K.` Module with 1 empty 64-row Pattern, song length `1`, and default speed/BPM (`F06` / `F7D`).
2. WHERE WAV or raw audio sample import is included THE MOD-Agent SHALL convert the input audio to 8-bit signed mono PCM (`<= 65534` words) and store it in the selected Sample slot (`1..31`).

---

## Walking Skeleton Slice
- **Requirements 1, 2, 3, and 4** form the walking skeleton:
  1. Load the downloaded `.mod` file from `data/`.
  2. Parse and display its samples and Pattern 0 in readable tracker format.
  3. Ask the **MOD-Agent** in natural language to write or modify a channel (e.g., a bassline or arpeggio pattern) in a copy of the `.mod` file.
  4. Open the resulting `.mod` file in `ProTracker 2.app` (`pt2-clone-macos`) and play it back.

## Out of Scope
- Commodore 64 SID music (`.sid`) — explicitly deferred for this 3-week hackathon.
- Multi-channel PC tracker formats (`XM`, `IT`, `S3M`, or >4 channel `8CHN` MODs).

## Open Questions
1. **Väg 1 eller Väg 2 (arkitektur):** Ska MOD-Agent i första versionen vara ett fristående CLI/program som läser och skriver `.mod`-filer bredvid `ProTracker 2.app` (Väg 1), eller byggas in direkt i C-källkoden till `pt2-clone` (Väg 2)?
2. **Implementationsspråk:** Detta repo är uppsatt för Python (`pyproject.toml` med `uv`), men i anteckningarna nämndes även C++ eller Kotlin — ska vi köra Python i detta repo för Väg 1?
3. **Tävlingsregler kring befintlig kod:** Om Väg 2 väljs, hur ser Kiroweens regler ut för att forka `pt2-clone` (BSD 3-Clause) jämfört med att skriva ett helt eget verktyg som bara läser/skriver `.mod`-filer?
4. **Icke-funktionella krav:** Behöver input från dig — se `/req-non-functional`.
