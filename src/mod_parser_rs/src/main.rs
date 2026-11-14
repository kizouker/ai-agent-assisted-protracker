use mod_parser_rs::{load, ModError};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mod-parser-rs <file.mod>");
        std::process::exit(1);
    }
    match load(&args[1]) {
        Ok(m) => {
            println!("Title      : {:?}", m.title);
            println!("Tag        : {}", std::str::from_utf8(&m.format_tag).unwrap_or("????"));
            println!("Song length: {}", m.song_length);
            println!("Patterns   : {}", m.patterns.len());
            for (i, s) in m.samples.iter().enumerate() {
                if s.length_words > 0 {
                    let loop_info = if s.has_loop() {
                        format!("loop @{}w +{}w", s.repeat_offset_words, s.repeat_length_words)
                    } else {
                        "no loop".into()
                    };
                    println!("  Sample {:2}  {:22}  len={:5}B  vol={:2}  ft={}  {}",
                        i + 1, s.name, s.byte_length(), s.volume, s.finetune, loop_info);
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
