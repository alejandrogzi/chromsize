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

    let sizes = get_sizes(args.fasta).expect("ERROR. Could not get sizes:");
    writer(sizes, args.out);
}
