//! CLI entry point.
//!
//! Usage:
//!   mod-parser-rs <file.mod>              – print song summary
//!   mod-parser-rs <file.mod> --pattern N  – also print pattern N

use mod_parser_rs::{inspector, load, ModError};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mod-parser-rs <file.mod> [--pattern N]");
        std::process::exit(1);
    }

    let path = &args[1];
    let pattern_index: Option<usize> = args
        .windows(2)
        .find(|w| w[0] == "--pattern")
        .and_then(|w| w[1].parse().ok());

    match load(path) {
        Ok(m) => {
            print!("{}", inspector::format_summary(&m));
            if let Some(idx) = pattern_index {
                println!();
                match inspector::format_pattern(&m, idx) {
                    Ok(text) => print!("{text}"),
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(ModError::Parse { offset, message }) => {
            eprintln!("Parse error at byte {offset}: {message}");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
