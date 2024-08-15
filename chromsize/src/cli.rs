use clap::{self, Parser};
use num_cpus;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    name = "chromsize",
    version = "0.0.1",
    author = "Alejandro Gonzales-Irribarren <alejandrxgzi@gmail.com>",
    about = "just get your chrom sizes"
)]
pub struct Args {
    #[clap(
        short = 'f',
        long = "fasta",
        help = "Path to FASTA file",
        value_name = "FASTA",
        required = true
    )]
    pub fasta: PathBuf,

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
}
