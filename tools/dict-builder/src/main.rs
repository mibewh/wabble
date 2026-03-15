use std::path::PathBuf;
use std::{env, fs, process};

use wabble_dict::{FstDictionary, Gaddag};

fn main() {
    let args: Vec<String> = env::args().collect();

    let (input, dict_output, gaddag_output) = parse_args(&args).unwrap_or_else(|| {
        eprintln!(
            "Usage: {} --input <wordlist.txt> --output <dict.fst> [--gaddag <gaddag.fst>]",
            args[0]
        );
        process::exit(1);
    });

    eprintln!("Reading word list from: {}", input.display());
    let content = fs::read_to_string(&input).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {e}", input.display());
        process::exit(1);
    });

    let words: Vec<&str> = content.lines().collect();
    eprintln!("Read {} lines", words.len());

    // Build dictionary FST
    eprintln!("Building dictionary FST...");
    let dict_bytes = FstDictionary::build(&words).unwrap_or_else(|e| {
        eprintln!("Failed to build dictionary: {e}");
        process::exit(1);
    });
    fs::write(&dict_output, &dict_bytes).unwrap_or_else(|e| {
        eprintln!("Failed to write {}: {e}", dict_output.display());
        process::exit(1);
    });

    // Verify
    let dict = FstDictionary::from_bytes(dict_bytes).unwrap();
    eprintln!(
        "Dictionary FST: {} words, written to {}",
        dict.len(),
        dict_output.display()
    );

    // Build GADDAG if requested
    if let Some(gaddag_path) = gaddag_output {
        eprintln!("Building GADDAG FST...");
        let gaddag_bytes = Gaddag::build(&words).unwrap_or_else(|e| {
            eprintln!("Failed to build GADDAG: {e}");
            process::exit(1);
        });
        fs::write(&gaddag_path, &gaddag_bytes).unwrap_or_else(|e| {
            eprintln!("Failed to write {}: {e}", gaddag_path.display());
            process::exit(1);
        });
        eprintln!(
            "GADDAG FST: {} bytes, written to {}",
            gaddag_bytes.len(),
            gaddag_path.display()
        );
    }

    eprintln!("Done.");
}

fn parse_args(args: &[String]) -> Option<(PathBuf, PathBuf, Option<PathBuf>)> {
    let mut input = None;
    let mut output = None;
    let mut gaddag = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                input = Some(PathBuf::from(args.get(i)?));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(args.get(i)?));
            }
            "--gaddag" => {
                i += 1;
                gaddag = Some(PathBuf::from(args.get(i)?));
            }
            _ => return None,
        }
        i += 1;
    }

    Some((input?, output?, gaddag))
}
