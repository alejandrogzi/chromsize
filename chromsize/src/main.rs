use clap::{self, Parser};

pub mod cli;
use cli::Args;

use chromsize::*;

fn main() {
    let args = Args::parse();

    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .unwrap();

    let sizes = chromsize(args.fasta).expect("Error getting chromosome sizes");
    writer(sizes, args.out);
}
