//! chromsize
//! Alejandro Gonzales-Irribarren, 2024
//!
//! `chromsize` is a utility designed to extract chromosome names
//! and their corresponding lengths from FASTA files. It supports
//! both plain and gzipped FASTA formats [and .2bit] and offers an option to
//! include only the accession ID from the FASTA or 2bit headers.

use clap::{self, Parser};
use num_cpus;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    name = "chromsize",
    version = env!("CARGO_PKG_VERSION"),
    author = "Alejandro Gonzales-Irribarren <alejandrxgzi@gmail.com>",
    about = "just get your chrom sizes"
)]
pub struct Args {
    #[clap(
        short = 'i',
        long = "input",
        help = "Path to FASTA/2bit file",
        value_name = "PATH",
        required = true
    )]
    pub input: PathBuf,

    #[clap(
        short = 'o',
        long = "output",
        help = "Path to output chrom sizes",
        value_name = "OUTPUT",
        required = true
    )]
    pub out: PathBuf,

    #[clap(
        short = 't',
        long,
        help = "Number of threads",
        value_name = "THREADS",
        default_value_t = num_cpus::get()
    )]
    pub threads: usize,

    #[clap(
        short = 'a',
        long = "accession-only",
        help = "only keep the accession id part of the header (stop after blank)",
        default_value_t = false
    )]
    pub accession_only: bool,
}
