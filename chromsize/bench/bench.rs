use clap::Parser;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

const TOOLS: [&str; 10] = [
    "seqkit fx2tab --length --name --header-line {assembly} > chrom.sizes",
    "target/release/chromsize -f {assembly} -o chrom.sizes",
    "faidx {assembly} -i chromsizes > chrom.sizes",
    "samtools faidx {assembly} && wait | cut -f1,2 {assembly}.fai > chrom.sizes",
    "faSize -detailed -tab {assembly} > chrom.sizes",
    "awk '/^>/ {if (seqlen){print seqlen}; print ;seqlen=0;next; } { seqlen += length($0)}END{print seqlen}' {assembly} > chrom.sizes",
    "awk '/^>/{if (l!=\"\") print l; print; l=0; next}{l+=length($0)}END{print l}' {assembly} > chrom.sizes",
    "bioawk -c fastx '{print \">\" $name ORS length($seq)}' {assembly} > chrom.sizes",
    "cat {assembly} | awk '$0 ~ \">\" {if (NR > 1) {print c;} c=0;printf substr($0,2,100) \"\t\"; } $0 !~ \">\" {c+=length($0);} END { print c; }' > chrom.sizes",
    "bioawk -c fastx '{ print $name, length($seq) }' < {assembly} > chrom.sizes",
];
const ASSEMBLIES: [&str; 9] = ["GCF_000146045.2_R64_genomic.fa", "ce11.fa", "dm6.fa", "canFam4.fa",  "danRer11.fa", "GRCh38.primary_assembly.genome.fa","GCA_002915635.3.fa",  "GCF_019279795.1.fa",  "GCA_027579735.1.fa"];
const STDOUT: &str = "chrom.sizes";
const CSV: &str = "chrom.sizes.csv";
const MD: &str = "chrom.sizes.md";

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(
        short = 'd',
        long = "dir",
        help = "Path to the reference directory",
        default_value = "assets"
    )]
    assets: PathBuf,

    #[clap(short = 'a', 
        value_delimiter = ',',
        num_args = 1..,
        help = "Extra arguments to pass to hyperfine"
    )]
    hyperfine_args: Vec<String>,
}

pub struct HyperfineCall {
    pub warmup: u32,
    pub min_runs: u32,
    pub max_runs: Option<u32>,
    pub export_csv: Option<String>,
    pub export_markdown: Option<String>,
    pub parameters: Vec<(String, Vec<String>)>,
    pub setup: Option<String>,
    pub cleanup: Option<String>,
    pub commands: Vec<String>,
    pub extras: Vec<String>,
}

impl Default for HyperfineCall {
    fn default() -> Self {
        Self {
            warmup: 3,
            min_runs: 5,
            max_runs: None,
            export_csv: None,
            export_markdown: None,
            parameters: Vec::new(),
            setup: None,
            cleanup: None,
            commands: Vec::new(),
            extras: Vec::new(),
        }
    }
}

impl HyperfineCall {
    pub fn invoke(&self) -> ExitStatus {
        let mut command = Command::new("hyperfine");

        command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::null());

        command.arg("--warmup").arg(self.warmup.to_string());
        command.arg("--min-runs").arg(self.min_runs.to_string());
        if let Some(export_csv) = &self.export_csv {
            command.arg("--export-csv").arg(export_csv);
        }
        if let Some(export_markdown) = &self.export_markdown {
            command.arg("--export-markdown").arg(export_markdown);
        }
        for (flag, values) in &self.parameters {
            command.arg("-L").arg(flag).arg(values.join(","));
        }
        if let Some(setup) = &self.setup {
            command.arg("--setup").arg(setup);
        }
        if let Some(cleanup) = &self.cleanup {
            command.arg("--cleanup").arg(cleanup);
        }
        if let Some(max_runs) = self.max_runs {
            command.arg("--max-runs").arg(max_runs.to_string());
        }
        if !self.extras.is_empty() {
            command.args(&self.extras);
        }

        for cmd in &self.commands {
            command.arg(cmd);
        }

        command.status().expect("Failed to run hyperfine")
    }
}

fn benchmark() -> Result<(String, String), Box<dyn std::error::Error>> {
    let args = Args::parse();

    std::fs::create_dir_all("runs")?;
    let assets = args.assets.to_string_lossy();

    #[allow(clippy::needless_update)]
    let code = HyperfineCall {
        warmup: 5,
        min_runs: 10,
        max_runs: Some(20),
        export_csv: Some(format!("runs/{}", CSV).to_string()),
        export_markdown: Some(format!("runs/{}", MD).to_string()),
        parameters: vec![(
            "assembly".to_string(),
            ASSEMBLIES
                .iter()
                .map(|s| format!("{}/{}", assets, s))
                .collect(),
        )],
        setup: Some("cargo build --release".to_string()),
        cleanup: Some(format!("rm -f {} assets/*.fai", STDOUT)),
        commands: TOOLS
            .iter()
            .map(|cmd| format!("{}", cmd))
            .collect::<Vec<String>>(),
        extras: args
            .hyperfine_args
            .iter()
            .map(|s| format!("--{}", s))
            .collect(),
        ..Default::default()
    }
    .invoke()
    .code()
    .expect("Benchmark terminated unexpectedly");

    if code != 0 {
        return Err(format!("Benchmark failed with exit code {}", code).into());
    }

    Ok((format!("runs/{}", CSV), format!("runs/{}", MD)))
}

fn main() {
    match benchmark() {
        Ok((csv, md)) => {
            println!("Benchmark results saved to:");
            println!("  - {}", csv);
            println!("  - {}", md);
        }
        Err(e) => eprintln!("{}", e),
    }
}
