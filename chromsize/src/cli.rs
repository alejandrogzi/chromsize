use clap::{self, Parser};
use num_cpus;
use std::path::PathBuf;

/// Command line arguments for the chromsize application.
///
/// This struct defines the configuration options that can be passed
/// to the chromsize program, including input file, output file, and
/// threading options.
#[derive(Parser, Debug)]
#[clap(
    name = "chromsize",
    version = env!("CARGO_PKG_VERSION"),
    author = "Alejandro Gonzales-Irribarren <alejandrxgzi@gmail.com>",
    about = "just get your chrom sizes"
)]
pub struct Args {
    /// Path to sequence file (use '-' or omit to read stdin)
    #[clap(
        short = 's',
        long = "sequence",
        help = "Path to sequence file (FASTA/2bit, use '-' or omit to read stdin)",
        value_name = "SEQUENCE",
        default_value = "-"
    )]
    pub sequence: PathBuf,

    /// Path to output chrom sizes
    #[clap(
        short = 'o',
        long = "output",
        help = "Path to output chrom sizes",
        value_name = "OUTPUT",
        required = true
    )]
    pub out: PathBuf,

    /// Number of threads
    #[clap(
        short = 't',
        long,
        help = "Number of threads",
        value_name = "THREADS",
        default_value_t = num_cpus::get()
    )]
    pub threads: usize,
}
