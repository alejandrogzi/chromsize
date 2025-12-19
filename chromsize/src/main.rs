//! chromsize
//! Alejandro Gonzales-Irribarren, 2024
//!
//! `chromsize` is a utility designed to extract chromosome names
//! and their corresponding lengths from input files. It supports
//! both plain and gzipped input formats and offers an option to
//! include only the accession ID from the input headers.
//!
//! ## Usage
//!
//! To use `chromsize`, you typically provide the input input file
//! and specify the desired output file.
//!
//! ```bash
//! chromsize [OPTIONS] --input <input> --output <OUTPUT>
//! ```
//!
//! ## Options
//!
//! Here's a breakdown of the available command-line options:
//!
//! * **`-i`, `--input <input>`**
//!     * **Purpose**: Specifies the path to the input input file. This is a **required** option.
//!     * **Example**: `--input genome.input` or `--input sequences.input.gz`
//!
//! * **`-o`, `--output <OUTPUT>`**
//!     * **Purpose**: Specifies the path where the output chromosome sizes will be written.
//!                    This is a **required** option. The output will typically be a tab-separated
//!                    file with chromosome names and their lengths.
//!     * **Example**: `--output chrom_lengths.txt`
//!
//! * **`-t`, `--threads <THREADS>`**
//!     * **Purpose**: Sets the number of threads to use for processing. This can speed up processing
//!                    for large input files.
//!     * **Default**: `8`
//!     * **Example**: `--threads 4` (to use 4 threads)
//!
//! * **`-a`, `--accession-only`**
//!     * **Purpose**: A flag that, when present, instructs `chromsize` to only keep the accession ID
//!                    part of the input header. This means it will stop reading the header at the first
//!                    blank space. If omitted, the entire header line up to the first newline character
//!                    will be used as the chromosome name.
//!     * **Example**: `--accession-only`
//!
//! * **`-h`, `--help`**
//!     * **Purpose**: Displays a help message with usage information and available options.
//!     * **Example**: `chromsize --help`
//!
//! * **`-V`, `--version`**
//!     * **Purpose**: Prints the version information of the `chromsize` tool.
//!     * **Example**: `chromsize --version`
//!
//! ## Example Usage Scenarios
//!
//! 1.  **Get chromosome sizes from a plain input file, using full headers, with default threads:**
//!
//!     ```bash
//!     chromsize --input input.fa --output chrom_sizes.txt
//!     ```
//!
//! 2.  **Get chromosome sizes from a gzipped input file, extracting only accession IDs, using 4 threads:**
//!
//!     ```bash
//!     chromsize --input input.input.gz --output accession_sizes.txt --accession-only --threads 4
//!     ```
//!
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
