//! 4-channel ProTracker .mod parser and serialiser.
//!
//! # Quick start
//! ```no_run
//! use mod_parser_rs::{load, save};
//! let mod_file = load("data/mods/turrican 2.mod").unwrap();
//! save(&mod_file, "/tmp/out.mod").unwrap();
//! ```

pub mod inspector;
pub mod model;

use model::*;
use std::io;
use std::path::Path;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ModError {
    Io(io::Error),
    Parse { offset: usize, message: String },
}

impl std::fmt::Display for ModError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModError::Io(e) => write!(f, "IO error: {e}"),
            ModError::Parse { offset, message } => {
                write!(f, "Parse error at byte {offset}: {message}")
            }
        }
    }
}

impl From<io::Error> for ModError {
    fn from(e: io::Error) -> Self {
        ModError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, ModError>;

// ---------------------------------------------------------------------------
// load
// ---------------------------------------------------------------------------

/// Parse a `.mod` file from disk and return a [`ModFile`].
pub fn load<P: AsRef<Path>>(path: P) -> Result<ModFile> {
    let data = std::fs::read(path)?;
    parse(&data)
}

/// Parse raw bytes into a [`ModFile`].
pub fn parse(data: &[u8]) -> Result<ModFile> {
    // Minimum size check
    if data.len() < HEADER_SIZE {
        return Err(ModError::Parse {
            offset: 0,
            message: format!(
                "File is too short: {} bytes; a valid .mod header requires {HEADER_SIZE} bytes.",
                data.len()
            ),
        });
    }

    // Format tag (bytes 1080..1083)
    let tag: [u8; 4] = data[1080..1084].try_into().unwrap();
    if !SUPPORTED_TAGS.contains(&&tag) {
        return Err(ModError::Parse {
            offset: 1080,
            message: format!(
                "Unsupported format tag {:?}. Supported: M.K., M!K!, FLT4, 4CHN.",
                std::str::from_utf8(&tag).unwrap_or("????")
            ),
        });
    }

    // Song title (bytes 0..19)
    let title: String = data[..20]
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();

    // 31 sample headers (bytes 20..949)
    let mut samples = Vec::with_capacity(MAX_SAMPLES);
    for i in 0..MAX_SAMPLES {
        let off = 20 + i * 30;
        samples.push(SampleHeader::from_bytes(&data[off..off + 30]));
    }

    // Song length (byte 950)
    let song_length = data[950];
    if !(1..=128).contains(&song_length) {
        return Err(ModError::Parse {
            offset: 950,
            message: format!(
                "Invalid song length {song_length}; expected 1..128."
            ),
        });
    }

    let restart_byte = data[951];

    // Pattern order table (bytes 952..1079)
    let order_table: [u8; ORDER_TABLE_SIZE] = data[952..1080].try_into().unwrap();

    let format_tag = tag;

    // Number of patterns
    let n_patterns = (*order_table.iter().max().unwrap() as usize) + 1;
    let required = HEADER_SIZE + n_patterns * BYTES_PER_PATTERN;
    if data.len() < required {
        return Err(ModError::Parse {
            offset: HEADER_SIZE,
            message: format!(
                "File is truncated: need {required} bytes for {n_patterns} patterns but only {} bytes available.",
                data.len()
            ),
        });
    }

    // Patterns
    let mut patterns = Vec::with_capacity(n_patterns);
    for i in 0..n_patterns {
        let start = HEADER_SIZE + i * BYTES_PER_PATTERN;
        patterns.push(Pattern::from_bytes(&data[start..start + BYTES_PER_PATTERN]));
    }

    // Sample PCM data
    let mut sample_offset = HEADER_SIZE + n_patterns * BYTES_PER_PATTERN;
    let mut sample_data = Vec::with_capacity(MAX_SAMPLES);
    for (i, sh) in samples.iter().enumerate() {
        let byte_len = sh.byte_length();
        if data.len() < sample_offset + byte_len {
            return Err(ModError::Parse {
                offset: sample_offset,
                message: format!(
                    "Truncated reading sample {} PCM: need {byte_len} bytes but only {} remain.",
                    i + 1,
                    data.len() - sample_offset
                ),
            });
        }
        sample_data.push(data[sample_offset..sample_offset + byte_len].to_vec());
        sample_offset += byte_len;
    }

    Ok(ModFile {
        title,
        samples,
        song_length,
        restart_byte,
        order_table,
        format_tag,
        patterns,
        sample_data,
    })
}

// ---------------------------------------------------------------------------
// save / serialise
// ---------------------------------------------------------------------------

/// Write a [`ModFile`] to disk.
pub fn save<P: AsRef<Path>>(mod_file: &ModFile, path: P) -> Result<()> {
    std::fs::write(path, serialise(mod_file))?;
    Ok(())
}

/// Serialise a [`ModFile`] to bytes.
pub fn serialise(m: &ModFile) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();

    // Song title (20 bytes)
    let tb = m.title.as_bytes();
    let tlen = tb.len().min(20);
    out.extend_from_slice(&tb[..tlen]);
    out.resize(20, 0);

    // 31 sample headers
    for i in 0..MAX_SAMPLES {
        out.extend_from_slice(&m.samples[i].to_bytes());
    }

    // Song length, restart byte
    out.push(m.song_length);
    out.push(m.restart_byte);

    // Order table (128 bytes)
    out.extend_from_slice(&m.order_table);

    // Format tag
    out.extend_from_slice(&m.format_tag);

    // Patterns
    for pat in &m.patterns {
        out.extend(pat.to_bytes());
    }

    // Sample PCM data — Req 3.3: zero first 2 bytes of non-looped samples
    for i in 0..MAX_SAMPLES {
        let pcm = if i < m.sample_data.len() { m.sample_data[i].as_slice() } else { &[] };
        if !pcm.is_empty() && !m.samples[i].has_loop() && pcm.len() >= 2 {
            out.push(0x00);
            out.push(0x00);
            out.extend_from_slice(&pcm[2..]);
        } else {
            out.extend_from_slice(pcm);
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    // -----------------------------------------------------------------------
    // helpers
    // -----------------------------------------------------------------------

    fn build_minimal_mod(
        title: &str,
        n_patterns: usize,
        tag: &[u8; 4],
        song_length: u8,
    ) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();

        // Song title
        let tb = title.as_bytes();
        let tlen = tb.len().min(20);
        out.extend_from_slice(&tb[..tlen]);
        out.resize(20, 0);

        // 31 sample headers — all empty
        for _ in 0..MAX_SAMPLES {
            out.extend_from_slice(&[0u8; 22]); // name
            out.extend_from_slice(&0u16.to_be_bytes()); // length words
            out.push(0); // finetune
            out.push(0); // volume
            out.extend_from_slice(&0u16.to_be_bytes()); // repeat offset
            out.extend_from_slice(&1u16.to_be_bytes()); // repeat length (1 = no loop)
        }

        // Song length, restart byte
        out.push(song_length);
        out.push(0x7F);

        // Order table
        let mut order = vec![0u8; ORDER_TABLE_SIZE];
        for i in 0..song_length as usize {
            order[i] = (i % n_patterns) as u8;
        }
        out.extend_from_slice(&order);

        // Format tag
        out.extend_from_slice(tag);

        // Pattern data (all zeros)
        out.extend(vec![0u8; n_patterns * BYTES_PER_PATTERN]);

        out
    }

    #[allow(dead_code)]
    fn byte_eq(a: &[u8], b: &[u8]) -> bool {
        a == b
    }
    #[allow(dead_code)]

    // -----------------------------------------------------------------------
    // AC 1.1 — parser reads all fields
    // -----------------------------------------------------------------------

    #[test]
    fn test_reads_title() {
        let raw = build_minimal_mod("Haunted Tune", 1, b"M.K.", 1);
        let m = parse(&raw).unwrap();
        assert_eq!(m.title, "Haunted Tune");
    }

    #[test]
    fn test_reads_31_sample_headers() {
        let raw = build_minimal_mod("Test", 1, b"M.K.", 1);
        let m = parse(&raw).unwrap();
        assert_eq!(m.samples.len(), MAX_SAMPLES);
    }

    #[test]
    fn test_reads_song_length() {
        let raw = build_minimal_mod("Test", 3, b"M.K.", 3);
        let m = parse(&raw).unwrap();
        assert_eq!(m.song_length, 3);
    }

    #[test]
    fn test_reads_order_table() {
        let raw = build_minimal_mod("Test", 1, b"M.K.", 1);
        let m = parse(&raw).unwrap();
        assert_eq!(m.order_table.len(), ORDER_TABLE_SIZE);
        assert_eq!(m.order_table[0], 0);
    }

    #[test]
    fn test_reads_patterns() {
        let raw = build_minimal_mod("Test", 2, b"M.K.", 2);
        let m = parse(&raw).unwrap();
        assert_eq!(m.patterns.len(), 2);
    }

    #[test]
    fn test_reads_pattern_rows_and_channels() {
        let raw = build_minimal_mod("Test", 1, b"M.K.", 1);
        let m = parse(&raw).unwrap();
        assert_eq!(m.patterns[0].rows.len(), PATTERN_ROWS);
        assert_eq!(m.patterns[0].rows[0].len(), CHANNELS);
    }

    #[test]
    fn test_reads_sample_pcm_empty_for_zero_length() {
        let raw = build_minimal_mod("Test", 1, b"M.K.", 1);
        let m = parse(&raw).unwrap();
        assert_eq!(m.sample_data.len(), MAX_SAMPLES);
        assert!(m.sample_data[0].is_empty());
    }

    #[test]
    fn test_accepts_all_supported_tags() {
        for tag in SUPPORTED_TAGS {
            let raw = build_minimal_mod("Test", 1, tag, 1);
            let m = parse(&raw).unwrap();
            assert_eq!(&m.format_tag, tag);
        }
    }

    // -----------------------------------------------------------------------
    // AC 1.2 — byte-identical round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn test_synthetic_roundtrip_is_byte_identical() {
        let raw = build_minimal_mod("Round Trip", 2, b"M.K.", 2);
        let m = parse(&raw).unwrap();
        let result = serialise(&m);
        assert_eq!(
            result, raw,
            "Round-trip produced different bytes (lengths: {} vs {})",
            result.len(), raw.len()
        );
    }

    #[test]
    fn test_save_load_matches_title_and_order() {
        let raw = build_minimal_mod("Save Load", 1, b"M.K.", 1);
        let m1 = parse(&raw).unwrap();
        let bytes = serialise(&m1);
        let m2 = parse(&bytes).unwrap();
        assert_eq!(m1.title, m2.title);
        assert_eq!(m1.song_length, m2.song_length);
        assert_eq!(m1.order_table, m2.order_table);
    }

    // -----------------------------------------------------------------------
    // AC 1.3 — validation errors with byte offsets
    // -----------------------------------------------------------------------

    #[test]
    fn test_rejects_too_short_file() {
        let err = parse(&[0u8; 100]).unwrap_err();
        match err {
            ModError::Parse { message, .. } => {
                assert!(message.contains("too short"), "got: {message}");
            }
            other => panic!("Expected Parse error, got {other:?}"),
        }
    }

    #[test]
    fn test_rejects_bad_format_tag() {
        let mut raw = build_minimal_mod("Test", 1, b"M.K.", 1);
        raw[1080..1084].copy_from_slice(b"XXXX");
        let err = parse(&raw).unwrap_err();
        match err {
            ModError::Parse { offset, message } => {
                assert_eq!(offset, 1080);
                assert!(message.contains("Unsupported"), "got: {message}");
            }
            other => panic!("Expected Parse error, got {other:?}"),
        }
    }

    #[test]
    fn test_rejects_truncated_pattern_data() {
        let raw = build_minimal_mod("Test", 2, b"M.K.", 2);
        // Keep only first pattern worth of data
        let truncated = &raw[..HEADER_SIZE + BYTES_PER_PATTERN];
        let err = parse(truncated).unwrap_err();
        match err {
            ModError::Parse { message, .. } => {
                assert!(message.contains("truncated") || message.contains("truncated"),
                    "got: {message}");
            }
            other => panic!("Expected Parse error, got {other:?}"),
        }
    }

    #[test]
    fn test_error_message_includes_byte_offset_1080_for_bad_tag() {
        let mut raw = build_minimal_mod("Test", 1, b"M.K.", 1);
        raw[1080..1084].copy_from_slice(b"????");
        if let ModError::Parse { offset, .. } = parse(&raw).unwrap_err() {
            assert_eq!(offset, 1080);
        }
    }

    // -----------------------------------------------------------------------
    // NoteCell encoding / decoding
    // -----------------------------------------------------------------------

    #[test]
    fn test_note_cell_roundtrip() {
        let cell = NoteCell { sample: 0x1A, period: 428, effect_cmd: 0xC, effect_arg: 0x40 };
        assert_eq!(NoteCell::from_bytes(&cell.to_bytes()), cell);
    }

    #[test]
    fn test_empty_note_cell_is_four_zero_bytes() {
        assert_eq!(NoteCell::default().to_bytes(), [0, 0, 0, 0]);
    }

    #[test]
    fn test_note_name_known_period() {
        let cell = NoteCell { sample: 1, period: 428, effect_cmd: 0, effect_arg: 0 };
        assert!(cell.note_name().starts_with("C-2"), "got: {}", cell.note_name());
    }

    #[test]
    fn test_note_name_zero_period_is_dashes() {
        let cell = NoteCell::default();
        assert!(cell.note_name().starts_with("---"), "got: {}", cell.note_name());
    }

    // -----------------------------------------------------------------------
    // SampleHeader encoding / decoding
    // -----------------------------------------------------------------------

    #[test]
    fn test_sample_header_roundtrip() {
        let sh = SampleHeader {
            name: "Kick Drum".into(),
            length_words: 256,
            finetune: 2,
            volume: 50,
            repeat_offset_words: 0,
            repeat_length_words: 1,
        };
        assert_eq!(SampleHeader::from_bytes(&sh.to_bytes()), sh);
    }

    #[test]
    fn test_has_loop_false_when_repeat_length_1() {
        let sh = SampleHeader {
            name: String::new(), length_words: 0, finetune: 0,
            volume: 0, repeat_offset_words: 0, repeat_length_words: 1,
        };
        assert!(!sh.has_loop());
    }

    #[test]
    fn test_has_loop_true_when_repeat_length_gt_1() {
        let sh = SampleHeader {
            name: String::new(), length_words: 0, finetune: 0,
            volume: 0, repeat_offset_words: 0, repeat_length_words: 4,
        };
        assert!(sh.has_loop());
    }

    #[test]
    fn test_byte_length_is_words_times_two() {
        let sh = SampleHeader {
            name: String::new(), length_words: 100, finetune: 0,
            volume: 0, repeat_offset_words: 0, repeat_length_words: 1,
        };
        assert_eq!(sh.byte_length(), 200);
    }

    // -----------------------------------------------------------------------
    // Real-file round-trip tests (run with: cargo test -- --include-ignored)
    // These are marked ignore so `cargo test` in CI doesn't require the files;
    // run them locally with: cargo test real_file -- --ignored
    // -----------------------------------------------------------------------

    fn real_mod_files() -> Vec<std::path::PathBuf> {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()   // src/
            .parent().unwrap()   // repo root
            .join("data/mods");
        let mut files: Vec<_> = std::fs::read_dir(&base)
            .unwrap_or_else(|_| panic!("data/mods not found at {}", base.display()))
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                matches!(
                    p.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()).as_deref(),
                    Some("mod")
                )
            })
            .collect();
        files.sort();
        files
    }

    #[test]
    #[ignore]
    fn test_real_file_roundtrips_are_byte_identical() {
        let files = real_mod_files();
        assert!(!files.is_empty(), "No .mod files found in data/mods/");
        for path in &files {
            let raw = std::fs::read(path).unwrap();
            let m = parse(&raw).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let result = serialise(&m);
            assert_eq!(
                result, raw,
                "Round-trip mismatch for {}",
                path.file_name().unwrap().to_string_lossy()
            );
        }
    }

    #[test]
    #[ignore]
    fn test_real_files_parse_without_error() {
        for path in real_mod_files() {
            load(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        }
    }
}
