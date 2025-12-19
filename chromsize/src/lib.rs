//! chromsize: fast chromosome size extraction from FASTA and 2bit.
//!
//! chromsize reads sequence data (FASTA or 2bit) from a file or stdin, detects
//! the input format by content, and returns chromosome sizes as `(name, size)`
//! pairs. Gzip-compressed input is auto-detected and decompressed for both
//! files and stdin.
//!
//! ## CLI
//! ```
//! chromsize --sequence <SEQUENCE> --output <OUTPUT> [-t <THREADS>]
//!
//! -s, --sequence <SEQUENCE>  Sequence file (FASTA/2bit, use '-' or omit to read stdin)
//! -o, --output <OUTPUT>      Output path for chrom.sizes
//! -t, --threads <THREADS>    Number of threads (default: all cores)
//! ```
//!
//! Examples:
//! - stream FASTA: `cat genome.fa | chromsize -o chrom.sizes`
//! - stream gzip FASTA: `zcat genome.fa.gz | chromsize -o chrom.sizes`
//! - file input: `chromsize -s genome.fa -o chrom.sizes`
//! - 2bit from stdin: `cat genome.2bit | chromsize -s - -o chrom.sizes`
//!
//! ## Library
//! ```rust
//! use std::path::PathBuf;
//!
//! let input = PathBuf::from("/path/to/genome.fa");
//! let output = PathBuf::from("/path/to/chrom.sizes");
//!
//! let sizes = chromsize::get_sizes(&input).expect("failed to read input");
//! chromsize::writer(&sizes, &output).expect("failed to write sizes");
//! ```
//!
//! The `get_sizes` function auto-detects FASTA vs 2bit by content and supports
//! stdin when the input path is `-`.
pub mod size;
pub use size::*;
