use chromsize::{get_sizes, writer};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use twobit::convert::fasta::FastaReader;
use twobit::convert::to_2bit;

const FASTA: &str = ">chr1\nACGT\n>chr2\nAAC\n";
const EXPECTED_OUTPUT: &str = "chr1\t4\nchr2\t3\n";

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after UNIX_EPOCH")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "chromsize-tests-{}-{}",
            std::process::id(),
            nanos
        ));
        fs::create_dir_all(&path).expect("failed to create temporary test directory");
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn expected_sizes() -> Vec<(String, u64)> {
    vec![("chr1".to_string(), 4), ("chr2".to_string(), 3)]
}

fn assert_sizes_and_output(input: &Path, outdir: &Path, outname: &str) {
    let sizes = get_sizes(input).expect("failed to parse sequence input");
    assert_eq!(sizes, expected_sizes());

    writer(&sizes, outdir, outname.to_string()).expect("failed to write chrom sizes output");
    let output = fs::read_to_string(outdir.join(outname)).expect("failed to read output file");
    assert_eq!(output, EXPECTED_OUTPUT);
}

#[test]
fn parses_fa_and_writes_expected_output() {
    let tmp = TempDir::new();
    let fa_path = tmp.path.join("sample.fa");
    fs::write(&fa_path, FASTA).expect("failed to write FASTA fixture");

    assert_sizes_and_output(&fa_path, &tmp.path, "fa.chrom.sizes");
}

#[test]
fn parses_fa_gz_and_writes_expected_output() {
    let tmp = TempDir::new();
    let gz_path = tmp.path.join("sample.fa.gz");
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(FASTA.as_bytes())
        .expect("failed to gzip FASTA fixture");
    let compressed = encoder.finish().expect("failed to finish gzip stream");
    fs::write(&gz_path, compressed).expect("failed to write gz fixture");

    assert_sizes_and_output(&gz_path, &tmp.path, "fa-gz.chrom.sizes");
}

#[test]
fn parses_2bit_and_writes_expected_output() {
    let tmp = TempDir::new();
    let twobit_path = tmp.path.join("sample.2bit");
    let fasta_reader = FastaReader::mem_open(FASTA.as_bytes().to_vec())
        .expect("failed to create FASTA reader for 2bit conversion");
    let mut twobit_file = File::create(&twobit_path).expect("failed to create 2bit fixture file");
    to_2bit(&mut twobit_file, &fasta_reader).expect("failed to create 2bit fixture");
    twobit_file.flush().expect("failed to flush 2bit fixture");

    assert_sizes_and_output(&twobit_path, &tmp.path, "2bit.chrom.sizes");
}
