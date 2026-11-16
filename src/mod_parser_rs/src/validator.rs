//! Ticket 3 – Amiga/ProTracker constraint validation.
//!
//! AC 3.1 – Convert note names (C-1..B-3) to exact 12-bit period values.
//! AC 3.2 – Reject out-of-range sample index, volume, row index.
//! AC 3.3 – Enforce that first 2 bytes of non-looped sample PCM are 0x00.

use crate::model::{note_to_period, SampleHeader, PATTERN_ROWS};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    UnknownNote(String),
    SampleIndexOutOfRange(u8),
    VolumeOutOfRange(u8),
    RowOutOfRange(usize),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::UnknownNote(n) => {
                write!(f, "Unknown note {n:?}: valid range is C-1..B-3 (finetune 0)")
            }
            ValidationError::SampleIndexOutOfRange(s) => {
                write!(f, "Sample index {s} out of range: valid range is 0..31")
            }
            ValidationError::VolumeOutOfRange(v) => {
                write!(f, "Volume {v} out of range: valid range is 0..64")
            }
            ValidationError::RowOutOfRange(r) => {
                write!(f, "Row {r} out of range: valid range is 0..63")
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, ValidationError>;

// ---------------------------------------------------------------------------
// AC 3.1 – note name → period (12-bit Amiga value)
// ---------------------------------------------------------------------------

/// Convert a note name to its Amiga period value (finetune 0).
///
/// Accepts the format used by ProTracker: "C-1", "C#2", "B-3", etc.
/// Returns `Err(UnknownNote)` if the name is not in the period table.
pub fn note_name_to_period(note: &str) -> Result<u16> {
    note_to_period(note).ok_or_else(|| ValidationError::UnknownNote(note.to_string()))
}

// ---------------------------------------------------------------------------
// AC 3.2 – range checks
// ---------------------------------------------------------------------------

/// Validate a sample slot index (0 = "no sample", 1..31 = slot).
/// Returns `Err(SampleIndexOutOfRange)` if > 31.
pub fn validate_sample_index(index: u8) -> Result<()> {
    if index > 31 {
        Err(ValidationError::SampleIndexOutOfRange(index))
    } else {
        Ok(())
    }
}

/// Validate a channel volume (0..64).
/// Returns `Err(VolumeOutOfRange)` if > 64.
pub fn validate_volume(volume: u8) -> Result<()> {
    if volume > 64 {
        Err(ValidationError::VolumeOutOfRange(volume))
    } else {
        Ok(())
    }
}

/// Validate a pattern row index (0..63).
/// Returns `Err(RowOutOfRange)` if >= 64.
pub fn validate_row(row: usize) -> Result<()> {
    if row >= PATTERN_ROWS {
        Err(ValidationError::RowOutOfRange(row))
    } else {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// AC 3.3 – non-looped sample PCM silence guard
// ---------------------------------------------------------------------------

/// Ensure the first 2 bytes of a non-looped sample's PCM data are 0x00.
///
/// If `sample` has no loop (`repeat_length_words <= 1`) and `pcm` has at
/// least 2 bytes, the returned slice (or owned copy) has those bytes zeroed.
/// If the sample has a loop or PCM is shorter than 2 bytes, returns `pcm`
/// unchanged.
///
/// This matches the Amiga Paula behaviour: after a one-shot sample finishes
/// the chip re-reads the first word as a silent loop, so non-zero bytes
/// cause audible noise.
pub fn silence_non_looped_header(header: &SampleHeader, pcm: &[u8]) -> Vec<u8> {
    if !header.has_loop() && pcm.len() >= 2 {
        let mut out = pcm.to_vec();
        out[0] = 0x00;
        out[1] = 0x00;
        out
    } else {
        pcm.to_vec()
    }
}

/// Return true if a non-looped sample's PCM data already satisfies AC 3.3.
pub fn is_silence_guarded(header: &SampleHeader, pcm: &[u8]) -> bool {
    if !header.has_loop() && pcm.len() >= 2 {
        pcm[0] == 0x00 && pcm[1] == 0x00
    } else {
        true // looped or empty: no constraint
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{note_to_period, SampleHeader, PERIOD_TABLE};

    // -----------------------------------------------------------------------
    // AC 3.1 – note name → period
    // -----------------------------------------------------------------------

    #[test]
    fn test_all_36_notes_resolve() {
        for (period, name) in &PERIOD_TABLE {
            let result = note_name_to_period(name);
            assert_eq!(result, Ok(*period), "note {name} failed");
        }
    }

    #[test]
    fn test_c1_maps_to_856() {
        assert_eq!(note_name_to_period("C-1"), Ok(856));
    }

    #[test]
    fn test_b3_maps_to_113() {
        assert_eq!(note_name_to_period("B-3"), Ok(113));
    }

    #[test]
    fn test_c2_maps_to_428() {
        assert_eq!(note_name_to_period("C-2"), Ok(428));
    }

    #[test]
    fn test_unknown_note_returns_error() {
        let err = note_name_to_period("X-9").unwrap_err();
        assert_eq!(err, ValidationError::UnknownNote("X-9".into()));
    }

    #[test]
    fn test_out_of_range_octave_returns_error() {
        assert!(note_name_to_period("C-4").is_err());
        assert!(note_name_to_period("C-0").is_err());
    }

    #[test]
    fn test_period_round_trips_through_name() {
        // Every period in the table should round-trip via note_to_period
        for (period, name) in &PERIOD_TABLE {
            assert_eq!(note_to_period(name), Some(*period));
        }
    }

    // -----------------------------------------------------------------------
    // AC 3.2 – range checks
    // -----------------------------------------------------------------------

    #[test]
    fn test_sample_index_0_valid() {
        assert!(validate_sample_index(0).is_ok());
    }

    #[test]
    fn test_sample_index_31_valid() {
        assert!(validate_sample_index(31).is_ok());
    }

    #[test]
    fn test_sample_index_32_invalid() {
        assert_eq!(
            validate_sample_index(32),
            Err(ValidationError::SampleIndexOutOfRange(32))
        );
    }

    #[test]
    fn test_volume_0_valid() {
        assert!(validate_volume(0).is_ok());
    }

    #[test]
    fn test_volume_64_valid() {
        assert!(validate_volume(64).is_ok());
    }

    #[test]
    fn test_volume_65_invalid() {
        assert_eq!(
            validate_volume(65),
            Err(ValidationError::VolumeOutOfRange(65))
        );
    }

    #[test]
    fn test_row_0_valid() {
        assert!(validate_row(0).is_ok());
    }

    #[test]
    fn test_row_63_valid() {
        assert!(validate_row(63).is_ok());
    }

    #[test]
    fn test_row_64_invalid() {
        assert_eq!(
            validate_row(64),
            Err(ValidationError::RowOutOfRange(64))
        );
    }

    #[test]
    fn test_error_messages_mention_constraint() {
        assert!(ValidationError::SampleIndexOutOfRange(32).to_string().contains("0..31"));
        assert!(ValidationError::VolumeOutOfRange(65).to_string().contains("0..64"));
        assert!(ValidationError::RowOutOfRange(64).to_string().contains("0..63"));
        assert!(ValidationError::UnknownNote("Z".into()).to_string().contains("C-1..B-3"));
    }

    // -----------------------------------------------------------------------
    // AC 3.3 – silence guard
    // -----------------------------------------------------------------------

    fn no_loop_header() -> SampleHeader {
        SampleHeader {
            name: String::new(),
            length_words: 256,
            finetune: 0,
            volume: 64,
            repeat_offset_words: 0,
            repeat_length_words: 1, // no loop
        }
    }

    fn loop_header() -> SampleHeader {
        SampleHeader {
            name: String::new(),
            length_words: 128,
            finetune: 0,
            volume: 48,
            repeat_offset_words: 0,
            repeat_length_words: 128, // has loop
        }
    }

    #[test]
    fn test_non_looped_pcm_first_two_bytes_zeroed() {
        let pcm = vec![0xAB, 0xCD, 0x01, 0x02];
        let result = silence_non_looped_header(&no_loop_header(), &pcm);
        assert_eq!(&result[..2], &[0x00, 0x00]);
        assert_eq!(&result[2..], &[0x01, 0x02]); // rest unchanged
    }

    #[test]
    fn test_looped_pcm_unchanged() {
        let pcm = vec![0xAB, 0xCD, 0x01, 0x02];
        let result = silence_non_looped_header(&loop_header(), &pcm);
        assert_eq!(result, pcm);
    }

    #[test]
    fn test_pcm_shorter_than_2_bytes_unchanged() {
        let pcm = vec![0xAB];
        let result = silence_non_looped_header(&no_loop_header(), &pcm);
        assert_eq!(result, pcm);
    }

    #[test]
    fn test_is_silence_guarded_true_when_zeroed() {
        let pcm = vec![0x00, 0x00, 0x01];
        assert!(is_silence_guarded(&no_loop_header(), &pcm));
    }

    #[test]
    fn test_is_silence_guarded_false_when_not_zeroed() {
        let pcm = vec![0xAB, 0x00, 0x01];
        assert!(!is_silence_guarded(&no_loop_header(), &pcm));
    }

    #[test]
    fn test_is_silence_guarded_true_for_loop() {
        // Looped samples have no constraint — always "guarded"
        let pcm = vec![0xAB, 0xCD];
        assert!(is_silence_guarded(&loop_header(), &pcm));
    }

    #[test]
    fn test_is_silence_guarded_true_for_empty_pcm() {
        assert!(is_silence_guarded(&no_loop_header(), &[]));
    }
}
