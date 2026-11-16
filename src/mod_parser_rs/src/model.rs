//! Data model for 4-channel ProTracker .mod files (31 samples, M.K. family).

pub const HEADER_SIZE: usize = 1084;
pub const PATTERN_ROWS: usize = 64;
pub const CHANNELS: usize = 4;
pub const BYTES_PER_CELL: usize = 4;
pub const BYTES_PER_PATTERN: usize = PATTERN_ROWS * CHANNELS * BYTES_PER_CELL; // 1024
pub const MAX_SAMPLES: usize = 31;
pub const ORDER_TABLE_SIZE: usize = 128;

pub const SUPPORTED_TAGS: [&[u8; 4]; 4] = [b"M.K.", b"M!K!", b"FLT4", b"4CHN"];

/// Amiga period table, finetune 0 — 36 notes C-1..B-3.
pub const PERIOD_TABLE: [(u16, &str); 36] = [
    (856, "C-1"), (808, "C#1"), (762, "D-1"), (720, "D#1"), (678, "E-1"), (640, "F-1"),
    (604, "F#1"), (570, "G-1"), (538, "G#1"), (508, "A-1"), (480, "A#1"), (453, "B-1"),
    (428, "C-2"), (404, "C#2"), (381, "D-2"), (360, "D#2"), (339, "E-2"), (320, "F-2"),
    (302, "F#2"), (285, "G-2"), (269, "G#2"), (254, "A-2"), (240, "A#2"), (226, "B-2"),
    (214, "C-3"), (202, "C#3"), (190, "D-3"), (180, "D#3"), (170, "E-3"), (160, "F-3"),
    (151, "F#3"), (143, "G-3"), (135, "G#3"), (127, "A-3"), (120, "A#3"), (113, "B-3"),
];

/// Return the note name for a period, or None if not in finetune-0 table.
pub fn period_to_note(period: u16) -> Option<&'static str> {
    PERIOD_TABLE.iter().find(|(p, _)| *p == period).map(|(_, n)| *n)
}

/// Return the period for a note name (e.g. "C-2"), or None if not in finetune-0 table.
/// Accepts both dash and hyphen: "C-2" and "C#2" are valid; octave suffix is 1..3.
pub fn note_to_period(note: &str) -> Option<u16> {
    PERIOD_TABLE.iter().find(|(_, n)| *n == note).map(|(p, _)| *p)
}

// ---------------------------------------------------------------------------
// SampleHeader
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleHeader {
    pub name: String,           // up to 22 chars
    pub length_words: u16,      // byte_length = words * 2
    pub finetune: u8,           // lower 4 bits; 8..15 = -8..-1
    pub volume: u8,             // 0..64
    pub repeat_offset_words: u16,
    pub repeat_length_words: u16, // 1 = no loop, >1 = loop
}

impl SampleHeader {
    pub fn byte_length(&self) -> usize {
        self.length_words as usize * 2
    }

    pub fn has_loop(&self) -> bool {
        self.repeat_length_words > 1
    }

    /// Parse 30 bytes at `data[offset..]`.
    pub fn from_bytes(data: &[u8]) -> Self {
        assert!(data.len() >= 30, "SampleHeader needs 30 bytes");
        let name = data[..22]
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect();
        let length_words = u16::from_be_bytes([data[22], data[23]]);
        let finetune = data[24] & 0x0F;
        let volume = data[25];
        let repeat_offset_words = u16::from_be_bytes([data[26], data[27]]);
        let repeat_length_words = u16::from_be_bytes([data[28], data[29]]);
        Self { name, length_words, finetune, volume, repeat_offset_words, repeat_length_words }
    }

    /// Serialise to exactly 30 bytes.
    pub fn to_bytes(&self) -> [u8; 30] {
        let mut out = [0u8; 30];
        let name_b = self.name.as_bytes();
        let len = name_b.len().min(22);
        out[..len].copy_from_slice(&name_b[..len]);
        out[22..24].copy_from_slice(&self.length_words.to_be_bytes());
        out[24] = self.finetune & 0x0F;
        out[25] = self.volume;
        out[26..28].copy_from_slice(&self.repeat_offset_words.to_be_bytes());
        out[28..30].copy_from_slice(&self.repeat_length_words.to_be_bytes());
        out
    }
}

// ---------------------------------------------------------------------------
// NoteCell
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NoteCell {
    pub sample: u8,      // 0..31
    pub period: u16,     // 0..4095 (12-bit Amiga period)
    pub effect_cmd: u8,  // 0..15
    pub effect_arg: u8,  // 0..255
}

impl NoteCell {
    pub fn from_bytes(b: &[u8; 4]) -> Self {
        let sample = (b[0] & 0xF0) | ((b[2] >> 4) & 0x0F);
        let period = (((b[0] & 0x0F) as u16) << 8) | b[1] as u16;
        let effect_cmd = b[2] & 0x0F;
        let effect_arg = b[3];
        Self { sample, period, effect_cmd, effect_arg }
    }

    pub fn to_bytes(self) -> [u8; 4] {
        let b0 = (self.sample & 0xF0) | ((self.period >> 8) as u8 & 0x0F);
        let b1 = (self.period & 0xFF) as u8;
        let b2 = ((self.sample & 0x0F) << 4) | (self.effect_cmd & 0x0F);
        let b3 = self.effect_arg;
        [b0, b1, b2, b3]
    }

    /// ProTracker text notation: `C-2 01 C40` or `--- 00 000`.
    pub fn note_name(&self) -> String {
        let note = if self.period == 0 {
            "---".to_string()
        } else {
            match period_to_note(self.period) {
                Some(n) => n.to_string(),
                None => format!("?{:03}", self.period),
            }
        };
        format!("{} {:02X} {:X}{:02X}", note, self.sample, self.effect_cmd, self.effect_arg)
    }
}

// ---------------------------------------------------------------------------
// Pattern
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    /// rows[row][channel]
    pub rows: Vec<[NoteCell; CHANNELS]>,
}

impl Pattern {
    pub fn empty() -> Self {
        Self { rows: vec![[NoteCell::default(); CHANNELS]; PATTERN_ROWS] }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        assert_eq!(data.len(), BYTES_PER_PATTERN);
        let mut rows = Vec::with_capacity(PATTERN_ROWS);
        for r in 0..PATTERN_ROWS {
            let mut row = [NoteCell::default(); CHANNELS];
            for c in 0..CHANNELS {
                let off = (r * CHANNELS + c) * BYTES_PER_CELL;
                row[c] = NoteCell::from_bytes(data[off..off + 4].try_into().unwrap());
            }
            rows.push(row);
        }
        Self { rows }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(BYTES_PER_PATTERN);
        for row in &self.rows {
            for cell in row {
                out.extend_from_slice(&cell.to_bytes());
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// ModFile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ModFile {
    pub title: String,
    pub samples: Vec<SampleHeader>, // always MAX_SAMPLES entries
    pub song_length: u8,            // 1..128
    pub restart_byte: u8,
    pub order_table: [u8; ORDER_TABLE_SIZE],
    pub format_tag: [u8; 4],
    pub patterns: Vec<Pattern>,
    pub sample_data: Vec<Vec<u8>>, // always MAX_SAMPLES entries
}

impl ModFile {
    pub fn pattern_count(&self) -> usize {
        (*self.order_table.iter().max().unwrap_or(&0) as usize) + 1
    }
}
