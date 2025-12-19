use clap::{self, Parser};

pub mod cli;
use cli::Args;

use chromsize::*;
use std::process;

/// Main entry point for the chromsize application.
///
/// This function parses command line arguments, initializes the thread pool,
/// processes the sequence file to get chromosome sizes, and writes the results.
///
/// # Examples
///
/// ```ignore
/// // Run with custom threads and input/output files
/// ./chromsize -s input.fa -o chrom.sizes -t 8
/// ```
fn main() {
    let args = Args::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("ERROR: failed to initialize thread pool: {}", e);
            process::exit(1);
        });

    let sizes = match get_sizes(args.sequence) {
        Ok(sizes) => sizes,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = writer(&sizes, args.out) {
        eprintln!("ERROR: {}", e);
        process::exit(1);
    }
}
