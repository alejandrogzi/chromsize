use clap::Parser;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

const TOOLS: [&str; 10] = [
    "seqkit fx2tab --length --name --header-line {assembly} > chrom.sizes",
    "target/release/chromsize -s {assembly} -o chrom.sizes",
    "faidx {assembly} -i chromsizes > chrom.sizes",
    "samtools faidx {assembly} && wait | cut -f1,2 {assembly}.fai > chrom.sizes",
    "faSize -detailed -tab {assembly} > chrom.sizes",
    "awk '/^>/ {if (seqlen){print seqlen}; print ;seqlen=0;next; } { seqlen += length($0)}END{print seqlen}' {assembly} > chrom.sizes",
    "awk '/^>/{if (l!=\"\") print l; print; l=0; next}{l+=length($0)}END{print l}' {assembly} > chrom.sizes",
    "bioawk -c fastx '{print \">\" $name ORS length($seq)}' {assembly} > chrom.sizes",
    "cat {assembly} | awk '$0 ~ \">\" {if (NR > 1) {print c;} c=0;printf substr($0,2,100) \"\t\"; } $0 !~ \">\" {c+=length($0);} END { print c; }' > chrom.sizes",
    "bioawk -c fastx '{ print $name, length($seq) }' < {assembly} > chrom.sizes",
];
const ASSEMBLIES: [&str; 9] = [
    "GCF_000146045.2_R64_genomic.fa",
    "ce11.fa",
    "dm6.fa",
    "canFam4.fa",
    "danRer11.fa",
    "GRCh38.primary_assembly.genome.fa",
    "GCA_002915635.3.fa",
    "GCF_019279795.1.fa",
    "GCA_027579735.1.fa",
];
const STDOUT: &str = "chrom.sizes";
const CSV: &str = "chrom.sizes.csv";
const MD: &str = "chrom.sizes.md";

/// Command line arguments for the benchmark utility.
///
/// This struct defines configuration options for running performance benchmarks
/// of various chromosome size extraction tools using hyperfine.
#[derive(Debug, Parser)]
pub struct Args {
    /// Path to the reference directory containing test assemblies
    #[clap(
        short = 'd',
        long = "dir",
        help = "Path to the reference directory",
        default_value = "assets"
    )]
    assets: PathBuf,

    /// Additional arguments to pass to the hyperfine benchmarking tool
    #[clap(short = 'a',
        value_delimiter = ',',
        num_args = 1..,
        help = "Extra arguments to pass to hyperfine"
    )]
    hyperfine_args: Vec<String>,
}

/// Configuration for hyperfine benchmark execution.
///
/// This struct encapsulates all parameters needed to run hyperfine benchmarks,
/// including warmup runs, execution limits, output formats, and commands to test.
pub struct HyperfineCall {
    /// Number of warmup runs before actual benchmarking
    pub warmup: u32,
    /// Minimum number of benchmark runs
    pub min_runs: u32,
    /// Maximum number of benchmark runs (optional)
    pub max_runs: Option<u32>,
    /// Path to export results in CSV format
    pub export_csv: Option<String>,
    /// Path to export results in Markdown format
    pub export_markdown: Option<String>,
    /// Parameterized variables for command substitution
    pub parameters: Vec<(String, Vec<String>)>,
    /// Setup command to run before each benchmark
    pub setup: Option<String>,
    /// Cleanup command to run after each benchmark
    pub cleanup: Option<String>,
    /// List of commands to benchmark
    pub commands: Vec<String>,
    /// Additional hyperfine command line arguments
    pub extras: Vec<String>,
}

impl Default for HyperfineCall {
    /// Creates a default HyperfineCall with sensible baseline settings.
    ///
    /// Sets up reasonable defaults for benchmarking with 3 warmup runs
    /// and 5 minimum runs, suitable for most performance testing scenarios.
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
    /// Executes the hyperfine benchmark with the configured parameters.
    ///
    /// This method builds and executes a hyperfine command using the struct's
    /// configuration. It sets up all command line arguments including warmup,
    /// runs, exports, parameters, setup/cleanup commands, and the actual
    /// benchmark commands.
    ///
    /// # Returns
    ///
    /// ExitStatus from the hyperfine process execution
    ///
    /// # Panics
    ///
    /// Panics if hyperfine command cannot be executed
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

/// Runs comprehensive benchmarks for chromosome size extraction tools.
///
/// This function sets up and executes hyperfine benchmarks comparing multiple
/// tools across various genome assemblies. It configures warmup runs, output formats,
/// parameterized assembly testing, and cleanup operations.
///
/// # Returns
///
/// Tuple containing (csv_path, markdown_path) for the benchmark results
///
/// # Errors
///
/// Returns error if benchmark fails or if file system operations fail
///
/// # Examples
///
/// ```ignore
/// let (csv, md) = benchmark()?;
/// println!("Results: {} {}", csv, md);
/// ```
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
            .map(|cmd| cmd.to_string())
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

/// Main entry point for the benchmark utility.
///
/// This function parses command line arguments, runs the benchmark suite,
/// and reports the location of result files or any errors that occurred.
///
/// # Examples
///
/// ```ignore
/// // Run with custom asset directory
/// ./chromsize-benchmark -d /path/to/assemblies
/// ```
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
