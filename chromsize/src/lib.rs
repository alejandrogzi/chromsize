//! chromsize
//! Alejandro Gonzales-Irribarren, 2024
//!
//! `chromsize` is a utility designed to extract chromosome names
//! and their corresponding lengths from FASTA files. It supports
//! both plain and gzipped FASTA formats and offers an option to
//! include only the accession ID from the FASTA headers.
//!
//! ## Usage
//!
//! To use `chromsize`, you typically provide the input FASTA file
//! and specify the desired output file.
//!
//! ```bash
//! chromsize [OPTIONS] --fasta <FASTA> --output <OUTPUT>
//! ```
//!
//! ## Options
//!
//! Here's a breakdown of the available command-line options:
//!
//! * **`-f`, `--fasta <FASTA>`**
//!     * **Purpose**: Specifies the path to the input FASTA file. This is a **required** option.
//!     * **Example**: `--fasta genome.fasta` or `--fasta sequences.fasta.gz`
//!
//! * **`-o`, `--output <OUTPUT>`**
//!     * **Purpose**: Specifies the path where the output chromosome sizes will be written.
//!                    This is a **required** option. The output will typically be a tab-separated
//!                    file with chromosome names and their lengths.
//!     * **Example**: `--output chrom_lengths.txt`
//!
//! * **`-t`, `--threads <THREADS>`**
//!     * **Purpose**: Sets the number of threads to use for processing. This can speed up processing
//!                    for large FASTA files.
//!     * **Default**: `8`
//!     * **Example**: `--threads 4` (to use 4 threads)
//!
//! * **`-a`, `--accession-only`**
//!     * **Purpose**: A flag that, when present, instructs `chromsize` to only keep the accession ID
//!                    part of the FASTA header. This means it will stop reading the header at the first
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
//! 1.  **Get chromosome sizes from a plain FASTA file, using full headers, with default threads:**
//!
//!     ```bash
//!     chromsize --fasta input.fa --output chrom_sizes.txt
//!     ```
//!
//! 2.  **Get chromosome sizes from a gzipped FASTA file, extracting only accession IDs, using 4 threads:**
//!
//!     ```bash
//!     chromsize --fasta input.fasta.gz --output accession_sizes.txt --accession-only --threads 4
//!     ```
//!

pub mod size;
pub use size::*;
