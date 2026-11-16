//! Ticket 2 – human-readable text view of a ModFile.
//!
//! Produces output suitable for the terminal AND as AI prompt context.

use crate::model::{ModFile, NoteCell, CHANNELS, PATTERN_ROWS, PERIOD_TABLE};

// ---------------------------------------------------------------------------
// AC 2.1 – song summary
// ---------------------------------------------------------------------------

/// Return a multi-line summary of the song: title, lengths, patterns,
/// order table, and all non-empty samples with name/length/vol/finetune/loop.
pub fn format_summary(m: &ModFile) -> String {
    let mut out = String::new();

    out.push_str(&format!("Title      : {}\n", m.title));
    out.push_str(&format!(
        "Tag        : {}\n",
        std::str::from_utf8(&m.format_tag).unwrap_or("????")
    ));
    out.push_str(&format!("Song length: {} positions\n", m.song_length));
    out.push_str(&format!("Patterns   : {}\n", m.patterns.len()));

    // Order table — only the active positions (0..song_length)
    let order_str: Vec<String> = m.order_table[..m.song_length as usize]
        .iter()
        .map(|n| n.to_string())
        .collect();
    out.push_str(&format!("Order      : [{}]\n", order_str.join(", ")));

    out.push_str("\nSamples:\n");
    let mut has_any = false;
    for (i, s) in m.samples.iter().enumerate() {
        if s.length_words == 0 {
            continue;
        }
        has_any = true;
        let loop_str = if s.has_loop() {
            format!(
                "loop @{}w +{}w",
                s.repeat_offset_words, s.repeat_length_words
            )
        } else {
            "no loop".to_string()
        };
        // Finetune: display as signed value (-8..+7)
        let ft_signed = if s.finetune > 7 {
            s.finetune as i8 - 16
        } else {
            s.finetune as i8
        };
        out.push_str(&format!(
            "  {:2}  {:<22}  len={:5}B  vol={:2}  ft={:+}  {}\n",
            i + 1,
            s.name,
            s.byte_length(),
            s.volume,
            ft_signed,
            loop_str
        ));
    }
    if !has_any {
        out.push_str("  (no samples)\n");
    }

    out
}

// ---------------------------------------------------------------------------
// AC 2.2 + 2.3 – pattern text view
// ---------------------------------------------------------------------------

/// Return a 64-row × 4-channel grid for `pattern_index` as ProTracker text.
///
/// Each cell is `Note Oct  SN  Eff` e.g. `C-2 01 C40` or `--- 00 000`.
/// For periods not in the finetune-0 table (AC 2.3) the note shows
/// `?NNN` where NNN is the raw period, e.g. `?500`.
pub fn format_pattern(m: &ModFile, pattern_index: usize) -> Result<String, String> {
    if pattern_index >= m.patterns.len() {
        return Err(format!(
            "Pattern index {} out of range (0..{})",
            pattern_index,
            m.patterns.len()
        ));
    }
    let pat = &m.patterns[pattern_index];
    let mut out = String::new();

    out.push_str(&format!(
        "Pattern {:2}  ({} rows × {} channels)\n",
        pattern_index, PATTERN_ROWS, CHANNELS
    ));
    out.push_str("  Row  Ch1          Ch2          Ch3          Ch4\n");
    out.push_str("  ---  -----------  -----------  -----------  -----------\n");

    for (r, row) in pat.rows.iter().enumerate() {
        let cells: Vec<String> = row.iter().map(|c| format_cell(c)).collect();
        out.push_str(&format!("  {:3}  {}\n", r, cells.join("  ")));
    }

    Ok(out)
}

/// Format one NoteCell as an 11-character ProTracker string.
///
/// Format: `NNN SS EEE`  (note, sample hex, effect+arg hex)
/// Unknown period: `?PPP SS EEE` (AC 2.3)
fn format_cell(c: &NoteCell) -> String {
    let note = if c.period == 0 {
        "---".to_string()
    } else {
        // Look up exact period in table
        match PERIOD_TABLE.iter().find(|(p, _)| *p == c.period) {
            Some((_, name)) => name.to_string(),
            // AC 2.3: show nearest note name + raw period
            None => {
                let (_, nearest) = PERIOD_TABLE
                    .iter()
                    .min_by_key(|(p, _)| (*p as i32 - c.period as i32).unsigned_abs())
                    .unwrap();
                format!("~{}", nearest) // e.g. ~C-2 (non-exact)
            }
        }
    };
    format!("{} {:02X} {:X}{:02X}", note, c.sample, c.effect_cmd, c.effect_arg)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        ModFile, NoteCell, Pattern, SampleHeader, MAX_SAMPLES, ORDER_TABLE_SIZE,
        PATTERN_ROWS,
    };

    fn make_mod() -> ModFile {
        let mut samples: Vec<SampleHeader> = (0..MAX_SAMPLES)
            .map(|_| SampleHeader {
                name: String::new(),
                length_words: 0,
                finetune: 0,
                volume: 0,
                repeat_offset_words: 0,
                repeat_length_words: 1,
            })
            .collect();
        samples[0] = SampleHeader {
            name: "Kick Drum".into(),
            length_words: 256,
            finetune: 0,
            volume: 64,
            repeat_offset_words: 0,
            repeat_length_words: 1,
        };
        samples[1] = SampleHeader {
            name: "Bass Synth".into(),
            length_words: 128,
            finetune: 2,
            volume: 48,
            repeat_offset_words: 0,
            repeat_length_words: 128,
        };
        let mut order_table = [0u8; ORDER_TABLE_SIZE];
        order_table[0] = 0;
        ModFile {
            title: "Test Song".into(),
            samples,
            song_length: 1,
            restart_byte: 0x7F,
            order_table,
            format_tag: *b"M.K.",
            patterns: vec![Pattern::empty()],
            sample_data: vec![vec![]; MAX_SAMPLES],
        }
    }

    // -----------------------------------------------------------------------
    // AC 2.1 – summary
    // -----------------------------------------------------------------------

    #[test]
    fn test_summary_contains_title() {
        let m = make_mod();
        let s = format_summary(&m);
        assert!(s.contains("Test Song"), "got:\n{s}");
    }

    #[test]
    fn test_summary_contains_song_length() {
        let m = make_mod();
        let s = format_summary(&m);
        assert!(s.contains("1 positions") || s.contains("Song length: 1"), "got:\n{s}");
    }

    #[test]
    fn test_summary_contains_pattern_count() {
        let m = make_mod();
        let s = format_summary(&m);
        assert!(s.contains("Patterns   : 1"), "got:\n{s}");
    }

    #[test]
    fn test_summary_lists_non_empty_samples() {
        let m = make_mod();
        let s = format_summary(&m);
        assert!(s.contains("Kick Drum"), "got:\n{s}");
        assert!(s.contains("Bass Synth"), "got:\n{s}");
    }

    #[test]
    fn test_summary_shows_sample_byte_length() {
        let m = make_mod();
        let s = format_summary(&m);
        assert!(s.contains("512B"), "kick 256 words = 512 bytes; got:\n{s}");
    }

    #[test]
    fn test_summary_shows_volume() {
        let m = make_mod();
        let s = format_summary(&m);
        assert!(s.contains("vol=64"), "got:\n{s}");
    }

    #[test]
    fn test_summary_shows_loop_status_no_loop() {
        let m = make_mod();
        let s = format_summary(&m);
        assert!(s.contains("no loop"), "got:\n{s}");
    }

    #[test]
    fn test_summary_shows_loop_status_has_loop() {
        let m = make_mod();
        let s = format_summary(&m);
        assert!(s.contains("loop @"), "got:\n{s}");
    }

    #[test]
    fn test_summary_shows_order_table() {
        let m = make_mod();
        let s = format_summary(&m);
        assert!(s.contains("Order"), "got:\n{s}");
        assert!(s.contains("[0]"), "got:\n{s}");
    }

    #[test]
    fn test_summary_skips_empty_samples() {
        let m = make_mod();
        let s = format_summary(&m);
        // Only 2 non-empty samples; should not list 31 entries
        let lines_with_len: Vec<_> = s.lines().filter(|l| l.contains("len=")).collect();
        assert_eq!(lines_with_len.len(), 2, "got:\n{s}");
    }

    #[test]
    fn test_summary_finetune_shown_as_signed() {
        let mut m = make_mod();
        m.samples[0].finetune = 10; // 10 -> -6 as signed 4-bit
        let s = format_summary(&m);
        assert!(s.contains("ft=-6"), "got:\n{s}");
    }

    // -----------------------------------------------------------------------
    // AC 2.2 – pattern text
    // -----------------------------------------------------------------------

    #[test]
    fn test_pattern_header_row_present() {
        let m = make_mod();
        let s = format_pattern(&m, 0).unwrap();
        assert!(s.contains("Row"), "got:\n{s}");
        assert!(s.contains("Ch1"), "got:\n{s}");
    }

    #[test]
    fn test_pattern_has_64_data_rows() {
        let m = make_mod();
        let s = format_pattern(&m, 0).unwrap();
        // Data rows start with spaces + row number
        let data_rows: Vec<_> = s
            .lines()
            .filter(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit()))
            .collect();
        assert_eq!(data_rows.len(), PATTERN_ROWS, "got:\n{s}");
    }

    #[test]
    fn test_empty_cell_prints_dashes() {
        let m = make_mod();
        let s = format_pattern(&m, 0).unwrap();
        assert!(s.contains("--- 00 000"), "got:\n{s}");
    }

    #[test]
    fn test_known_period_prints_note_name() {
        let mut m = make_mod();
        // Set row 0 ch0 to C-2 (period 428), sample 1, no effect
        m.patterns[0].rows[0][0] = NoteCell { sample: 1, period: 428, effect_cmd: 0, effect_arg: 0 };
        let s = format_pattern(&m, 0).unwrap();
        assert!(s.contains("C-2 01 000"), "got:\n{s}");
    }

    #[test]
    fn test_effect_cmd_and_arg_printed() {
        let mut m = make_mod();
        m.patterns[0].rows[0][0] = NoteCell { sample: 1, period: 428, effect_cmd: 0xC, effect_arg: 0x40 };
        let s = format_pattern(&m, 0).unwrap();
        assert!(s.contains("C-2 01 C40"), "got:\n{s}");
    }

    #[test]
    fn test_out_of_range_pattern_returns_error() {
        let m = make_mod();
        assert!(format_pattern(&m, 99).is_err());
    }

    // -----------------------------------------------------------------------
    // AC 2.3 – non-exact period shows nearest note
    // -----------------------------------------------------------------------

    #[test]
    fn test_non_exact_period_shows_tilde_prefix() {
        let cell = NoteCell { sample: 0, period: 500, effect_cmd: 0, effect_arg: 0 };
        let formatted = format_cell(&cell);
        // 500 is between 508 (A-1) and 480 (A#1); nearest is 508
        assert!(formatted.starts_with("~"), "got: {formatted}");
    }

    #[test]
    fn test_non_exact_period_names_nearest_note() {
        let cell = NoteCell { sample: 0, period: 505, effect_cmd: 0, effect_arg: 0 };
        let formatted = format_cell(&cell);
        // 505 is 3 away from 508 (A-1) and 25 away from 480 (A#1)
        assert!(formatted.contains("A-1"), "got: {formatted}");
    }

    // -----------------------------------------------------------------------
    // Real-file smoke test (ignored, needs data/mods/)
    // -----------------------------------------------------------------------

    #[test]
    #[ignore]
    fn test_real_file_summary_and_pattern_smoke() {
        use crate::load;
        use std::path::Path;
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .parent().unwrap()
            .join("data/mods");
        for entry in std::fs::read_dir(&base).unwrap() {
            let path = entry.unwrap().path();
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            if ext != "mod" { continue; }
            let m = load(&path).unwrap();
            let summary = format_summary(&m);
            assert!(!summary.is_empty());
            // Print pattern 0 of every file — must not panic
            let _ = format_pattern(&m, 0).unwrap();
        }
    }
}
